# ADR-008 — Monomial representation: interned arena, packed key, and fail-closed overflow

**Status:** Ratified 2026-07-31
**Reversibility:** one-way for the interning/id structure; cheap for the field width
**Amended:** 2026-07-31 — ids are **content-derived**, not first-encounter-ordered;
`W_KEY` and `W_RAW` are separate; arena exhaustion has an error path and a memory model;
the deciding microbenchmark is respecified against a synthetic recorded trace
(critique-engineering §5, §10, §11, §17).
**Gates lanes:** P1, P2, P3, P5, G2, G7.
**Evidence:** `docs/research/algorithms-and-representation.md` §1.1–§1.6;
`docs/research/critique-engineering.md` §5, §10, §11, §17.

---

## Context

The source spec says bit-packed monomials with comparison as a single integer compare are
"most of your Gröbner performance." **Measured, that is false, and believing it would
misdirect a lane.**

Where an F4 run's time actually goes (Groebner.jl, mod `2^30+3`, drl, seconds; the table
captures >99% of runtime):

| phase | Cyclic-9 | Eco-14 | Goodwin (w.) | Yang1 |
|---|---|---|---|---|
| Pair selection | 4.07 | 1.95 | 20.57 | 1.68 |
| Symbolic preprocessing | 8.34 | 8.64 | 5.90 | 5.62 |
| **Linear algebra** | **242.03** | **168.83** | **284.01** | **7.73** |
| Pair update | 6.71 | 2.03 | 79.50 | 10.80 |
| Auto-reduction | 4.67 | 44.60 | 0.00 | 0.19 |
| **LA share** | **91%** | **75%** | **73%** | **28%** |

Packing itself, measured by the same source: **~15%**. The divisor-query index, measured by
Roune & Stillman with everything else held fixed: plain monomial list 1270 s vs
divmask+kd-tree 112 s on `hcyclic8` (11×), and **>8 hours vs 1333 s** on `yang1` (>20×).
S-pair criteria, on `yang1`: 1 998 099 720 pairs generated, 148 812 surviving — four orders
of magnitude.

And the structural point the spec misses: **in F4, monomial comparison largely
disappears.** The monomials of a Macaulay matrix are sorted once to assign column indices;
after that every inner-loop comparison is of small integers. The packed compare is used in
matrix *construction* and in the S-pair queue, not in the 73–91% that is elimination.

So the driver ranking is: (1) not doing the work — Gebauer–Möller, 10⁴×; (2) sparse GF(p)
linear algebra, 73–91%; (3) divisor-query index, 10–20×; (4) interning with a
multiplicative hash; (5) bit-packing, 15%.

**The representation is still a genuine one-way door — but because everything above
inherits the *interning and id structure*, not because compare speed dominates.**

Separately: **exponent wraparound is the single most dangerous silent failure in the
library.** A wrapped exponent field silently yields a correct Gröbner basis *of a different
ideal*. Every certificate in the verification thesis passes. There is no downstream
detector.

---

## Decision

### 1. Terms are `(MonomialId, Coeff)` pairs into an interned arena

```rust
struct MonomialEntry {
    key:     [u64; W_KEY],  // order-normalized comparison key (ADR-009); compare = word compare
    raw:     [u64; W_RAW],  // raw packed exponents; divisibility, lcm, gcd, degree queries
    divmask: u64,           // Bloom-style filter for fast NEGATIVE divisibility answers
}
pub struct MonomialId(u32);
// A polynomial's terms are (MonomialId, C), never (Monomial, C).
```

Roughly three words per **distinct** monomial, paid once, not per term. `W_KEY` and `W_RAW`
are **const generics** over `{1, 2, 4, 8}` (ADR-006 Tier M).

**`W_KEY` and `W_RAW` are separate.** *Amended 2026-07-31.* The original declared both as
`[u64; W]` with a single `W`, but the field counts differ by order: lex needs `n` key
fields, grlex needs `n+1`, grevlex needs `n` (ADR-009), while `raw` always needs exactly
`n`. For grlex at 8 variables with 8-bit fields, `raw` fits in one word and `key` needs
two — a single `W` either wastes a word on every distinct monomial or silently overflows
the key.

