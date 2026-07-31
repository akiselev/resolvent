# CLAUDE.md — the working agreement for resolvent

This is an operating document, not orientation. It states what you must do, what you must
not do, and what a reviewer will check. Read `README.md` for what resolvent is; read this
before you write anything.

resolvent is built primarily by agents graded by oracles. That model only works if every
unit of work carries its own verdict function and if nobody quietly widens a decision that
everything above it inherits. Both properties are fragile and both are enforced here.

---

## 0. Document precedence

Several documents claim authority over overlapping ground. The order is:

1. **An ADR that carries an `**Amended:**` line citing a critique finding.** That is the
   corrected record and it wins over both the critique and the plan it amends.
2. **The critiques** — `docs/research/critique-engineering.md` and
   `docs/research/critique-plan.md` — win over any document that has *not* yet been amended
   against them. They were written against the plans and they are specific. §3 below lists
   the ones that will bite you first, with their current state.
3. **`API.md`** for anything in a public signature. It is canonical and supersedes
   `plans/api-shape.md`.
4. **`docs/decisions/ADR-NNN`** for internal architecture and every one-way door.
5. **`plans/architecture.md`**, then `plans/verification.md`, then `plans/roadmap.md`.

`plans/api-shape.md` is historical. `docs/decisions/RECONCILIATION.md` is referenced by
`API.md` and by ADR-019 and **does not exist** at the time of writing; if you resolve a
conflict between the architecture track and the consumer track, record it there.

The ADR set is being amended in flight. **Re-read the header of any ADR you are about to
build against**, and prefer citing an ADR by its header fields (`Status`, `Reversibility`,
`Amended`, `Gates lanes`) over quoting a body you read an hour ago.

If two documents at the same level disagree, that is a defect. Stop, record it, and get it
settled by an ADR. Do not pick one and proceed — that is how the workspace ended up with
two normative API specifications describing different libraries.

---

## 1. Prime directive: nothing merges without its certificate green

An operation is not implemented until its certificate is implemented and asserted **in the
same test that exercises the operation**. `plans/verification.md` §2 is the catalogue; §7.5
is the per-lane checklist a reviewer runs. What "its certificate" means, per operation:

| Operation | Certificate that must be green |
|---|---|
| `Integer`/`Rational` arithmetic | Inverse-op round-trip; differential against the `rug` dev-oracle; generators targeting word boundaries, where carry bugs cluster |
| `Fp` arithmetic | **Exhaustive** over all `(a,b)` for every prime `p < 2^10` against an `i128` reference; random for `p < 2^63`; `a·a⁻¹ == 1` for every unit |
| Bulk / batched GF(p) | Componentwise equality with the certified scalar path, including tails and misaligned lengths. Note this is complete for *arithmetic* and silent on control flow — see §3.1, *batched lane faults* |
| Prime generation | Cross-checked against an independent segmented sieve over a committed window. A composite in the registry is undetectable by every downstream certificate |
| CRT combine | `result ≡ r_i (mod p_i)` for every `i`; result in the symmetric range; **moduli asserted pairwise distinct**; `M = Π p_i` asserted ≥ the bound the caller sized against |
| Rational reconstruction | `n ≡ d·a (mod M)`, `gcd(n,d) == 1`, `|n|,|d| ≤ √(M/2)` |
| Monomial encode/decode/multiply | Round-trip at *and past* the capacity boundary; `decode(a⊗b) == decode(a)+decode(b)` **or** the multiply returned `Err` — never a third outcome |
| `UPoly` / `MPoly` arithmetic | `(a·b)/b == a`; degree additivity; agreement with a naive `O(n²)` reference; evaluation homomorphism at points drawn from the **fleet seed schedule**, not the default seed |
| gcd | `H \| A`, `H \| B`, **and a Bézout witness `u·A + v·B == H`**. Over ℤ[x] the modular gcd returns its GF(p) cofactors so the degree half is itself certified |
| Square-free (Yun) | `Π f_i^i == f`; factors pairwise coprime; each square-free |
| Resultant | Cofactors `u·f + v·g == Res`; the degree bound `deg_x Res_y ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)`; `Res == 0 ⇔ deg gcd > 0`; agreement of the three independent routes |
| Subresultant chain | Specialization property at random good primes and evaluation points; valid degree sequence |
| Root isolation | Exact Sturm count per interval; pairwise disjoint and ordered; `f(lo) ≠ 0 ≠ f(hi)`; Descartes variation exactly 1; all inside the Cauchy bound; round-trip from constructed roots |
| Factorization over GF(p) | Multiply back **and** the complete irreducibility test per factor |
| Factorization over ℤ | Multiply back; modular irreducibility certificate where one exists, with its **rate tracked**; factors pairwise non-associate after normalization; the Landau–Mignotte bound validated against the known-factorization generator |
| Gröbner, certified | Every `f ∈ F` reduces to zero mod `G`; **all** `C(\|G\|,2)` S-pairs reduce to zero *without consulting any pair-elimination criterion*; stored cofactors `g_j = Σ h_ij f_i` check by multiplication |
| FGLM | Two-way reduction, **plus** the lex output satisfies Buchberger's criterion in the lex order, **plus** the lex staircase has exactly `dim_ℚ ℚ[x]/I` standard monomials |
| `AlgebraicReal` | The eleven properties in `plans/verification.md` §2.6, under an explicit step budget, with "did not finish" graded as **wrong** |
| L4 hash-consing / `diff` | Injectivity; on the polynomial subset `diff` equals `UPoly::derivative` exactly; canonical bytes byte-identical across insertion orders, thread counts, processes, and feature combinations |

