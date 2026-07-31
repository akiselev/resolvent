# ADR-009 — The monomial order is runtime ring data, normalized into the comparison key

**Status:** Ratified 2026-07-31
**Reversibility:** one-way (it is visible in every multivariate signature)
**Amended:** 2026-07-31 — divisibility is **order-free** and is not one of the
order-specific sites; FGLM gets a dual-key arena rather than re-interning
(critique-engineering §9, §10).
**Gates lanes:** P1, P2, P3, G1, G2, G7.
**Evidence:** `docs/research/algorithms-and-representation.md` §1.4, §1.5;
`docs/research/critique-engineering.md` §9, §10.

---

## Context

Rust offers three ways to carry a monomial order, and the choice is usually presented as a
trade between speed and flexibility:

| Option | Compare cost | What it actually costs |
|---|---|---|
| (a) Type parameter `Poly<C, O: Order>` | inlined, static | Monomorphization across orders × coefficient rings × widths. Heterogeneous storage impossible. Every consumer reading an order from data needs a hand-written dispatch shim. FGLM becomes a type conversion (fine) and so does every debugging detour (not fine). |
| (b) Runtime data, branch or fn-pointer per compare | one predicted branch, or an indirect call | Loses inlining inside sorts; the indirect-call variant is genuinely slow in a comparison sort. |
| (c) Order normalized into the key at intern time | plain `u64` compare, no branch, no order | An order-specific encode at intern time and an order-specific constant subtract on multiply. Both O(1), both *outside* the sort inner loop. |

Option (c) exists because of a structural fact, not a trick: **all three practical orders
are non-negative integer matrix orders.** Fix a matrix `M` with non-negative entries and
compare `M·a` against `M·b` lexicographically; pack `M·a` **big-endian, most significant
weight first**, and comparison of two monomials is exactly unsigned comparison of the
packed words, with no order-specific branch at all.

- **lex**: `M = I`. Key `= (a_1, …, a_n)`, `n` fields.
- **grlex**: `M = [1…1; I]`. Key `= (|a|, a_1, …, a_n)`, `n+1` fields.
- **grevlex**: the naive matrix has a block of `−1`s, which breaks non-negativity. Replace
  it with complements. For `c ≥ D` (the degree bound), grevlex equals lex on

  `K(a) = ( |a|, c − a_n, c − a_{n−1}, …, c − a_2 )` — **`n` fields, and `a_1` is dropped.**

  *Proof.* If `|a| ≠ |b|` the first field decides, matching grevlex. If `|a| = |b|` and
  `a ≠ b`, let `k` be the largest index with `a_k ≠ b_k`. `k = 1` is impossible: it would
  force `a_j = b_j` for all `j ≥ 2`, and `|a| = |b|` would then give `a_1 = b_1`. So
  `k ≥ 2`, and scanning `K` from field 2 (which holds index `n`) the first difference is at
  index `k`, where `K(a) = c − a_k` and `K(b) = c − b_k`. So `K(a) > K(b)` iff `a_k < b_k`
  — the grevlex condition. ∎

  grevlex therefore needs no more fields than lex, and `a_1` is genuinely redundant.

**Multiplication still works in key space.** For lex and grlex the key is linear, so
`K(ab) = K(a) + K(b)` field-wise. For grevlex, `(c − a_i) + (c − b_i) = c + (c − (a_i+b_i))`,
so `K(ab) = K(a) + K(b) − C` where `C` is a constant packed word holding `c` in every
complement field and `0` in the degree field. **Two SWAR ops per word.** Underflow
(`a_i + b_i > c`) trips the same guard-bit test as overflow (ADR-008).

Divisibility `m | n` needs raw exponents (per-field `a_i ≤ b_i`). **It is computed from
`raw`, and on `raw` it is order-free.** That is why `MonomialEntry` carries `raw` alongside
`key` (ADR-008).

*Corrected 2026-07-31.* This ADR originally said divisibility "*is* order-dependent — unlike
compare", and listed "an order-specific divisibility direction" as the third of three
order-specific sites, "all O(1) and all *outside* sort inner loops". Both halves are wrong,
and the second is dangerous:

- Divisibility *in key space* is order-dependent (the direction flips in grevlex's
  complement fields), but nothing computes it in key space. `raw` holds raw packed
  exponents precisely so that divisibility, lcm, gcd and degree are a single SWAR per-field
  comparison with no order, no complements and no branch. ADR-008 already says `raw` exists
  for exactly these queries; this ADR failed to draw the consequence.