The hash used for interning is **multiplicative** (`h(u) + h(v) = h(uv)`), so a product's
hash is a sum and the matrix-construction phase never re-hashes from scratch.

**Ids are content-derived, not encounter-ordered.** *Amended 2026-07-31, and this is a
determinism decision, not an optimization.* An interner is a shared mutable accumulator,
which ADR-012 §5 bans; symbolic preprocessing — the natural second parallel target after
row reduction — is nothing but interning. If ids were assigned in first-encounter order
(ADR-012 §4 as originally written), then `terms.par_iter().map(|t| ring.intern(t)).collect()`
— which *looks* like the permitted ordered-combination shape, because the collection is
ordered — assigns ids in thread-arrival order. Gate 0's thread matrix catches that only on
instances whose tie-breaks actually consult id order, which is data-dependent: it passes
for months and then fails once, and the minimizer cannot shrink a schedule bug because the
bug is not in the input.

So: **`MonomialId` is a pure function of the packed key**, via a deterministic content hash
with a fixed collision-resolution order (linear probing in a table whose capacity growth is
itself a pure function of the insert multiset). Parallel interning is then deterministic by
construction and the question evaporates. Paired with this, an invariant that belongs in
every multivariate lane brief:

> **INV-M1. No tie-break anywhere may consult `MonomialId` ordering.** Tie-break on `key`,
> which is content-derived and totally ordered by construction.

If content-derived ids prove impractical at P2 (they should not — the key is already the
hash input), the fallback is that **interning is serialized or two-phase**: collect
candidates in index order, intern single-threaded in that order. That fallback caps the
parallel speedup of matrix construction, and if it is taken the plan must stop describing
symbolic preprocessing as parallelizable.

**This is the one-way door.** Everything above — polynomial storage, the S-pair queue, the
divisor index, F4's column assignment — inherits it.

**There is no global interner and no ambient state.** The arena is owned by the `Ring`
context object (ADR-009) and reached explicitly: an `MPoly` holds an `Arc<Ring>`, so it is a
**self-contained `Send + Sync` value** that carries its own arena reachably and can be moved
between threads, serialized, or compared without consulting anything ambient. "Interned"
here means *arena-owned by an explicitly-passed ring*, never *global table*. Two `MPoly`s
over different rings are not comparable and the type says so; `MonomialId` is meaningful
only relative to the `Ring` that issued it, and every API taking a `MonomialId` also takes
(or holds) the ring.

This is stated in these words because "no global interner" and "terms are interned ids" are
both true and read as contradictory. They are not: the rejected thing is global mutable
state, and this design has none.

### 2. Divisibility gets a first-class index, not a linear scan

The `divmask` gives a fast *negative* answer; the positive path goes through a kd-tree (or
equivalent) over the arena. This is item (3) in the driver ranking and it is worth 10–20×,
so it is designed in from the start rather than added when the profile says so. A lane
brief that says "make monomial compare fast" must instead say "build the divisor index".

### 3. Overflow is fail-closed **and** recoverable

Three facts make this cheap:

- **If every field has the same width, an overflow in any exponent field implies an
  overflow in the total-degree field**, because `a_i ≤ Σ a_j = deg` always. One comparison
  of the degree field suffices — no per-field check needed.
- **SWAR guard bits catch it anyway.** Reserve the top bit of each field as zero; after a
  field-wise add, `(sum & GUARD_MASK) != 0` iff some field carried into its guard. One AND
  and one compare per word.
- **Exponents only grow during a run**, so overflow is recoverable by widening and
  restarting; the work lost is bounded by the run so far.

Therefore:

```rust
impl Ring {
    pub fn mul_monomial(&self, a: MonomialId, b: MonomialId) -> Result<MonomialId>;
    //   Err(Unsupported::TotalDegree { got, max })       on guard-bit trip. Never wraps.
    //   Err(Unsupported::MonomialArenaFull { capacity }) on id-space exhaustion.
    pub unsafe fn mul_monomial_unchecked(&self, a: MonomialId, b: MonomialId) -> MonomialId;
    //   Only where a bound was PROVED at the call site, with the proof in a comment.

    pub fn arena_stats(&self) -> ArenaStats;   // distinct monomials, bytes, load factor
}
```