Four rules govern certificates themselves. They are not optional and they are what separate
a self-certifying library from one that says it is:

- **A certificate may not invoke the operation it certifies, nor any routine on that
  operation's call graph.** `fn gcd(_,_) -> Integer { ONE }` satisfies "`g|a`, `g|b`,
  `gcd(a/g,b/g)==1`" when the coprimality check calls the same `gcd`. If a certificate
  cannot avoid the call graph, the row is an **invariant**, not a certificate, and must be
  labelled as one.
- **Every certificate ships with a mutant set.** At least one deliberately wrong
  implementation, committed under `#[cfg(test)]` in the same module, with a test asserting
  the certificate **rejects** it. The mutant must compile and produce a plausible wrong
  value; rejection by the type system does not count. Pick mutants from the failure family
  the operation actually has: coarsening, refining, off-by-one in a bound, identity, trivial
  constant (`1` / `0` / `Unknown` / `Probable` / `Decline`), sign flip, silent wrap. A gate
  that has never been observed to fail is not known to work — this is the rule the license
  gate already follows and it applies to all fifty certificates, not one.
- **Every "don't know" or "probably" outcome carries a tracked rate with a committed
  ceiling.** `Unknown` always, `Probable` always, and decline-always each pass every
  soundness certificate in the plan and are useless. The rate is measured in the PR that
  lands the API, committed to `sharpness-ceilings.toml`, and gated in CI. Lowering a ceiling
  is free; raising one requires a recorded justification and is counted in CI output. `TBD`
  is not a ceiling.
- **A randomized certificate is only a certificate across the fleet seed schedule.** At the
  single fixed default seed it is a golden test at one point, and it is graded as an
  invariant. The number of distinct seeds a randomized certificate was checked at is
  reported alongside the score.

**Self-certification is the primary gate; external oracles are secondary.** A
self-certificate failure is a bug in resolvent with certainty. An oracle disagreement may be
a normalization difference and needs triage. A lane brief that treats them as equivalent
signals generates triage work that looks like bugs. The one documented inversion is the fast
Gröbner mode, whose primary verdict is external differential testing — see §3.2.

**Build the oracle side first, every time.** Sturm exists to grade Descartes; Buchberger
exists to grade F4; Zassenhaus exists to grade van Hoeij; the naive `O(n²)` multiply exists
to grade the fast one. None of them will ever be the production algorithm. A
performance-graded lane's CI job **does not exist** until its oracle lane is green and
frozen. An oracle module's permitted-import set is committed and CI-enforced: the day
someone "fixes" Sturm's coefficient growth by routing it through the Ducos PRS that Sturm is
supposed to grade, the strongest certificate in Layer 2 silently becomes a check of a
component against itself and no test changes colour.

---

## 2. The frozen layer

Layer 0 and Layer 1 representation decisions are one-way doors: everything above inherits
them, and reversing one is a rewrite, not a refactor. They are settled in
`docs/decisions/`. The ones marked **one-way** in their own header:

| ADR | Decision |
|---|---|
| 001 | MIT OR Apache-2.0; Tier A/B/C reading discipline; mechanical `cargo-deny` gate |
| 004 | Coefficients are **ℤ-primitive**; ℚ is a boundary façade. No inner loop calls a rational gcd. Root isolation works in ℤ[x] on **dyadic** intervals |
| 006 | Generics may cross a crate boundary; they may not cross into an inner loop. At most one runtime `match` per *call*, never per element. `Ord` is not required on `Ring`. `LANES`/`Scalar` stay on the base trait so batched multi-modular remains reachable |
| 007 | Three representations. **`UPoly<C>` is defined first and standalone** and knows nothing about monomials, orders, or `Ring` |
| 008 | Interned arena + order-normalized packed key + `divmask`; guard-bit overflow detection; widen-and-restart. The door is the **interning/id structure**, not the field width |
| 009 | The monomial order is runtime `Ring` data, normalized into the comparison key at intern time. Comparison is an order-free unsigned word compare. grevlex is the production order; lex is reached by FGLM, never computed directly |
| 010 | Modular methods everywhere; certainty in the return type; `Proved` is the default path |
| 011 | Fail at construction, not at query; no panics; structured `Unsupported`; step budgets in steps, never wall-clock |
| 012 | Counter-based seeded RNG; index-addressed primes; ordered combination; replayable traces |
| 013 | `AlgebraicReal` is `Arc<Inner>`, `&self` monotone refinement, `Send + Sync`, total `Ord` |
| 014 | No `Hash` without an explicit factorization-backed canonical form; no general arithmetic; multiplicity is not a field of the number; **`SqrtExt` stays first-class and is never subsumed** |
| 019 | One open trait tower in `resolvent-base`. No second ops-surface scalar trait, no seam crate |
| 020 | Every arena is a caller-owned value; handles are arena-relative and never reach a result |