- Divisibility is **not** outside an inner loop. It is *the* inner loop of symbolic
  preprocessing and of reducer selection, which is why ADR-008's driver ranking puts the
  divisor-query index at 10–20×. An `Order`-matching branch there violates ADR-006's
  boundary rule verbatim ("at most one runtime `match` per *call*, never per element"), in
  the hottest loop in the library.

As originally written, a lane brief derived from this section would have produced exactly
that routine.

Separately: **there is no lex Gröbner lane.** msolve's pipeline is drl basis → FGLM
conversion to lex; Groebner.jl "specializes in computation in the degree reverse
lexicographical monomial ordering"; every benchmark table in the literature is drl.
Computing a lex basis directly on a system where drl+FGLM takes seconds routinely does not
terminate in useful time. So "compute a lex basis" is `drl-GB → FGLM`, and FGLM is its own
lane with its own certificate (the lex basis must reduce the drl basis to zero and vice
versa).

---

## Decision

**The monomial order is runtime data on the `Ring` context object, and it is normalized
into the packed comparison key at intern time.**

1. `Ring::new(vars, order)` takes the order as a value. `Order` is an enum:
   `Lex`, `GrLex`, `GrevLex`, `Block(Vec<Order>, Vec<usize>)`, `Matrix(Vec<Vec<u32>>)` —
   the last requiring non-negative entries, which is checked at construction (ADR-011:
   fail at construction).
2. **Comparison of two monomials is an unsigned compare of `[u64; W]`**, order-free and
   branch-free, using `key`.
3. **Order-specific work happens in exactly two places**, both O(1) per operation and both
   outside sort inner loops: **encode** (at intern time) and **the constant subtract on
   multiply**. Divisibility, lcm, gcd and total degree are computed from `raw` and are
   **order-free**. No divisibility routine anywhere matches on `Order`.
4. **A zero-cost newtype `Ordered<O>(Ring)` is available** for callers who want the order
   reflected in the type system. It carries no runtime cost and no separate implementation;
   it is a phantom-typed wrapper that asserts the wrapped ring's order at construction.
5. **The production Gröbner order is grevlex.** Lex is reached by FGLM, never computed
   directly. `groebner(ideal, Order::Lex)` is legal and internally does drl + FGLM,
   documented as such, so a caller does not accidentally request the fatal path.
6. **A ring may be created as a *conversion pair*, and then its arena carries two keys.**
   *Added 2026-07-31.*

   ```rust
   struct MonomialEntry {
       key_a:   [u64; W_KEY],   // the ring's primary order (drl)
       key_b:   [u64; W_KEY],   // the paired order (lex), present only for a pair ring
       raw:     [u64; W_RAW],
       divmask: u64,
   }
   impl Ring { pub fn new_pair(vars: &[&str], a: Order, b: Order) -> Result<Ring>; }
   ```

   One arena, one id space, two comparison keys. Cost: one extra `[u64; W_KEY]` per
   **distinct** monomial, for pair rings only.

---

## Consequences