**Id exhaustion is an error, not a panic.** *Added 2026-07-31.* `MonomialId(u32)` caps the
arena at 2³² distinct monomials. That is very probably enough, but the original signature
returned `Result` only for the guard-bit trip, so exhaustion would have been an index panic
— violating ADR-011's absolute no-panic rule in a one-way-door type.

**The arena has a memory model, stated rather than assumed.** It is monotonic: monomials
interned for S-pairs later eliminated by Gebauer–Möller are never reclaimed, and a long ℚ
run over 2000 primes on a shared ring accumulates the union across primes. That is probably
fine — the monomial *set* is largely prime-independent — but "probably fine" was a claim
made by omission. `arena_stats()` plus a committed corpus assertion on the largest instance
makes it measured. Eviction is **not** implemented: a compacting arena would renumber ids,
and INV-M1 exists precisely so nothing depends on id values, but renumbering would still
invalidate every outstanding `MPoly`, which ADR-020 forbids.

and the top-level driver owns a **widen-and-restart loop**: on overflow, abort, re-encode
every monomial at `w' = 2w` (or fall back to unpacked `Vec<u32>`), restart. The restart is
recorded as a `TraceEvent::WidenRestart { from, to }` so the run stays replayable
(ADR-012).

**Never panic on overflow** — a library that panics on a legitimate input is unusable
inside a geometry kernel with its own error discipline (ADR-011).
**Never wrap** — see §Context.

### 4. Capacity, stated so a lane brief can cite it

With `w`-bit fields and one guard bit, payload max is `2^(w-1) − 1`:

| field width `w` | total-degree bound | fields / 64-bit word | vars in one word (grevlex layout) |
|---|---|---|---|
| 4 | 7 | 16 | 16 |
| 8 | **127** | 8 | **8** |
| 16 | 32 767 | 4 | 4 |
| 32 | 2 147 483 647 | 2 | 2 |

Corroborated independently: Groebner.jl reports 31 variables at total degree ≤ 127 (8-bit
fields, 4 words); Singular documents "at least 32767" (16-bit fields), order-dependent —
and order-dependence is not an accident, because the number of fields the key needs depends
on the order (ADR-009).

**Practical reading:** for the resultant/elimination workloads a geometry consumer
generates, `n ≤ 8` and `D ≤ 127` fits in one word comfortably. For 15-variable systems it
is 2–4 words. **Above ~32 variables, packing buys little and unpacked `Vec<u32>` is the
honest representation** — Groebner.jl says exactly this and switches. resolvent switches
too, on the same criterion, and the switch is a `Ring` construction-time choice.

---

## Consequences

- **The lane briefs change shape.** The Layer-1 performance lane is *divisor index + S-pair
  criteria*, not *monomial compare*. Packing is a 15% lane and is scheduled as such.
- **Field width is a tuning knob, not a door.** It can be widened and the run restarted
  without recompilation (which requires ADR-009's runtime order, and is a second reason for
  it). This demotes what looked like the scariest Layer-1 decision to something recoverable.
- **`Result` on monomial multiply pollutes signatures** through S-pair construction and
  symbolic preprocessing. Accepted: the alternative is the silent-wrong-ideal failure, and
  `?` is cheap. The `unchecked` variant exists for proved-bounded call sites and requires a
  written proof at the call site — reviewed, not assumed.
- **Interning costs a hash lookup per monomial encountered.** That is item (4) in the driver
  ranking and is what makes matrix construction possible at all; it is not a cost to
  minimize away.