**If you want to change one of these, write a new ADR that supersedes the old one.** Copy
the file's structure: Context, Decision, Consequences, Alternatives considered and why
rejected, What would reverse this. Set the old ADR's status to
`Superseded by ADR-NNN`. Do not edit a ratified decision in place, and do not diverge
silently in code — a divergence between an ADR and an implementation is a defect that will
be caught late, by a lane three trunks away that inherited the ADR's version.

**Ratification is a header field, not a vibe.** Every ADR carries:

```
**Status:** Ratified <date>        — or — Proposed (<date>)
**Reversibility:** one-way | costly | cheap
**Amended:** <date> — what changed, and which critique finding drove it
**Gates lanes:** <lane ids>
```

Two rules follow, and they are what makes the freeze an edge rather than an intention:

- **A lane may not start against a `Proposed` ADR.** "The file exists" is not ratification.
  The `Gates lanes:` field is the manifest CI reads; if your lane appears there and the
  status is `Proposed`, your CI job does not exist yet.
- **Read the `Amended:` line before you read the body.** An ADR body that predates its
  amendment is the version the critiques attacked. §3 lists the corrections that matter most
  and their current state.

---

## 3. The corrections that bite first

The critiques found defects in specifications that everything above inherits. Some have
since landed as ADR amendments; some have not. **The state below is a snapshot — confirm it
from the ADR's own `Status:` and `Amended:` fields before you build against it.** An agent
following an un-amended document literally will write code that must be thrown away.

### 3.1 Corrected — build against the amendment, not the body it replaced

**`Ring::zero()` / `Ring::one()` were unimplementable** (ADR-006, amended). They were
declared as receiverless associated functions in `API.md` §3.2, ADR-006 and
`plans/architecture.md`:222-229. Of the closed instantiation set — `Fp`, `Fp4`, `Integer`,
`Rational`, `Zn`, `GFpk`, `NumberFieldElem` — only `Integer` and `Rational` have a static
zero. `Fp` carries `p` and its Barrett reciprocal *by value*, so `Fp::zero()` cannot answer
"zero of which prime field", and `API.md:1188` already writes `vec![fp.zero(); n]` —
ring-object arithmetic, the exact shape ADR-006 forbids. The corrected trait adds
`type Ctx` with `fn zero(ctx: &Self::Ctx)`, `fn one(ctx: &Self::Ctx)` and
`fn ctx(&self) -> &Self::Ctx`. Element-to-element arithmetic is untouched, so **no context
enters an inner loop**; only construction consults one, which is per-call and is exactly
ADR-006's own boundary rule. `UPoly<C>` therefore stores one `C::Ctx` alongside its
coefficients — which it needs anyway, since a `UPoly<Fp>` that does not know its own `p`
cannot be printed, serialized, or compared. **Typecheck the trait block with real impls for
`Fp` and `Integer` before writing anything above Layer 0.**

**`Reducible::Image: Field` was false over algebraic extensions** (ADR-006, amended). For
ℚ(α), reduction mod `p` lands in `GF(p)[x]/(f mod p)`, a field only when `p` is inert. For
the multiquadratic towers geometry produces — ℚ(√2,√3), Galois group `(ℤ/2)²`, no 4-cycle —
**no prime is inert**, so the trait had no valid implementation at all. This is the same
Chebotarev obstruction the plan already documents for Swinnerton–Dyer factorization
certificates; nobody connected it to the trait bound. Corrected to
`type Image: CommutativeRing` with `fn reduce(&self, m: &Modulus) -> Result<_, BadPrime>`,
and `Liftable: Reducible`. The consequence has to be respected downstream: the modular path
over algebraic extensions is **multi-modular over split factors** — factor `f mod p`, work
in each `GF(p^{d_i})`, CRT back — a different algorithm with its own bad-prime predicate. It
is a **lane**, not "an added instantiation". ℚ(√2,√3) belongs in that lane's corpus.