- **The type-parameter question dissolves** rather than being traded off. Compare is faster
  than option (a) would be in practice (no branch at all, versus a monomorphized comparator
  that still executes the order's logic), while the order stays data.
- **Widen-and-restart on exponent overflow becomes possible without recompilation**
  (ADR-008). If the order were a type parameter, changing field width would still be fine —
  but the two decisions compound: with runtime order *and* runtime width, the `Ring` object
  is the single place a driver rebuilds, and everything above is untouched.
- **The consumer-facing seam is `&Ring` + `MonomialId`**, not a generic parameter that
  infects consumer types. This is constraint #1 at the multivariate layer: an adapter deals
  with two concrete things, neither of which is a type parameter it must thread through its
  own structs.
- **Heterogeneous storage works.** A `Vec<MPoly>` over rings with different orders is
  representable (each carries its ring handle), which matters for FGLM, for elimination
  with block orders, and for a debugger.
- **`Block` and `Matrix` orders come nearly free**, because the key normalization already
  handles arbitrary non-negative matrix orders. Elimination orders — which is what
  `Res_y`-free elimination via Gröbner needs — are block orders.
- **Cost: the key must be recomputed if the order changes**, i.e. re-interning into a new
  ring. That is correct for a one-off conversion of a *finished* basis.

  **It is not what FGLM does, and the original text said it was.** *Corrected 2026-07-31.*
  FGLM does not convert a basis by re-encoding monomials. It walks monomials in **lex**
  order while computing, for each, the normal form **modulo the drl basis** — which requires
  drl lead-term comparison and drl divisibility queries against the drl divisor index — and
  does linear algebra over the quotient basis. **Both orders are live on the same monomials,
  in the same loop, for the whole computation.**

  With the order baked into the key at intern time and the arena owned by the ring, the
  naive reading of the original text gives FGLM two `Ring`s, two arenas, two encodings of
  every monomial, a maintained id bijection, and two divisor indices — none of which was
  designed, while lane G7 was sized as an ordinary certificate lane. §Decision 6's
  **dual-key pair ring** is the fix: one arena, shared ids, one divisor index (built on
  `raw`, which is order-free per §Context), and no bijection to maintain. Divisibility being
  order-free is what makes the single index correct for both orders.

  The same correction applies to `groebner(_, Order::Lex)` (§Decision 5): it constructs a
  **pair ring** `(GrevLex, Lex)` and computes in it, rather than re-interning every input
  and intermediate into a second ring and mapping results back. Re-interning would be a real
  cost on the plan's own documented "correct" path for lex, and a second determinism surface
  (two id assignments) — though note that under ADR-008's content-derived ids the second
  assignment would at least be reproducible.

  **Lane G7 is re-sized accordingly** (ROADMAP §3): it delivers the dual-key arena *and* the
  FGLM linear algebra, and its brief names the two-orders-live-at-once property explicitly.
- **Cost: `c` must be chosen ≥ the degree bound `D` at ring construction**, and the
  complement fields consume the same guard-bit budget. Handled by ADR-008's widen-and-
  restart on underflow, which uses the identical detection.

---

## Alternatives considered and why rejected

**(a) Order as a type parameter.** Rejected on three counts, in increasing weight:
monomorphization across orders × rings × widths; impossibility of heterogeneous storage;
and — decisively — every consumer that reads an order from data (a file, a config, an SMT
input) has to write a dispatch shim, which pushes resolvent's type-level choice into the
consumer's API. That is the coupling constraint #1 exists to prevent. Retained *only* as
the opt-in `Ordered<O>` newtype for callers who want it.

**(b) Order as runtime data with a per-compare branch or function pointer.** Rejected. The
branch version is defensible (well-predicted) but still blocks inlining of the comparator
into `sort_unstable_by`; the fn-pointer version is measurably bad in a comparison sort.
Both give up a free win for nothing, since (c) exists.

**Storing raw exponents only and computing the key per comparison.** Rejected. It converts
the cheapest operation in the system into an arithmetic one, and comparison is the most
frequent monomial operation by call count.

**For FGLM: two rings with a maintained id bijection.** The honest alternative to §Decision
6, and rejected: it costs a `HashMap<MonomialId, MonomialId>` lookup on every cross-order
step in the hot loop, it doubles the arena memory, it needs two divisor indices, and the
bijection is a synchronization invariant that no certificate checks. The dual-key entry
costs one word per distinct monomial in pair rings only and deletes the invariant entirely.
An explicit `OrderPair` ring type with a maintained map remains the fallback **if** the
extra key word measures badly on `arena_stats()` for the largest M6 instance; it is
documented as FGLM-only if taken.

**For FGLM: re-interning the drl basis into a lex ring (the original text's implied
design).** Rejected because it does not express the algorithm — see §Consequences. It is
recorded here rather than deleted because an agent reading the original sentence would have
implemented it and discovered in week three that it cannot compute a normal form modulo the
drl basis.

**Supporting only grevlex.** Tempting — it is the only production order — and rejected
because elimination genuinely needs block orders, FGLM needs lex as a *target*, and
`Matrix` costs nothing once the key normalization exists.

**Making lex a first-class computation path.** Rejected on the literature: it is the classic
way to make a Gröbner implementation appear not to terminate. `groebner(_, Lex)` routes
through FGLM, and the doc comment says why.

---

## What would reverse this

- **A measured need for a non-matrix order** — e.g. a genuinely non-linear tie-break, or a
  local (negative-weight) order for standard-basis computation in a local ring. Local orders
  are *not* matrix orders with non-negative entries and are not supported by this scheme.
  Response: a separate `LocalRing` path with its own comparator; this ADR governs global
  orders and would be scoped, not reversed. That scoping should be written down if a
  consumer for singularity theory ever appears.
- **The arena-lookup cost dominating the key compare** (ADR-008's open microbenchmark).
  That would change where the key is *stored*, not whether the order is normalized into it.