- **Open, and it must be measured before P1/P2/P3 start — experiment E-MONO.** Comparing
  two monomials by id requires a random load from the arena, and that cache miss may
  dominate the `u64` compare entirely, which would deflate the packing benefit *further*.

  **The harness is specified here, because the original specification deadlocked.** It said
  "microbenchmark … on a realistic S-pair queue workload", which requires a working
  Buchberger/F4 — lanes G1/G2, Wave 4 — while gating lanes P1/P2/P3 in Wave 2, which G2
  depends on. As scheduled, P2 waited on an experiment that waited on G2 that waited on P2.

  > **E-MONO.** Write a ~200-line throwaway Buchberger over GF(p) with plain `Vec<u32>`
  > exponents. Run it on Katsura-6 and Cyclic-6 and **record the operation trace**: an
  > ordered stream of `(lcm_query, divisibility_query, insert, compare)` events with their
  > operands. Commit the trace. Then replay it against (a) inline packed monomials in terms
  > and (b) ids plus arena lookup, and separately measure the divisor-query index's speedup
  > under each. Report medians of `k` runs with IQR on the pinned machine.
  >
  > The throwaway Buchberger is discarded afterwards and is explicitly *not* lane G1. It
  > shares no code with production and needs no certificate beyond "it terminates and its
  > output reduces the input to zero".

  If (a) wins for the S-pair queue specifically, the queue stores inline copies while the
  arena remains the identity authority — a local change, not a reversal. The **ownership**
  rule (the arena belongs to the `Ring`, ADR-020 §1) is fixed regardless, so the experiment
  decides a representation rather than an architecture.

---

## Alternatives considered and why rejected

**`u8` exponents with wrapping arithmetic and "degrees never get that big in practice."**
Rejected. It produces a correct basis of a different ideal, and every certificate passes.
There is no more dangerous failure mode available.

**Panicking on overflow.** Rejected. Violates ADR-011's no-panic rule, and a panic is
indistinguishable from a hang to an embedding kernel.

**Unbounded `Vec<u32>` exponents everywhere, no packing.** Rejected as the default — it
gives up the 15% and, more importantly, the compact `key`/`raw`/`divmask` triple that makes
the divisor index cache-friendly. **Kept as the automatic fallback above ~32 variables**,
which is where packing stops paying.

**Inline packed monomials in terms — `(PackedMon, Coeff)` — with no arena.** The strongest
alternative, and the one that most obviously satisfies "an `MPoly` is a self-contained
`Send + Sync` value with no ambient state". Rejected, because the arena-owned-by-the-ring
design satisfies that property *too* (see §Decision 1) while additionally giving: one copy
per distinct monomial rather than one per term occurrence; O(1) equality by id; the
multiplicative hash `h(u) + h(v) = h(uv)`, which is what makes symbolic preprocessing's
"have I seen this monomial?" lookup cheap; and a divisor index over identities rather than
over copies. Both msolve (hash tables with linear probing over exponent vectors, 32-bit
divisor masks) and Groebner.jl intern. The inline representation is kept for exactly one
place where it may win — the S-pair queue's cached keys, see §Consequences *Open*.

Note the honest part: the 10–20× divisor-index win is attributable to the **index**, not to
interning as such — a divmask can be computed from inline exponents. What interning buys is
the memory, the id equality, and the multiplicative hash. That is a smaller claim than the
index's, and it is the right size of claim.

**Storing only `key`, deriving `raw` on demand.** Rejected. Divisibility, lcm, gcd, and
degree queries all need raw exponents, and they are frequent enough (item 2 in the call-
frequency ranking) that decoding per query would dominate. One extra word per *distinct*
monomial is the right trade.

**Making field width a compile-time const the whole crate is generic over.** Rejected: it
forecloses widen-and-restart without recompilation, which is the mechanism that demotes
overflow from a door to a knob. `W` (word count) is const-generic at the *kernel* level
only, with the arena choosing `W` at construction.

---

## What would reverse this

- **The arena-lookup microbenchmark showing ids are a net loss.** Response: inline
  monomials in the S-pair queue while keeping the arena as the identity authority. Partial,
  local, and does not touch the id-based term representation.
- **A workload that is overwhelmingly high-variable** (48–66 variables, the `yang1`/`mayr42`
  class). Response: the unpacked fallback already handles it; the `Ring` chooses at
  construction.

The `(MonomialId, Coeff)` term structure does not reverse. That is the door, and its
reversal is a rewrite of every multivariate algorithm.