**Divisibility is order-free and is an inner loop** (ADR-009, amended). The original text
listed "an order-specific divisibility direction" as one of three O(1) sites "all outside
sort inner loops". Divisibility *is* the inner loop of symbolic preprocessing and reducer
selection — the reason the divisor-query index is worth 10–20×. Compute divisibility, lcm,
gcd, and degree from `raw`, the raw packed exponents, which are order-free. Order-specific
work is two places: encode, and the constant subtract on multiply. `key` and `raw` need
different word counts (grlex needs `n+1` key fields against `n` raw fields), hence separate
`W_KEY` and `W_RAW`.

**Monomial ids are content-derived, not first-encounter-ordered** (ADR-008, amended). An
interner is a shared mutable accumulator, which the determinism contract bans, and symbolic
preprocessing is nothing but interning. `terms.par_iter().map(|t| ring.intern(t)).collect()`
*looks* like the permitted ordered-combination shape; the collection is ordered, the ids are
not. Content-derived ids — a deterministic hash of the packed key with a fixed
collision-resolution order — make parallel interning deterministic. Regardless: **no
tie-break anywhere may consult `MonomialId` ordering.** Tie-break on the key, which is
content-derived and totally ordered.

**Batched lane faults are expressible** (ADR-006, amended). `Field::inv(&self) ->
Option<Self>` cannot say *which* lane of a batched tuple ring was non-invertible, so the
batch could not be split. `BatchField::inv_batch(&self) -> Result<Self, LaneMask>` fixes the
arithmetic half. The control-flow half is not fixed by a trait and must be in the lane
brief: under batching all `N` primes share one matrix construction and one pair-selection
path, so a prime whose lead-monomial set diverges corrupts shared control flow instead of
producing a minority to discard. **Brief the lane as "batching *and* splitting"**, with the
offending prime index recorded in the `Trace`. Componentwise equality against `N` scalar
runs is complete for arithmetic and silent on both failures.

**`BulkOps` is deleted** (ADR-006, amended). Bulk kernels are free functions in
`resolvent-modular` over concrete types, selected by one `match` on the `RingTag` at the top
of each phase. A generic caller over an arbitrary `C` gets the naive loop and the doc comment
says so. Re-exposing a Tier-M kernel as a trait method either duplicates it across the
instantiation set — the thing Tier M exists to prevent — or is a forwarder that buys nothing
but a bound, and an agent reads it as license to add `fn row_reduce(..)` next to it.

**Published crates have zero dev-dependencies** (ADR-002 / ADR-005 gate L6a, amended). The
`rug` differential oracle lives in a `publish = false` crate and tests only the *public*
surface of `resolvent-int` — which is sufficient by design, because the newtype wall makes
the public surface the whole point. CI asserts every `publish = true` crate's
`[dev-dependencies]` table is empty. Otherwise `cargo publish` records an LGPL-3.0+
dev-dependency that `cargo deny` (scoped to the published graph minus dev-only features)
will not catch, and downstream `cargo test` builds GMP.

**`MonomialId` exhaustion has an error path** (ADR-008, amended). `MonomialId(u32)` caps the
arena at 2³² distinct monomials; route id allocation through
`Unsupported::MonomialArenaFull { capacity }` rather than an index panic, and keep a memory
model — `Ring::arena_stats()` plus a corpus assertion on the largest instance — rather than
asserting by omission that a monotonic arena is fine.

### 3.2 Not yet corrected — do not start these lanes

**The two Gröbner modes do not share a reducer.** ADR-010 §5 promises they do and calls that
the fast mode's only internal oracle. They cannot: the fast reducer is a `u32` GF(p) kernel
(`plans/architecture.md`:188) and the certified mode reduces over ℚ, because a cofactor
identity mod `p` certifies nothing over ℚ. What they actually share is matrix construction,
symbolic preprocessing, the monomial layer, and the row format — **not the reducer**. Two
consequences the ADR must absorb before the Gröbner trunk starts: the sparse GF(p)
row-reduction lane needs its own internal oracle (a naive dense `u32` Gaussian elimination
over the same `FpParams`, in the same crate) with external differential testing promoted to
its primary verdict; and the cofactor prototype must measure **primes and wall time to
reconstruct the cofactor system over ℚ**, not a GF(p) memory multiplier — cofactor
coefficients are larger than basis coefficients, so reconstruction sets the prime count, and
there are `|F|×|G|` of them.

**`Ord` on `AlgebraicReal` is unbounded on the default path.** The separation-bound argument
makes comparison terminating, not attainable: for the degree-~200, ~500-bit resultants the
plan predicts, the Davenport–Mahler bound is tens of thousands of bits of refinement.
`Ord::cmp` has no `Result`, no budget, and no way out, and it is what `sort()`, `BTreeMap`,
`binary_search` and `max()` all call. Measure the actual step distribution on the
elimination corpus and publish it, then either keep `Ord` with a diagnostic ceiling far
below the theoretical bound plus a documented, benchmarked
`try_cmp(&self, &Self, Budget) -> Result<Ordering, Decline>` that latency-path consumers are
directed to, or carry a construction-time step ceiling on the value. **Do not leave this
unsettled past the algebraic-number milestone**; it is in every signature and it is the
consumer's most-called function.

**Shared refinement makes declines schedule-dependent.** Cloning an `AlgebraicReal` shares
refinement progress, so the step count of a given `cmp` depends on what has already been
compared and, under `parallel`, on what other threads did. State and test the invariant:
*the refinement cache may change how much work a call does, never what it returns, including
whether it declines.* That forces `AlgebraicReal` budgets to be derived from the separation
bound, always. Relatedly, **certificates, `Evidence`, `Telemetry` and `BudgetTick` are
excluded from canonical bytes** — only the mathematical value is serialized — or the
tuning-matrix byte-identity gate fails on its first run, because the batch width is a tuning
knob that changes `primes_used`.

**Three blocking experiments require the artifact they gate.** The monomial term-type
microbenchmark wants "a realistic S-pair queue workload" (a working Buchberger/F4, which it
gates); the cofactor prototype wants Katsura-8 (an engine it gates); the `AlgebraicReal`
mutability benchmark wants a working `AlgebraicReal` (the lane it gates). Each must be
respecified against a synthetic harness before it can run: a recorded S-pair operation trace
from a throwaway 200-line Buchberger over GF(p) with `Vec<u32>` exponents on
Katsura-6/Cyclic-6; cofactors measured on Buchberger-with-cofactors at Katsura-6/7 over ℚ
with the multiplier reported as a function of instance size; and ~300 lines of `cmp`,
`refine` and polynomial sign evaluation over the univariate milestone's `UPoly<Integer>`,
with roots built as `Π(x−rᵢ)`. ADR-008 has already been respecified this way; the other two
have not.

**If you fix one of these, fix the ADR in the same commit.** A correction that lives only in
a critique document will be re-broken by the next agent reading the ADR.

## 4. Fail-closed discipline

**Fail at construction, not at query.** Every invariant is checked when a value is built —
square-freeness, isolation, non-vanishing at interval endpoints, ring compatibility, degree
and variable bounds, exponent range. Construction returns `Result`. Every method on a
well-formed value that is mathematically total is total in the type system too. This is what
lets `sign_of` return a bare `Sign` and `AlgebraicReal` implement a real `Ord` while the
no-panic rule stays absolute (ADR-011).

Mechanically:

- **No `unwrap`, `expect`, `panic!`, slice indexing, or unchecked arithmetic in any
  published crate outside `#[cfg(test)]`.** Every published crate denies
  `clippy::{unwrap_used, expect_used, panic, indexing_slicing, arithmetic_side_effects}`
  and CI runs `cargo clippy -- -D warnings`. `debug_assert!` is encouraged; it compiles out.
- **No panics as control flow, and no panics at all.** A violated internal invariant returns
  `Error::Internal { invariant: &'static str }`. An embedding kernel may sit behind an
  `extern "C"` boundary where unwinding is UB, and a caller under `panic = "abort"` cannot
  recover. More fundamentally: to a user of an exact kernel a panic and a hang are the same
  event — an operation that produced no answer. Allocation failure keeps Rust's default
  (abort); resolvent does not pretend to handle OOM and says so in the crate docs.
- **`Unsupported` is a structured value, never a string.** A consumer's fail-closed path
  matches on variants; a string forces string-matching and breaks silently on rewording. Add
  a variant rather than a message.
- **No silent approximation, ever.** A cheap rung that cannot decide returns
  `Verdict::Unknown` and the caller climbs to the exact rung. A bare `Sign` is returned iff
  the function is total and exact; anything that can be indeterminate returns
  `Verdict<Sign>` and never `Sign`. `Verdict` is produced only by enclosure and filter APIs
  and never by an algebraic-decision API.
- **No `tolerance`, `epsilon`, `atol`, `rtol`, `snap`, or "close enough" parameter, at any
  layer, under any name.** A grep gate enforces it. Equality by tolerance is *intransitive*
  — `α = β`, `β = γ`, `α ≠ γ` — a sort then produces garbage, and a geometry consumer
  produces a topologically inconsistent arrangement. `refine_to(width)` is not a tolerance:
  it never affects a verdict, and that is property-tested as idempotence under refinement.
- **No "probably correct" mode anywhere that is not visible in the type.** A heuristic
  result is `Certified<T>` carrying `Certainty::Probable(reason)` with its evidence. The
  default path returns `Proved`. A caller who wants speed asks for it by name. Certificates
  have private fields, no public mint, public read accessors, and a `certifies(claim)`
  tether to the claim they attest.
- **Budgets are counted in steps, never wall-clock**, because wall-clock is
  nondeterministic. Where a proven bound exists (Mignotte–Davenport, Landau–Mignotte,
  Hadamard, Cauchy) the budget is derived from the bound, exhaustion is proven impossible,
  and the budget is a **bug detector**: exceeding it is a `debug_assert!` in debug and a
  diagnostics counter in release, and the loop continues because it is still correct. Where
  no proven bound exists (van Hoeij lattice iteration, stabilization-driven reconstruction)
  the budget **is** the exit and exhaustion returns a typed decline carrying resumable
  state. Which regime applies is stated per entry point.
- **Never wrap, never saturate.** Exponent overflow returns `Err` and the driver widens and
  restarts. Wraparound in a packed exponent field yields a *correct Gröbner basis of a
  different ideal*, and every other certificate in the library passes on it. Guard-bit
  detection is compiled into release builds, not behind `debug_assert`.

---

## 5. Determinism

Same input ⇒ same output, bit-for-bit, on any machine, at any thread count, in any build
profile — and the *path* taken is recorded and replayable. This is a verification constraint
before it is a consumer one: **a non-deterministic library cannot have a regression corpus.**
Every golden file, every minimized counterexample, and every change-point baseline assumes
it (ADR-012).

**Forbidden:**

- `rand` as a dependency of any published crate. `SystemTime`, `Instant` in any decision
  path, `std::process::id`, address-derived values, and `HashMap`'s default `RandomState`
  are denied by lint. There is exactly one RNG type in the workspace.
- "Pick a random prime." Ever.
- A sequential RNG threaded through parallel work — draw order then depends on chunking and
  scheduling, so 8 threads differ from 1.
- Shared mutable accumulators updated from `for_each`. Combination is in **index order**,
  never completion order.
- `HashMap` iteration order reaching any return value or any decision. There is no such
  thing as an "internal" choice in an algorithm whose output depends on search order, and
  F4's pair selection, symbolic preprocessing, and tracer all do.
- Floating point in any decision path. Address-derived lock ordering. Wall-clock-adaptive
  strategy switching ("this is taking too long, try the other algorithm").
- Nondeterministic parallel reduction of any kind.

**What replaces each:**

- **Counter-based RNG** (`output = F(key, counter)`, Philox/ChaCha-shaped). A `Session`
  carries a `Seed`; a worker at logical index `k` uses `rng.substream(k)`, so the value drawn
  at a given logical position is a function of that *position*, not of scheduling, thread
  count, or chunk size. The default seed is a fixed checked-in constant, so a caller who does
  nothing gets a reproducible run.
- **Index-addressed primes.** `prime(i)` is a pure function of `i` over a checked-in
  generator. A modular run consumes primes in index order; a rejected prime is recorded **by
  index with its rejection reason**. Evaluation points come from the seeded counter RNG at
  index-derived positions and are recorded identically.
- **Fixed-seed hashing** (`rustc-hash` is seedless by construction), `BTreeMap` in every
  ordering-visible position, and any table iterated to produce output sorted by a declared
  total order first.
- **`par_iter().map(..).collect::<Vec<_>>()` plus reductions over the ordered `Vec`** as the
  only permitted parallel shape. Work-splitting granularity may change timing and must not
  change values.
- **Adaptive switching on step counts**, which are deterministic, rather than on time.
- **Recorded, replayable traces.** `op_with_trace(input) -> (Certified<T>, Trace)` pairs with
  `op_replay(input, &Trace) -> Certified<T>` and CI asserts the replay is byte-identical. A
  bug report is `(input, trace)` and nothing else.
- **Tuning thresholds are inputs, not scattered constants.** One `Tuning` struct with
  documented defaults. Same input + same `Tuning` ⇒ same output; different `Tuning` ⇒ same
  *values*, different timing. CI asserts value-equality across a `Tuning` matrix, which
  doubles as a free naive-vs-fast agreement oracle.

CI asserts all of this three ways: the thread-count matrix (`RAYON_NUM_THREADS ∈ {1,2,8}`,
in-process and cross-process, across feature combinations, canonical bytes compared), the
tuning matrix, and replay equality. Without those gates this section is a comment. If the
per-commit gate ever gets too slow, tier the corpus — `fast` at 90 s with a printed census,
`full` at PR time with the complete determinism matrix, `slow` nightly — and do **not** cut
the determinism matrix. It is the most expensive gate and the least often red, which is
exactly why it is the one that gets sacrificed first and must not be.

`rayon` is behind a default-off `parallel` feature, appears only in `-algebra` and `-real`,
and appears in no public signature.

---

## 6. License and provenance discipline

resolvent's best algorithmic references are copyleft and resolvent's output must be
MIT OR Apache-2.0. The framing is adopted from
`/home/dev/projects/arrangements/DESIGN.md` §1 rather than reinvented:

> MIT OR Apache-2.0. **Independent reimplementation informed by architectural study of the
> GPL/LGPL sources** — not "clean-room"; that term means the authors never saw the original,
> and we do read Singular, FLINT, PARI, and msolve at the level needed to understand *what*
> they do. Algorithms and ideas are not copyrightable, and the published literature covers
> the substance. Process discipline: write Rust with the literature notes open, **not** the
> reference source tree; no copied constants, comments, or identifier structure; review
> diffs against the notes, not the sources.

**Tier A — freely readable, freely cited.** Refereed literature and textbooks: Faugère
(F4), van Hoeij (lattice recombination), Zassenhaus, Collins/Brown (subresultant PRS),
Ducos, Rouillier–Zimmermann and Sagraloff–Mehlhorn (real root isolation), von zur Gathen &
Gerhard, Cohen, Geddes/Czapor/Labahn. Also the *user-facing documentation and manuals* of
any system — documentation describes behaviour, and matching documented behaviour is a
compatibility goal, not a derivation. Also permissively licensed Rust: `feanor-math` (MIT),
`dashu`, `ark-ff`. Still do not copy verbatim: MIT carries an attribution obligation and a
copied block would need its notice carried, which defeats the purpose.

**Tier B — readable for *understanding*, never for *transcription*.** Singular, FLINT,
PARI, msolve, CoCoALib, Macaulay2, Sage, Groebner.jl (GPL-2.0), GroebnerWalk.jl (GPL-3.0);
and SymPy (BSD, so no hazard, but the same discipline for consistency).

*Permitted:* reading to understand which algorithm variant is used, why a step exists, what
edge case a guard protects, what the overall pipeline is.

*Forbidden without exception:* copying code, comments, identifier names, file or module
structure, or **magic constants and tuning thresholds**. Thresholds are the likeliest
accidental transcription and the least defensible — "switch to Karatsuba at 32 limbs" is
someone else's measurement on someone else's machine, and it is *wrong for ours*. Every
threshold in resolvent is re-derived by measurement on resolvent's own corpus and the
measurement is checked in. This is simultaneously a licensing rule and a correctness rule,
which is why it holds under pressure.

*Procedure:* read → write a note in `docs/research/` **in your own words** → **close the
source** → implement from the note.

**Tier C — do not read at all.** Symbolica: its license grants no copying right and
conditions source-availability, so it is a *stricter* hazard than GPL, and there is no
algorithmic content in it that is not in the published literature. Any commercial CAS source
(Magma, Maple, Mathematica internals). **Any repository with no declared license** — no
license means all rights reserved, which is stricter than GPL, not looser. This tiering is
stated explicitly because "source-available" sounds safer than "GPL" and agents reliably
infer it backwards.

**How to record what informed a design decision.** Every non-obvious algorithm module
carries a `Derivation:` line in its module doc-comment citing **both** the paper **and** a
path into `docs/research/`:

```rust
//! Derivation: van Hoeij, J. Symbolic Comput. 33(5):425-445, 2002, §3;
//! see docs/research/notes-van-hoeij-recombination.md §2.
```

CI resolves the path, fails if the note does not exist, and fails if the note lacks a
`Sources:` block tagging each reference with its tier. A note may serve many modules; a
module may not exist without one. A citation alone is not enough — a `Derivation:` line
pointing at a paper is satisfiable by pasting a reference the author never opened, which is
exactly what an agent working from a source tree would do. The note is the artifact that
proves the discipline was followed, and it is not reconstructible after the fact.

**Provenance cannot wait; paperwork can.** Deferring attribution files, SPDX headers, and
legal review to release prep is correct. Deferring the `Derivation:` → note tether, the
per-lane record of which Tier-B sources were consulted, the `cargo-deny` gate with its three
planted rejection cases, and a Tier-A citation for every benchmark family is not — those are
unreconstructible later. In particular: Eco-*n*, Noon-*n*, and Reimer-*n* have no published
invariant, and "pin them to a specific generator source" in practice means a Singular `.lib`,
an msolve test directory, or a Groebner.jl benchmark file, all GPL-2.0. Transcribe the
system from its **paper**, or drop the family.

Mechanical gates, all on day one: `cargo-deny` over the *published* graph with an explicit
allow-list and every copyleft SPDX id denied; a regression corpus the gate must **fail** on
(`malachite`, `polynomen`, and a synthetic Apache-only crate depending on `rug`);
`cargo-about` with a stale attribution file failing CI. The workspace has exactly two crate
categories — `publish = true`, gated; and `publish = false`, which may carry LGPL
dev-dependencies and shell out to GPL binaries. There is no third category and no per-crate
exception process. Exception processes are how an Apache-2.0 crate ends up shipping with
mandatory LGPL dependencies.

Every external oracle is a **subprocess**. Nothing links.

---

## 7. Verifying honestly

- **Re-run the actual gate after your last edit.** Not before it, not a similar one.
- **`cargo build` does not compile test targets.** `cargo build --workspace --all-targets`
  does. A green build is not evidence about tests, and a green result from before a
  signature change is not evidence about the code after it.
- **`cargo check` is not `cargo build` is not `cargo test`.** Say which you ran.
- **Never report a pass you did not just observe.** If you did not run it, say you did not
  run it. If it failed, report the failure — a report of "tests pass" that turns out to mean
  "they passed an hour ago, before the last three edits" costs more than the failure would
  have.
- **A skipped oracle is a counted `SKIP`, never a pass.** The harness prints a skip census;
  a CI job declares which oracle tier it requires and **fails** if an oracle in that tier is
  absent. Silently green because nothing was installed is the failure mode this rule exists
  to prevent. (`gp` on this machine is a shell alias for `git push`; detect PARI by invoking
  `gp -q -f` on a known expression and checking the output, not by probing `$PATH`.)
- **A budget-exhausted outcome is classified before it is scored.** It is a failure if the
  instance is in the must-complete sub-corpus, or if the operation's budget was derived from
  a proven bound — in which case exhaustion is impossible for a correct implementation and
  the decline is a bug. Otherwise it is a survived instance counted in the decline rate.
  Raising a budget default is a diff, is counted in CI output, and requires a recorded
  justification: the cheapest way to make "any decline fails" go away is to raise budgets
  until declines become long runs, and a long run in CI is the hang that is the deadliest
  failure mode here.
- **Performance numbers need a pinned machine, medians of `k` runs with the IQR, and the
  recorded commit, generation, machine id, compiler version, feature flags, thread count,
  and certification mode.** A single sample is not a measurement. A compiler bump is a
  re-baseline event, not a regression.
- **Do not invent a benchmark number, an adoption claim, or a citation.** If you do not know
  something, write what would settle it. Every threshold in these documents is traceable to
  a cited figure or is marked TBD and set by measurement before it becomes a gate; keep it
  that way.

---

## 8. Git

**Never use destructive git to undo your own edit.** `git checkout <file>`, `git restore`,
`git reset --hard`, and `git clean` discard *every* uncommitted change in the target,
including work that exists in no git object and that `git fsck` cannot recover. This has
already destroyed an in-progress feature in this workspace.

- **Undo your own edit by making the inverse edit.** You know what you changed; reverse
  exactly that with an edit.
- **Run `git status --porcelain` before editing.** Treat every already-dirty file as holding
  work you cannot see and must not discard.
- **Need a scratch copy of a dirty file?** `cp` it somewhere first. Never let git arbitrate
  the rollback.
- **Safe git verbs are read-only:** `status`, `diff`, `show`, `log`, `fsck`.
- Destructive git on a dirty tree requires the user to ask for it explicitly.

**Commits.** Conventional prefixes, scoped to the crate or lane:

```
feat(modular): Fp Barrett reduction with exhaustive small-p certificate
fix(poly): guard-bit check was after the constant subtract, not before
adr(009): correct the divisibility claim; divisibility is order-free on raw
docs(verification): add the mutant-set requirement as checklist item 0
bench(int): gcd_ext ladder to 256k bits against the rug oracle
```

One decision per commit. A correction to a plan document and the code it governs belong in
the same commit; a correction that lands only in a critique will be re-broken by the next
agent reading the ADR. Commit or push only when asked, and branch first if you are on the
default branch.

**Where documents go.**

| Kind | Location |
|---|---|
| A decision, especially a one-way door | `docs/decisions/ADR-NNN-kebab-title.md`, numbered sequentially, never renumbered |
| Superseding a decision | A **new** ADR; set the old one's status to `Superseded by ADR-NNN` |
| A Tier-B reading note (the artifact the `Derivation:` line points at) | `docs/research/notes-<topic>.md`, with a `Sources:` block tagging each reference by tier |
| A measurement that gates a decision | `docs/research/` as a machine-readable table plus a prose note, with the decision line stated *before* the run |
| A schedule, a lane brief, a milestone | `plans/` |
| The public surface | `API.md` — canonical, edited in place, with the change argued against the invariants in §9 |
| An adversarial review | `docs/research/critique-*.md` or `challenge-*.md` |

House style throughout: state the load-bearing choice first, then the alternatives rejected
and why. Cite real file paths and line numbers when you claim something about existing code.
Prefer "fail closed" over "best effort". No marketing voice. If you do not know something,
write what would settle it instead of guessing.
