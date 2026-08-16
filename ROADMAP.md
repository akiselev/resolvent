# resolvent — roadmap and agent fan-out plan

**Status:** canonical. **Supersedes `plans/roadmap.md`**, which remains readable as the
working notes it was.
**Normative inputs.** `docs/decisions/ADR-001…025` (ADRs win on every decision they touch —
ADR-021 §1); `API.md` (canonical for consumer-facing shape). `plans/verification.md` defines
the verdict functions referenced here and is non-normative wherever an ADR speaks;
`plans/architecture.md` likewise.

This document answers one question: **what can be worked in parallel by separate agents, and
what cannot.** Milestones, exit gates and effort estimates exist here in service of that
answer. Where an exit criterion was previously a judgement, it has been rewritten as a
condition a CI job evaluates.

---

## 0. How to read this

### Lane grades

| Grade | Definition | Convergence | Fan-out |
|---|---|---|---|
| **`certificate-graded`** | The primary verdict is CERT / INV / PROP: a green gate means done, and a red gate means a bug with certainty. | Days. Monotone. | **Fan out aggressively.** |
| **`score-graded`** | The success criterion is *a number to optimize*, not a certificate to check — wall time, memory, instance ceiling, Unknown rate, `Proved` rate, primes needed. | **Months. Non-monotone. No completion condition.** | **Do not fan out.** Two agents on one score lane each optimize against a baseline the other is moving. One agent, one frozen baseline, a change-point-tracked series. |
| **`measurement`** | A score lane whose deliverable is a *committed number*, not an improvement. It terminates. | Days. | Safe, one agent. |
| **`conformance`** *(added 2026-08-08, ADR-030)* | The primary verdict is **external differential agreement** against a declared oracle tier, at a committed divergence rate, on a committed corpus. **There is no self-certificate and the lane brief says so in those words.** Its `lanes.toml` entry carries `self_certifying = false`, a non-empty **`oracle_systems`** list (external systems — *not* `oracle`, which holds lane ids), and a `divergence_ceiling`. | Weeks. Monotone in the rate, not in the capability. | One or two agents; the corpus is shared state. |
| **`decision`** | The deliverable is a ratified ADR. **Not an agent lane** — an agent may draft and may run the experiment; ratification is a human merge (ADR-021 §2). | — | — |

Two rules bound the conformance grade and are what keep it from becoming an escape hatch
(ADR-030 §2, §3): **soundness is never conformance-graded** — it is certificate-graded
separately, in the same crate, so a rewriting lane splits into rule-soundness (certificate)
and rewrite-quality (conformance) — and **a conformance lane gates nothing**, may be an oracle
for nothing, and fails rather than passing when its oracle is absent.

Two rules follow and are enforced mechanically, not culturally:

1. **A score lane's CI job does not exist until every lane in its `oracle` list is green and
   frozen** (ADR-021 §3). Without that, a score lane has no verdict function at all, and
   founding constraint #3 requires every lane to have one.
2. **A lane's crate is absent from the workspace members list while any ADR it inherits is
   not `Ratified`.** That is the freeze, and it is a dependency edge rather than an
   intention.

### The freeze, stated precisely

The plan's previously claimed global barrier was "the ADR freeze". **There are three global
barriers, in order, and only the first was named:**

1. **The ADR freeze** — the decisions in `lanes.toml`'s `gates` columns are `Ratified`.
2. **Lane Z0 — `resolvent-base`.** The trait tower, `Sign`, `Verdict`, `Certified` /
   `Certainty` / `ProofKind` / `Certificate`, `Error` / `Unsupported` / `Budget`, and the
   canonical serializer. `plans/architecture.md` §1.4 says of it "must land first;
   everything inherits it", ADR-006 marks its signature a one-way door — and it appeared in
   no lane, no wave, and no milestone's *Lands* list in the previous plan. It is the single
   most-depended-on deliverable in the project.
3. **The canonical serializer** (inside Z0). The determinism harness, the corpus format and
   every oracle adapter all serialize polynomials, and `plans/architecture.md` §4.5 already
   requires one implementation shared by all of them. Three agents writing three serializers
   is a merge that rewrites two of them.

Everything before those three is harness work. After them, three trunks run concurrently and
never touch each other's one-way doors.

### The sequencing gift, and it still holds

The first consumer touches none of the multivariate machinery: its polynomial type is a
dense `Vec<Rational>`, its resultants are hand-rolled 2×2 conic determinants, and its
`RealRoot` is exactly `AlgebraicReal`. So **after the shared ADRs land there are two trunks,
not one**, and the consumer-unblocking trunk never waits on the Gröbner one-way doors —
provided `UPoly<C>` is defined standalone and first, and `MPoly` converts down to it rather
than the reverse (ADR-007). That is the single largest schedule win available and it is free.

---

## 1. Milestones

Each states what lands, what it unlocks, an exit gate a CI job evaluates, and its real
dependencies. Numbers written `⟨committed⟩` are set by the first measurement and then
ratcheted (ADR-024 §3); a gate with an uncommitted number **fails**, because `TBD` is not a
ceiling.

---

### M0 — The harness (no algebra)

**Lands.** Workspace with the two-category rule (`publish = true` gated by `cargo-deny`;
`publish = false` may carry LGPL dev-deps and shell out to GPL binaries) and gate L6a (zero
dev-dependencies on published crates). CI Gate 0 with the tiered corpus (ADR-024 §1). License
gate plus its three-case regression corpus. `lanes.toml` and the ratification gate
(ADR-021 §3). Corpus format with provenance fields, generator interface, seed schedule,
minimizer, score reporter. Tier-0 sympy adapter with the S-expression protocol, the triage
classifier, and its **calibration corpus** (ADR-016 §5). Benchmark runner skeleton with
change-point reporting. `sharpness-ceilings.toml` and `tuning-thresholds.toml`, empty.

**Unlocks.** Every subsequent lane. Nothing can be graded before this exists.

**Exit gate.**

- Gate 0 green on a workspace containing no algebra, **within the committed `fast` budget of
  90 s**, with the tier census printed.
- The license gate **fails** on all three planted cases (`malachite`; `polynomen`; a
  synthetic Apache-only crate depending on `rug`). *A gate that passes what it must reject is
  not a gate.*
- `cargo metadata` asserts every `publish = true` crate has an empty `[dev-dependencies]`
  table (gate L6a).
- The ratification gate is **observed blocking**: a lane whose gating ADR is set to
  `Proposed` in a scratch commit has its crate absent from the workspace and its tests
  skipped, and the same lane with `Ratified` builds.
- The score harness reports a falsification against a deliberately-buggy stub within
  `⟨B⟩` CPU-seconds at fleet version 1, reports `survived` at `⟨B⟩` once the stub is fixed,
  and **two runs at the same `(fleet_version, commit)` are byte-identical**.
- The minimizer reduces each of three planted counterexamples to a **1-minimal** form — no
  single further reduction step in the triage order preserves the disagreement — of at most
  `⟨k_i⟩` terms within `⟨T⟩` seconds.
- The sympy adapter round-trips a polynomial through the S-expression protocol **and** its
  calibration corpus passes: `Res(x²−2, x²−3)`, `gcd(x²−1, x³−1)`, `factor(x⁴+1)` over ℚ and
  `GF(3)`, `isolate_roots` of a Chebyshev polynomial, each against a hand-computed committed
  answer.
- The oracle skip census prints, and a job declaring Tier 0 **fails** when `sympy` is absent.

**Depends on.** Nothing.

---

### M1 — Layer 0 and the representation freeze

**Lands.** Three halves, and all three must complete.

*Decisions (lane D1, `decision`-graded):* the ADRs in `lanes.toml`'s Wave-0 and Wave-1 gate
columns, `Ratified`. This is the freeze.

*Foundations (lane Z0):* `resolvent-base` — the ADR-006 trait tower **typechecked with real
`impl`s for `Fp` and `Integer`**, `Sign`, `Verdict`, `Certified`/`Certainty`/`ProofKind`/
`Certificate<C: Claim>`, `Error`/`Unsupported`/`Budget`, and the canonical serializer.

*Code:* `resolvent-int` (the `Integer`/`Rational` newtype wall over `dashu`, `dashu` in no
public signature and no re-export); `resolvent-modular` (`Fp` for word primes with
Barrett/Shoup and a benchmarked Montgomery path, `Zn`, `GF(p^k)`, bulk vector ops
first-class, the batched tuple ring with `inv_batch`); CRT, rational reconstruction,
deterministic prime selection with the sieve cross-check.

*Measurements committed before the freeze:* `bigint-benchmark-rs` re-run with `dashu` 0.5.2
pinned (every published figure used 0.4.2, one release **before** NTT landed in 0.4.3, so the
widely-cited number is stale); the `gcd`/`gcd_ext` ladder at 64 / 256 / 1k / 4k / 16k / **64k
/ 256k** bits against `rug`; and a `rational_reconstruct` microbenchmark at Hexapod's modulus
size (≈70 kbit).

**Unlocks.** Both trunks.

**Exit gate.**

- **The ADR-006 trait block compiles** with `impl Ring for Fp` and `impl Ring for Integer`,
  and the trait-law property suite is green for every instantiation in the closed set.
- `cargo public-api` snapshot for `resolvent-base` committed.
- `Fp` exhaustively certified against `i128` for **every pair `(a,b)` with `a,b < p`** for
  every prime `p < 2^10` — the loop bound is `p²`, not `p`, and a mutant with the wrong bound
  is in the mutant set (ADR-023 §1); random-certified for `p < 2^63`; `a·a⁻¹ == 1` for every
  unit.
- Bulk path componentwise-equal to the scalar path including tails and misaligned lengths.
- Batched tuple ring componentwise-equal to `N` independent scalar runs, **and** a planted
  per-lane zero forces `inv_batch` to return the correct `LaneMask`.
- **Prime registry cross-checked against an independent segmented sieve** over all primes
  below `2^24` plus the first `N` registry entries near `2^27`, `2^31`, `2^63`; the accepted
  set's count and hash committed as a golden file. A mutant with one corrupted Miller–Rabin
  witness is rejected.
- CRT: moduli asserted **pairwise distinct**, `M = Π pᵢ ≥` the caller's sizing bound, and
  `result ≡ rᵢ (mod pᵢ)` for every `i`.
- CRT and rational reconstruction 100 % `Proved` on the fleet, **including Hexapod**
  (1102 primes for a 0.00 s modular run — the instance that finds reconstruction bugs).
- Prime generation deterministic across runs, processes and thread counts.
- `docs/research/bignum-ladder.toml` exists, contains `(dashu_ns, rug_ns, ratio)` medians of
  `k` runs with IQR per instance on the pinned machine, **and** contains the ADR-002
  §Decision 7 verdict line evaluated against its pre-committed 8× trigger.
- Every ADR in `lanes.toml`'s "gates everything" column, plus at least one trunk's column,
  has a `Status:` line matching `^Ratified`.

**Depends on.** M0.

---

### M2 — The univariate engine over ℤ

**Lands.** `UPoly<C>` standalone (`Vec<C>` plus one `C::Ctx`, no monomial type, no order)
with content/primitive part, canonical associate normalization, Horner evaluation,
`map_coefficients`, derivative, `divrem`, pseudo-division, **public** reciprocal transform
`xⁿ·p(1/x)`; modular gcd with the **Bézout-witness** certificate; `gcd_ext`; Yun square-free
decomposition; Sturm sequences **as an oracle**; Descartes/VCA isolation over ℤ on dyadic
intervals.

**Unlocks.** Layer 3. Also a DAE integrator's event-detection shape — dense-output
coefficients lifted from `f64` to ℚ, isolated under a budget, declining rather than hanging.

**Exit gate.**

- `count_sturm(f,a,b) == len(isolate_descartes(f,a,b))` on the entire generator fleet.
- **`d*` measured and committed**: the largest degree at which Sturm's median runtime on the
  pinned machine is ≤ `⟨T⟩`. Below `d*` the isolation lane's verdict is CERT; **above `d*` it
  degrades to DIFF, and the degradation is recorded in the lane's status** rather than
  discovered later as a mysteriously slow CI job.
- gcd certificate 100 % `Proved`, in its **non-circular** form (ADR-023 §2): `H|A`, `H|B`,
  **and** a Bézout pair `(u,v)` with `u·A + v·B == H`; for the ℤ[x] two-part certificate the
  per-prime modular gcd returns its GF(p) cofactors so the degree half is itself certified.
  The identity mutant (`gcd → 1`) is rejected.
- Yun: `Π fᵢ^i == f`, factors pairwise coprime, each square-free; the coarsening and refining
  mutants are rejected.
- Isolation invariants: disjoint, ordered, `f(lo) ≠ 0 ≠ f(hi)`, Descartes variation exactly 1
  per interval, all within the Cauchy bound, multiplicities summing correctly, round-trip
  from constructed roots.
- **Landau–Mignotte / Cauchy bound validity** (ADR-023 §5): on every known-factorization
  instance the computed bound is `≥` the true maximum factor coefficient, with `bound/actual`
  tracked as a distribution against a committed ceiling.
- Tier-0 differential green on gcd, square-free and isolation with the pinned normalization.
- **Every randomized certificate in this milestone is evaluated over the fleet seed schedule**
  and the seed count is reported alongside the score (ADR-023 §3).

**Depends on.** M1 (all of it — Z0 included).

---

### M3 — Layer 3: algebraic numbers — **v0.1**

**Lands.** `AlgebraicReal` (square-free defining polynomial over ℤ, isolating rational
interval) with `refine`, total `cmp` with equality decided by gcd plus a sign-change
certificate, `try_cmp`, `cmp_rational`, exact `sign_of`, an outward-correct `(f64, f64)`
enclosure pair — **no float interval type in the public API**. `IsolatedRoot { value,
multiplicity }`. `SqrtExt` as a first-class type, not subsumed. Radical-tower sign at
arbitrary depth. Separation bounds, converting "terminates eventually" into "terminates in a
computable number of steps". `rational_between`. Bernstein/de Casteljau certified range
enclosure.

**Unlocks.** The consumer's whole predicate surface *at degree ≤ 4* — usable but not yet
interesting. Ship it as v0.1 anyway: it makes the API real, forces the adapter question, and
gives the score lanes a public baseline.

**Exit gate.**

- All eleven Layer-3 properties green under an explicit **step budget**, with exhaustion
  counted as a failure. Transitivity, sort stability and the no-hang budget are the three
  that matter.
- **INV-AR1 tested directly** (ADR-011 §4, ADR-013 §2): for a corpus of comparison sequences,
  the verdict of every call — including whether it declines — is identical when run cold,
  when run after arbitrary pre-refinement, and at `RAYON_NUM_THREADS ∈ {1, 8}`.
- **No `Equal` from bound exhaustion** (ADR-013 §5a): a fault-injection test that inflates
  the separation bound by 2× produces **internal-invariant failures, never `Equal` verdicts,
  and never a changed ordering**.
- **`Ord`'s step distribution measured and its ceiling committed** (ADR-013 §5b, lane Y1):
  the 99.9th-percentile `cmp` step count over the M4-shaped corpus is published, and the
  diagnostic ceiling is set from it with a stated outward margin. `try_cmp` is benchmarked
  against `cmp` on the same corpus.
- Radical-tower sign agrees with the materialized `AlgebraicReal` route on the whole fleet —
  the strongest free internal oracle in this layer, and its permitted-import set is committed
  (ADR-023 §6).
- `SqrtExt` sign-by-squaring agrees with the general route; cross-root comparison total.
- Separation-bound row graded **INV+PROP, not CERT**, plus a symbolic unit test against
  brute-force certified separations at degree ≤ 6.
- Bernstein: soundness certificate green; **the Unknown rate is measured, committed to
  `sharpness-ceilings.toml` in the same PR, and is exactly 0 on the clear-sign sub-corpus.**
- Constructive generators present for equal-value/different-representation pairs,
  deliberately-close triples, overlap-endpoint-on-a-root pairs, and `sign_of` at zero.
  Random generation finds none of these.
- The `enclosure_f64` conformance vectors (ADR-015 §5) pass, including subnormals, powers of
  two, exact halves, and the largest finite double.
- Zero panics and zero unbounded runs across the fuzz targets.

**Depends on.** M2; ADRs 013, 014, 015. **Experiment E-MUT must have returned** — and E-MUT
runs at the tail of M2, not inside M3, because it needs only `cmp`, `refine` and polynomial
sign evaluation over `UPoly<Integer>` with roots built as `Π(x−rᵢ)` (≈300 lines), not the
production isolator.

---

### M4 — Elimination — **v0.2, the consumer unlock**

**Lands.** `RecursiveView` as a borrowed view; subresultant PRS (Ducos) over ℤ; modular
evaluation–interpolation resultant for the bivariate case; a Bareiss/Bézout determinant route
as a third independent implementation; `Res_y(f,g)` with the ADR-025 conventions and
`ResultantOutcome`; bivariate gcd and common-component detection; curve analysis (critical
abscissas, per-interval branch counts, branch-index-to-root maps) with **no geometric type in
any signature**; cheap sharing of analysis results.

**Unlocks.** This is the real thing. Arbitrary-degree curves for the geometry consumer:
three hand-rolled resultants collapse into one call, the lossy double-squaring elimination
and its spurious-root filter disappear, and the degree-4 ceiling lifts.

**Exit gate.**

- **All three routes agree on the whole fleet, including the degree-drop and shared-component
  adversarial families**, under the ADR-025 conventions — with the `(−1)^{mn}` swap rule
  applied **explicitly** and no comparison anywhere accepting "up to sign".
- Route independence **enforced**, not asserted: each route's permitted-import set is
  committed in `lanes.toml` and CI fails on an edge into a route it grades. The
  Bareiss/Bézout route shares only `Integer` arithmetic and is worth building for exactly
  that reason.
- Resultant cofactors: `u·f + v·g == Res` exactly. Degree bound
  `deg_x Res_y ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)` on every output.
- `Res == 0 ⇔ deg gcd(f,g) ≥ 1`, cross-checked against the M2 gcd lane. An identically-zero
  resultant returns `ResultantOutcome::CommonComponent { gcd }`, **never** a bare zero.
- The Poisson-product check is marked **M8, not available here** — it needs a splitting field.
- Subresultant chain: degree sequence matches exactly; each `S_i` matches after the pinned
  Ducos scalar convention; the specialization property holds at random good primes and
  evaluation points **over the fleet seed schedule**.
- Curve analysis agrees with the independent rational-witness route (isolate the roots of
  `f(α,y)` at a rational abscissa strictly inside each interval) on the whole corpus. Branch
  counts consistent across adjacent intervals; branch matching a bijection.
- Tier-1 differential (PARI `polresultant`, sympy `subresultants` for the whole chain) green,
  with each adapter's calibration corpus passing first.
- **Score lane, tracked separately:** modular bivariate resultant ≥ 100× the Ducos route on
  ℤ[x,y] degree ~20. Baseline frozen before the lane starts.

**Depends on.** M2, M3, ADR-025. Does **not** depend on the monomial layer — only on a
bivariate representation.

---

### M5 — Factorization over ℤ

**Lands.** Cantor–Zassenhaus / Berlekamp over `GF(p)`; Hensel lifting to the Landau–Mignotte
bound; Zassenhaus recombination with an explicit `r` threshold (~10, i.e. ≤ 1024 subsets);
LLL; van Hoeij lattice recombination above the threshold.

**Unlocks.** Intersection multiplicity beyond the parity heuristic; minimal polynomials,
hence a defensible `canonicalize()` and `Hash` for `AlgebraicReal`; SMT NRA projection-set
control.

**Exit gate.**

- `GF(p)` factorization: multiply-back **and** the complete irreducibility test per factor.
  Over a finite field both halves are decidable and cheap; this is the one place
  factorization is fully certified.
- Over ℤ: multiply-back on every instance; **factors asserted pairwise non-associate after
  canonical normalization**, with the exponent multiset checked against the input degree — so
  `f = g·g` returned as two multiplicity-1 factors fails (ADR-023 §5).
- The modular irreducibility certificate wherever it exists, with its **success rate tracked
  against a committed ceiling**. A falling rate means the implementation got coarser or the
  corpus got harder, and both need a look.
- **The Landau–Mignotte bound has its own certificate row** and is not assumed: on every
  known-factorization instance, `bound ≥` the true maximum factor coefficient, `bound/actual`
  tracked. A too-small bound is how van Hoeij returns a coarse factorization that multiplies
  back correctly.
- Zassenhaus and van Hoeij agree for `r ≤ 20`.
- **Swinnerton–Dyer ladder:** degree 32 (`r ≈ 16`) completes under Zassenhaus; degree 64
  (`r ≈ 32`) separates van Hoeij from Zassenhaus; degree 256 is the "van Hoeij is really
  working" mark. One instance from this family is worth a thousand random ones: it is
  irreducible, has **no** modular irreducibility certificate at any prime, and a coarse
  implementation returns a nontrivial factorization.
- LLL output satisfies the Lovász and size-reduction conditions, with a unimodular transform
  and preserved determinant.
- Coarsening and refining mutants rejected for every factorization row.

**Depends on.** M2 (gcd, square-free), M1 (`GF(p^k)`, `Zn` for Hensel lifting to `p^k`).

---

### M6 — Multivariate and Gröbner

**Lands.** The monomial layer (packed order-normalized key, raw exponents, divmask,
content-derived ids, `W_KEY`/`W_RAW`, guard-bit overflow detection, widen-and-restart
driver); `MPoly` with heap-based multiply/divide; Buchberger + Gebauer–Möller **as the
oracle**; the divisor-query index; F4 matrix construction and sparse `GF(p)` row reduction
plus its **naive dense reference**; modular Gröbner with tracing, majority vote,
stabilization, and the batch-split driver; `groebner_certified` with cofactor retention
(subject to E-COFACTOR); FGLM over a dual-key pair ring.

**Unlocks.** Exact medial axis and anything needing ideal theory. **Not** the geometry
consumer, which needs none of it.

**Exit gate — staged, because this milestone has four different verdict types.**

| Stage | Gate |
|---|---|
| **Monomial layer** | Order axioms including multiplicative compatibility; agreement with a naive `Vec<u32>` comparator per order; encode/decode round-trip **at and past the capacity boundary**; overflow always detected, never wrapped; widen-and-restart produces the same answer as starting wide; `MonomialId` is a pure function of the key, verified by interning the same multiset in 100 shuffled orders and at 1/8 threads and asserting identical id assignment |
| **Overflow sweep** | For each width `w ∈ {4, 8, 16}` and each instance, with `D_max` from the wide run: the narrow run **must complete and match iff `D_max ≤ 2^(w−1) − 1`**, and **must report overflow otherwise**. False positives fail; silent wraps fail. Counts printed per width, and **a width at which zero instances complete is a failed sweep**. The boundary sub-corpus (degree exactly `D` and `D+1`) must be present at each width and land on opposite sides |
| **Buchberger (oracle)** | Correct on Cyclic-7, Katsura-8, Eco-10; agrees with Singular; cofactor certificate for both ideal inclusions. **The S-pair certificate enumerates all `C(\|G\|,2)` pairs and consults no elimination criterion**; a Gebauer–Möller mutant that drops one extra pair class is rejected |
| **F4 correctness** | Agrees with Buchberger on every instance Buchberger can reach; agrees with `groebner_certified` |
| **G3's own oracle** | Sparse `GF(p)` row reduction agrees with the **naive dense `u32` Gaussian elimination over the same `FpParams`, in the same crate** — because `groebner_certified` is a ℚ/ℤ reducer and never executes a line of G3 (ADR-010 §5) |
| **SIMD** | With `simd` enabled, every bulk kernel is **bit-identical** to its scalar fallback on random vectors including tails and misalignment (ADR-022 §4) |
| **Batching** | Componentwise equality with `N` scalar runs; **plus** a planted zero pivot in one lane forces a correct `LaneMask` and a batch split, and a planted lead-monomial divergence forces a split with the offending prime index recorded in the `Trace` |
| **FGLM** | Lex basis reduces the drl basis to zero and vice versa; **and the lex output satisfies Buchberger's criterion in the lex order**; **and the lex staircase has exactly `dim_ℚ ℚ[x]/I` standard monomials** — a number FGLM already computes |
| **Modular over ℚ** | Katsura-10/11, Cyclic-8, Chandra-13, Reimer-8 complete; Hexapod completes; `groebner` agrees with `groebner_certified` on every regression instance it can reach |
| **F4 performance** (score) | *Working*: Cyclic-8 < 60 s, Katsura-11 < 500 s, Eco-13 < 500 s. *Competitive*: Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s (≈2× SOTA) **and this rung is only publishable with ADR-022's SIMD leaf; without it the published rung is 3–4× SOTA, with AVX2 named as the reason.** **Do not plan for state of the art** (within 1.5× of msolve/Maple/Groebner.jl) |

**Depends on.** M1, M2, ADRs 008/009/010/020/022, and **E-MONO returned**. **Buchberger must
be green and frozen before F4 starts** — a CI-enforced edge, not a suggestion.

---

### M7 — Layer 4: the expression DAG (mostly parallel, starts at M1)

**Lands.** Hash-consed DAG over a caller-owned `Store` (never a thread-local or `static`);
node set `{ Const, Symbol(interned), ring ops, Apply(FuncId, args) }` with a caller-owned
`FuncTable` carrying arity and an optional derivative rule; `diff` **and
`diff_with(expr, sym, &LeafRules)`**; constant folding; `walk_topological` with stable ids;
`is_polynomial_in(&syms) -> Option<MPoly>`; `rebuild_from` for cross-store movement;
`canonicalize` as an explicit value-preserving function; canonical bytes with a schema
version. **No code emitter, and no e-graph dependency** (ADR-017 §1, §4 — both still stand).

Also in M7 (ADR-029, ADR-031, ADR-033):

- **The exactness lattice** — `Exact` / `Enclosed` / `Approximate` on every node, monotone
  under composition, plus `provenance_bytes` alongside `canonical_bytes` (ADR-031). This is in
  the node identity, so it is **not** an additive follow-on: it lands with X1 or it is a
  rewrite of the `Store`.
- **`simplify(expr, &RuleSet, budget)` and `RuleSet` with R/S/D rule classification**
  (ADR-033). Never implicit, no default rule set. Rule *soundness* is certificate-graded; rule
  *quality* is a separate conformance lane.
- **Assumptions on symbols**, as the discharge mechanism for class-D side conditions. In
  scope, unspecified — its lane is blocked until an ADR specifies it.
- **No *unsound* zero-test, ever** (ADR-032) — but the tier machinery is **not** in M7. It
  lands in `resolvent-calculus` at M9, because a Tier-1(b) reduction produces an
  `AlgebraicReal` and placing it here would give L4 a dependency on L3 (ADR-005, amended).

**Unlocks.** Build-time symbolic differentiation for a multiphysics forcing-term generator
and for a Pantelides index-reduction pass that is currently an identity stub waiting on
symbolic `d/dt`. FEM form compilation. **And, after ADR-029, M9** — M7 is now the foundation
of a stratum rather than a leaf, which is the sequencing change that matters most in this
document.

**Exit gate.**

- On the polynomial subset, `diff` equals `UPoly::derivative` **exactly** — an exact
  cross-layer oracle covering chain, product and power rules.
- Hash-consing injective; canonical bytes byte-identical across insertion orders, thread
  counts, processes and feature combinations, with golden files and a schema version, and a
  golden change without a version bump fails.
- `is_polynomial_in` sound in both directions: `Some(p)` ⇒ `p` and the expression agree at
  random points over the fleet seed schedule; `None` ⇒ a witness node that is not a ring op
  over the given symbols **or is not `Exact`**. The signature stays `Option<MPoly>`
  (`API.md` L4-5); the witness is what carries the diagnosis.
- **Exactness is monotone under composition** on generated DAGs with inexact leaves planted at
  every depth, and no `Exact` node has an `Approximate` descendant — **including `diff` of an
  inexact constant, which is `Approximate(0)`, not `Exact(0)`**. The **promotion mutant** —
  constant-folding an `Approximate` child into a `Rational` and labelling the result `Exact` —
  is rejected. (ADR-031 §3.)
- **Constant folding fires only when every operand is `Exact`.** resolvent performs no
  arithmetic on an inexact value; a planted folder that computes through one is rejected
  (ADR-031 §6).
- **No decision is made from an inexact node.** Sign queries over every `Enclosed` and
  `Approximate` node return `Unknown`, *including where the leaf enclosure excludes zero*.
  That case is the one a filter would have decided and the one ADR-015 forbids resolvent to
  own.
- **`provenance_bytes` is byte-identical across insertion orders, thread counts, processes and
  feature combinations** — the same matrix as `canonical_bytes` — which is what proves no
  arena-relative `ExprId` leaked into it.
- **Construction is not rewriting.** A corpus of terms whose canonical form differs from their
  constructed form round-trips through construction, `diff`, `walk_topological` and
  `canonical_bytes` with structural identity preserved. This is the promise cadabra2's `Cos2`
  tether rests on (ADR-033 §2).
- **Every class-R rule in the shipped rule sets is verified by GF(p) evaluation** with each
  `Apply` node replaced by a fresh variable, across the fleet seed schedule; **no class-D rule
  fires without its side condition discharged**, with a planted undischarged case asserted to
  refuse.
- `rebuild_from` round-trips: rebuilding an expression into a fresh `Store` yields identical
  canonical bytes.
- `diff_with` with `LeafDefault::Refuse` returns `Unsupported::NoLeafRule` rather than a
  silent zero; with an opaque `Apply` and no derivative rule it returns
  `Unsupported::NoDerivativeRule`.
- **The two adapter sketches compile and run against the real API in under 200 lines each.**
  That is the acceptance test and it is a real one: `diff_with`'s leaf-rule table is the
  difference between the index-reduction adapter existing and not.

**Depends on.** M1 (Z0 and `resolvent-int`), ADRs 012/017/020. **Not fully independent of
the other trunks, and the previous plan said it was:**

| Sub-lane | Actually depends on |
|---|---|
| X1 `Store`, node set, `FuncTable`, **the exactness lattice** | nothing beyond M1 — **genuinely independent** |
| X3 `walk_topological`, canonical + **provenance** bytes, `rebuild_from` | nothing beyond M1 — **genuinely independent** |
| X2 `diff` / `diff_with`, constant folding | **U2**, because its exit gate is agreement with `UPoly::derivative` |
| X4 `is_polynomial_in` | **P3**, because its return type is `MPoly` |
| **X5** `RuleSet` + `simplify`, rule soundness | *added 2026-08-08.* **X1 + Z3**, because class-R soundness is evaluation over GF(p) and `Fp` is Z3. **Not** a gcd lane: rational-function normal form is a *separate* unspecified capability (ADR-033 §5) and is not in X5 |
| **X5q** rewrite *quality* — **conformance-graded** | *added 2026-08-08.* X5, plus a live Tier-0 oracle. Gates nothing (ADR-030 §3) |
| **X6** assumptions on symbols | *added 2026-08-08.* X1. **Blocked: no ADR specifies it** (ADR-021 §3) |

X1 + X3 are the cleanest parallel work in the plan and should be staffed from the beginning.
X2 and X4 are not free of the other trunks and must not be scheduled as though they were.

**X1 is the scheduling item that bites.** The exactness lattice is a field in the hash-consed
node, so it is not additive: adding it after X1 ships is a rewrite of the `Store` and of every
golden file X3 committed. X1's brief must carry it from the first commit.

---

### M8 — Towers, CAD support, and 0-dimensional solving

**Lands.** `UPoly<NumberFieldElem>` as an instantiation **plus lane M8-N, the multi-modular
split-factor driver that makes it fast**; subresultant chains and principal subresultant
coefficients as first-class returned data; ideal saturation; RUR or triangular decomposition
for 0-dimensional real solving; cofactor/certificate return plumbed through gcd, resultant
and Gröbner as a *public* output.

**Unlocks.** SMT NRA and exact medial axis.

**Exit gate.**

- Subresultant chain matches sympy's `subresultants` after the pinned Ducos convention;
  principal subresultant coefficients returned and cross-checked against the chain.
- Root isolation over `ℚ(α₁..α_k)` passes the full Layer-3 property suite with tower
  coefficients.
- Saturation cross-checked against a Rabinowitsch-trick computation in one extra variable.
- **`Reducible` over a number field returns `Err(BadPrime)` rather than a zero divisor**, and
  the M8 corpus contains **ℚ(√2, √3)** specifically — the multiquadratic tower where no prime
  is inert and a naive implementation divides by a zero divisor (ADR-006 §Context defect 3).
- The multi-modular split-factor path agrees with the Tier-G reference path on the whole M8
  corpus, and its speedup over that path is a tracked number with a committed floor —
  because if it has no speedup, M8-N did not happen and `UPoly<NumberFieldElem>` is correct
  and useless for its only consumer.
- The Poisson-product resultant check, deferred from M4, lands here.

**Depends on.** M4, M5, M6.

---

### M9 — the analytic stratum — *declared 2026-08-08, specified almost nowhere*

**This milestone is deliberately not plannable yet, and saying so is the point.** ADR-029
declares the analytic surface in scope. `API.md` §4.2 admits it capability by capability. But
"in scope and unspecified" blocks a lane rather than licensing it (ADR-021 §3), and every
capability below is unspecified. **No M9 lane may open until its own ADR ratifies.** Listing
them here is what makes their absence visible instead of turning M9 into an invitation.

**Lands.** `resolvent-calculus` (ADR-005, amended) and `resolvent-display`.

| Capability | Verdict function it will be graded by | Grade | ADR |
|---|---|---|---|
| **C1 — ADR-032's zero-test tiers**: the reduction table, its witness relations, the Tier-2 opt-in. `is_zero` is a **free function** over `&Store`, never a `Store` method | `witness_relation_holds`; the numeric mutant and the `sin(π/6) → √3/2` table mutant must both be rejected; a compile-fail test that `is_zero` returns no `Verdict` (INV-18) | certificate | **ADR-032 — the one M9 capability that is specified, and the only M9 lane that may open** |
| Series and limits | Truncation bound plus agreement with direct evaluation | certificate | none yet |
| Symbolic integration | **Differentiate the result and compare to the integrand.** An unusually strong self-certificate — this is why ADR-017 §6 always called it cheap to add later | certificate | none yet |
| Partial fractions | Recombine and compare (admitted at `API.md` §4.2 as a consequence of integration's internal need) | certificate | none yet |
| ODE solving | Substitute the solution back | certificate | none yet |
| Integral transforms | Invert and compare | certificate | none yet |
| Special functions as symbolic objects | Functional equations and recurrences — `Γ(z+1) = zΓ(z)`, the Bessel recurrences | certificate | none yet |
| Arbitrary-precision numeric evaluation of the above | **unsettled, and it is the hazard** | — | none yet, **and it needs one first** |
| Pretty-printing / LaTeX | Round-trip for correctness; readability has no verdict function | conformance | none yet |

**The one item that must not be waved through.** Arbitrary-precision *numeric* evaluation of
special functions is the single place the analytic surface touches ADR-012 §6's "no floating
point in any decision path". It is admitted nowhere: `API.md` §4.2's special-functions row
admits the symbolic objects and explicitly excludes their numeric evaluation. Whoever writes
that ADR must answer how a certified enclosure is produced and why it cannot become a decision
input — and ADR-031's `Enclosed` state, which carries rational endpoints rather than an
interval type, is the shape the answer probably takes.

**Unlocks.** The capability list of an established CAS. Nothing internal depends on M9.

**Exit gate.** Not writable yet, and it would be dishonest to write one: an exit gate over
capabilities whose ADRs do not exist would be a gate over a guess. Each capability's ADR
carries its own, and M9 closes when they all do.

**Depends on.** M7 for all of it; M3 additionally for the zero-test tiers (they produce an
`AlgebraicReal`); M2 for series over `UPoly`.

---

### Milestone dependency graph

```
M0 harness
 └─ M1 Layer 0: FREEZE + Z0 (resolvent-base) + int + modular ──────────┐
     ├─ (univariate trunk)                                             │
     │   M2 univariate over ℤ  ── E-MUT runs at its tail               │
     │    ├─ M3 algebraic numbers  ── v0.1                             │
     │    │    └─ M4 elimination   ── v0.2  ← the unlock               │
     │    └─ M5 factorization ─────────────────┐                       │
     │                                         │                       │
     ├─ (multivariate trunk, after E-MONO)     │                       │
     │   M6 monomials → Buchberger → F4 → modular Gröbner              │
     │                                         │                       │
     ├─ (expression trunk: X1+X3 free, X2←U2, X4←P3, X5←X1+Z3)         │
     │   M7 Layer 4 DAG ───────────────────────────────────────────────┤
     │    └─ M9 analytic stratum (resolvent-calculus, -display)        │
     │         ← needs M7; zero-test tiers additionally need M3        │
     │         ← EVERY LANE BLOCKED: no ADR specifies them (bar 032)   │
     │                                                                 │
     └─ M8 towers / CAD / RUR / M8-N   ← needs M4 + M5 + M6 ───────────┘
```

**M7 is the foundation of M9, not a leaf** (ADR-029 §5). ADR-017's "sequence it last and do not
let it block anything" no longer holds, and M7's own one-way door — the exactness lattice in
the node — has to be decided before X1 is staffed. Two constraints govern the ordering: **no
analytic lane may start before M1's representation freeze**, and **M4 remains the release that
unlocks the strongest evidenced consumer**; scope growth is not a reason to reorder it.

---

## 2. The fan-out plan

**Wave** is the earliest wave a lane may start. **Par** is how many agents can usefully work
it simultaneously. **Size** in agent-sessions (S = 1–2, M = 3–6, L = 7–15, XL = 16+);
score-graded lanes get **no size estimate**, because the completion condition is a number, not
a state, and writing one would be dishonest.

### Wave 0 — before any freeze

| Lane | What | Grade | Par | Size | Verdict function |
|---|---|---|---|---|---|
| **H1** | Workspace, two-category rule, gates L1–L10 + L6a **+ L13–L15 (embedding, ADR-029 §2)**, `cargo-deny` + `cargo-about`, `lanes.toml` (**including the conformance keys, ADR-021 §3 as amended**) and the ratification gate | certificate | 1 | S | Gate fails on all three planted license cases; the ratification gate is observed blocking; **each of L13–L15 is observed rejecting a planted violation** |
| **H2a** | **The canonical serializer and its schema version** (`resolvent-base`, ADR-012 §9) | certificate | 1 | S | Round-trip; golden files; a golden change without a schema bump fails. **Blocking for H2b, H3, H4** |
| **H2b** | Determinism harness, thread/process/feature matrix, golden-file machinery | certificate | 1 | S | Byte-identical across runs/processes/threads/features |
| **H3** | Corpus format with provenance, generator interface, seed schedule, minimizer, score reporter, tier census | certificate | 1 | M | Falsifies a planted stub within `⟨B⟩`; 1-minimalizes three planted cases; `fast` budget enforced |
| **H4** | Tier-0 sympy adapter, S-expression protocol, triage classifier, **calibration corpus** | certificate | 1 | M | Round-trip; hand-computed calibration answers; correct Class A/B on planted disagreements |
| **H5** | Benchmark runner, pinned-machine protocol, per-series change-point calibration | measurement | 1 | M | Detects a planted level shift; does not flap on measured noise |
| **Z2** | `dashu` 0.5.2: `bigint-benchmark-rs` re-run + `gcd`/`gcd_ext` ladder **to 256 kbit** + `rational_reconstruct` at 70 kbit | measurement | 1 | S | `bignum-ladder.toml` committed **with the pre-committed 8× trigger evaluated** |
| **Y1** | Consumer-workload measurement: degree 3–8 curve pairs, `Res_y` degree and coefficient bit-length, `isolate_roots` + `sign_of` wall time against the existing dense-ℚ implementation | measurement | 1 | S | Numbers committed. **Every geometry performance requirement currently rests on a guess and this removes it** |

**Honest concurrency: 5, then 7 — not 8.** H2a blocks H2b, H3 and H4, because all three
serialize polynomials and `plans/architecture.md` §4.5 already requires one shared
implementation; three agents writing three serializers is a merge that rewrites two of them.
H1, H5, Z2 and Y1 do not depend on it and run alongside. The previous plan listed these as
"5–7 lanes of genuinely independent work with no shared state", which was wrong about the
shared state.

### Wave 1 — Layer 0, after the freeze

| Lane | What | Grade | Par | Size | Verdict function |
|---|---|---|---|---|---|
| **Z0** | **`resolvent-base`**: the ADR-006 trait tower typechecked with real impls, `Sign`, `Verdict`, `Certified`/`Certificate`/`Certainty`/`ProofKind`, `Error`/`Unsupported`/`Budget`, absorbing H2a's serializer | certificate | 1 | M | Trait-law property tests green for every instantiation in the closed set; `cargo public-api` snapshot committed. **Blocking for all of Wave 1** |
| **Z1** | `resolvent-int` newtype wall; `Integer`/`Rational`; `try_from_f64` fail-closed; bit-length and rounding accessors | certificate | 1 | M | Ring/field axioms; `rug` oracle **in `resolvent-oracles`, against the public surface only**; round-trip |
| **Z3** | `resolvent-modular`: `Fp` Barrett/Shoup + Montgomery, `Zn`, `GF(p^k)` | certificate | **2–3** | M | Exhaustive small-`p` vs `i128` **over `p²` pairs**; field axioms; Frobenius closure |
| **Z4** | Bulk/vector `GF(p)` ops (correctness only) | certificate | 1 | S | Componentwise equality with the certified scalar path, including tails and misalignment |
| **Z5** | Batched tuple ring `Fp4` + `inv_batch` (correctness only) | certificate | 1 | S | Componentwise equality with `N` scalar runs **plus** planted per-lane faults producing correct `LaneMask`s |
| **Z6** | CRT, rational reconstruction, deterministic prime selection with the **sieve cross-check**, good-prime predicates | certificate | 1 | M | Congruence + distinctness + range certificates; sieve golden file; Hexapod |
| **Z8** | `Fp` scalar throughput; Barrett vs Montgomery decided by measurement | **score** | 1 | — | Throughput on the pinned machine. **Blocked on Z3 frozen** |
| **Z9** | `resolvent-modular::simd` (ADR-022) | **score** | 1 | — | Bit-identical to the scalar fallback; throughput. **Blocked on Z4 frozen** |

**Honest concurrency: 1, then 5 lanes across up to 7 agents** (Z3 usefully takes 2–3), with
Z8 and Z9 additionally waiting on Z3 and Z4 being frozen. **Z0 is blocking, and this is the
correction that matters most in this wave**: six agents writing signatures against an
unwritten error taxonomy and an uncompiled trait tower produce six taxonomies. The previous
plan had no Z0 at all and listed the `Certificate` type as a peer lane (Z7) rather than a
prerequisite.

Z3 is the best agent lane in the project: small, extremely well specified, exhaustively
certifiable, and it decomposes cleanly into `Fp` / `Zn` / `GF(p^k)`.

### Wave 2 — the trunks open

**Univariate trunk**

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **U1** | `UPoly<C>` arithmetic, content/primitive, associate normalization, `divrem`, pseudo-division, reciprocal transform, `map_coefficients` | certificate | **2** | M | The naive `O(n²)` reference is **part of the deliverable**, so the oracle ships with the code |
| **U2** | Taylor shift, evaluation, Horner, derivative | certificate | 1 | S | Round-trip + evaluation homomorphism over the fleet seed schedule |
| **U3** | Modular gcd + `gcd_ext` + Yun, with **Bézout-witness certificates** | certificate | 1 | M | The degree half is the one people forget, and the Bézout witness is what makes it non-circular |
| **U4** | **Sturm — as an oracle** | certificate | 1 | S | Exact distinct-root count. Will never be the production isolator. **Build it because it grades U5**, and its permitted-import set is committed so it cannot later absorb the PRS it grades |
| **U5** | Descartes/VCA over ℤ on dyadic intervals | certificate | 1 | M | Graded by U4. Dyadic endpoints are the whole constant factor: `x→x+1` and `x→2^k x` stay integral, where arbitrary rational endpoints multiply denominators every subdivision |
| **U6** | Separation bounds + Landau–Mignotte/Cauchy validity rows | certificate (INV+PROP) | 1 | S | Validity, a tracked tightness distribution, and a symbolic unit test at degree ≤ 6 |
| **E-MUT** | The four `AlgebraicReal` mutability prototypes | measurement | 1 | S | ≈300 lines over U1/U2. **Runs here, not in Wave 3** |
| **U7** | ANewDsc Newton acceleration | **score** | 1 | — | **Blocked on U5 frozen.** Re-derive U5's interval invariants for the Newton path first — they were written for bisection and Newton steps jump |
| **U8** | Abbott QIR refinement | **score** | 1 | — | Blocked on M3's `AlgebraicReal` |
| **U9** | Fast Taylor shift (middle-product/FFT) | **score** | 1 | — | Crossover around degree 512; below that, do not bother |

**Multivariate trunk** — after ADRs 008/009/020 and **E-MONO**

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **E-MONO** | Record an S-pair operation trace from a throwaway Buchberger on Katsura-6/Cyclic-6; replay against both term representations | measurement | 1 | S | **Runs first.** ~200 lines of throwaway plus the replay harness |
| **P1** | Monomial encode/decode/compare/multiply, guard bits, widen-and-restart, `W_KEY`/`W_RAW` | certificate | 1 | M | Order axioms + naive comparator + capacity-boundary generators |
| **P2** | Term type + `divmask` + **content-derived ids** | certificate | 1 | M | Injectivity, hash multiplicativity `h(u)+h(v)=h(uv)`, one-sided divmask soundness, **and id assignment invariant under interning order and thread count** |
| **P3** | `MPoly` arithmetic with heap-based multiply/divide, owned ring handle | certificate | 1 | M | Representation invariants + naive reference |
| **P4** | Kronecker substitution utility | certificate | 1 | S | Round-trip |
| **P5** | Divisor-query index (kd-tree or equivalent) over **order-free `raw`** | **score** | 1 | — | Worth 10–20× in the reduction path — more than bit-packing's 15%. Blocked on P2 |

**Expression trunk**

| Lane | What | Grade | Par | Size | Blocked on |
|---|---|---|---|---|---|
| **X1** | Hash-consed `Store`, node set, `FuncTable`, **the exactness lattice** (ADR-031) | certificate | 1 | M | M1 only. The lattice is in the node and is not additive — see the note under M7 |
| **X3** | `walk_topological`, canonical **and provenance** bytes, schema version, `rebuild_from` | certificate | 1 | S | M1 only |
| **X2** | `diff` / `diff_with`, constant folding, `canonicalize` | certificate | 1 | M | **U2** |
| **X4** | `is_polynomial_in` bridge to Layer 1, refusing any non-`Exact` subtree | certificate | 1 | S | **P3** |
| **X5** | `RuleSet` with R/S/D classification, `simplify`, rule **soundness** (ADR-033) | certificate | 1 | M | **X1 + Z3**. Added 2026-08-08 |
| **X5q** | Rewrite **quality** against a Tier-0 oracle | **conformance** | 1 | — | X5 + a live oracle. Gates nothing. Added 2026-08-08 |
| **X6** | Assumptions on symbols — the class-D discharge mechanism | certificate | 1 | M | X1. **Blocked: no ADR** |

### Wave 3 — Layer 3 and elimination

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **A1** | `AlgebraicReal` core: construction, `refine`, `cmp`, `try_cmp`, `cmp_rational`, `sign_of`, enclosure, `IsolatedRoot` | certificate | 1 | L | The eleven properties plus INV-AR1 are the verdict. **E-MUT must have returned** |
| **A2** | `SqrtExt` | certificate | 1 | M | Graded against A1's general route |
| **A3** | Radical-tower sign at arbitrary depth | certificate | 1 | M | Graded against A1 materialization — a strong free oracle, with a committed permitted-import set |
| **A4** | `rational_between` | certificate | 1 | S | |
| **A5** | Bernstein certified range enclosure | certificate | 1 | M | Soundness cert **plus** a committed Unknown-rate ceiling, zero on the clear-sign sub-corpus |
| **T1** | Subresultant PRS (Ducos) over ℤ, ADR-025 conventions | certificate | 1 | L | |
| **T2** | Modular evaluation–interpolation resultant | certificate | 1 | L | |
| **T3** | Bareiss / Bézout determinant route | certificate | 1 | M | **The most independent of the three** — shares only `Integer` |
| **T4** | Bivariate gcd, common-component detection, `ResultantOutcome` | certificate | 1 | M | |
| **T5** | Rational-witness fiber oracle | certificate | 1 | M | **Build before T6.** It is T6's only strong verdict |
| **T6a** | Critical-abscissa extraction | certificate | 1 | M | `Res_y(f, ∂f/∂y)` plus isolation — well graded |
| **T6b** | Branch matching across critical fibers | certificate *(weak)* | 1 | L | **Poor agent target — see §4** |
| **T7** | Analysis-result sharing / caching | **score** | 1 | — | The prior art recomputes the resultant and re-isolates on every predicate call |
| **T8** | Modular resultant throughput | **score** | 1 | — | ≥100× Ducos at degree ~20. Blocked on T1+T2 frozen |

### Wave 4 — factorization and Gröbner

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **K1** | Cantor–Zassenhaus / Berlekamp over `GF(p)` | certificate | 1 | M | **Fully certified** — irreducibility over a finite field is decidable and cheap |
| **K2** | Hensel lifting + Landau–Mignotte bound **with its own certificate row** | certificate | 1 | M | Uses `Zn` |
| **K3** | Zassenhaus recombination | certificate | 1 | M | Also the oracle for K5 at `r ≤ 20` |
| **K4** | LLL | certificate | 1 | M | Output conditions directly checkable. **A better agent target than it looks** |
| **K5** | van Hoeij lattice recombination | partial-certificate | 1 | XL | **Worst agent target in the plan — see §4** |
| **G1** | Buchberger + Gebauer–Möller, **plus the exhaustive-S-pair verifier** | certificate | 1 | L | The oracle. Criteria eliminate 99.99 % of pairs; the verifier may consult none of them |
| **E-COFACTOR** | Cofactor **reconstruction** cost on Buchberger at Katsura-6/7 | measurement | 1 | S | Runs at G1's tail. Decides whether G4 is an API or an oracle |
| **G2** | F4 symbolic preprocessing + matrix construction | certificate | 1 | L | Graded by agreement with G1 |
| **G2r** | **Naive dense `u32` Gaussian elimination reference** | certificate | 1 | S | G3's internal oracle. One session, and without it G3 has none |
| **G3** | Sparse `GF(p)` row reduction | **score** | 1 | — | **73–91 % of an F4 run. The hardest score lane.** Graded by G2r internally and by Singular/msolve **as its primary verdict**. Blocked on G1+G2+G2r frozen |
| **G4** | Cofactor tracking (`groebner_certified`) | certificate + memory score | 1 | L | Scope set by E-COFACTOR |
| **G5** | Modular Gröbner: tracing, majority vote, stabilization | **score** | 1 | — | Returns `Probable`. **External differential testing is its primary verdict**, an inversion of the normal rule, written into the brief |
| **G6** | Batched multi-modular **and the split driver** | **score** | 1 | — | Up to ~2.7× amortized at `N=4`; 8/16/32 gave no further gain. Blocked on Z5 |
| **G7** | **Dual-key pair ring + FGLM** | certificate | 1 | XL | Re-sized from L: FGLM needs two orders live on the same monomials in the same loop (ADR-009 §Consequences) |

### Wave 5 — the analytic stratum, almost entirely closed

| Lane | What | Grade | Par | Size | Blocked on |
|---|---|---|---|---|---|
| **C1** | ADR-032's zero-test tiers in `resolvent-calculus`: the Tier-1(b) reduction table with a witness relation per entry, Tier-1(c) one-sided non-vanishing, the Tier-2 Schanuel opt-in, `is_zero` as a **free function** over `&Store` | certificate | 1 | M | **X1 + M3.** Needs the `Store` and `AlgebraicReal`. The only Wave-5 lane that may open |

**Every other capability in M9 has no lane here, and that is the mechanism working.** Series,
limits, integration, ODE, transforms, special functions, assumptions and presentation are all
admitted at `API.md` §4.2 and specified by no ADR, so under ADR-021 §3 their crates are absent
from the workspace and their lanes cannot start. Writing speculative lane rows for them would
convert "blocked" into "available", which is exactly the failure the ratification gate exists
to prevent. **Each needs a `decision`-graded lane first** — an agent may draft the ADR and run
its experiments; only the repository owner may ratify it.

### Fan-out summary

| Wave | Certificate lanes | Score/measurement lanes | Max useful concurrent agents |
|---|---|---|---|
| 0 | 5 | 3 | **5, then 7** — H2a blocks H2b/H3/H4 |
| 1 | 6 | 2 | **1, then 7** — Z0 blocks everything; Z3 takes 2–3 agents |
| 2 | 16 across three trunks | 6 + 1 conformance | **10–12** — E-MONO blocks the multivariate trunk; X2 waits on U2, X4 on P3, X5 on X1+Z3; X6 is blocked on an unwritten ADR |
| 3 | 12 | 2 | **8–10** — T5 before T6a/T6b; T8 after T1+T2 frozen |
| 4 | 10 | 4 | **5–6** — G1 → G2 → G2r → G3; K1 → K2 → K3 → K5 |
| 5 | **C1** only | — | **1** — C1, the ADR-032 zero-test tier lane in `resolvent-calculus`, and **nothing else**. Every other M9 capability is blocked on an ADR that does not exist, which is the mechanism working, not a gap to route around (ADR-021 §3) |

**The critical serialization, stated plainly.** Before the freeze, everything is harness
work. The barrier is not "Layer 0 and Layer 1 must be *built* first" — it is "**Layer 0 and
Layer 1 must be *decided* first, and `resolvent-base` must exist**". The univariate trunk
inherits a strictly smaller set of decisions than the multivariate one, which is why it can
and should start earlier.

**Within each trunk the serialization is oracle-first:** Sturm before Descartes, Descartes
before ANewDsc, Ducos before modular resultants, the rational-witness fiber oracle before
curve analysis, Zassenhaus before van Hoeij, Buchberger before F4, **the naive dense reducer
before the sparse one**. Each is an edge in `lanes.toml` and CI enforces it.

---

## 3. Which milestones and lanes are poor agent targets

Said plainly, because pretending otherwise wastes months.

**M1's ADR freeze — not an agent lane at all.** The deliverable is judgment about
irreversible tradeoffs with no verdict function. An agent can *draft* an ADR and can *run the
experiment* an ADR needs; ratification is a human merge. Trying to grade it automatically
produces an ADR that argues for whatever is easiest to test — which is exactly how the
original trait tower acquired a receiverless `zero()` that five of seven rings cannot
implement.

**T6b, branch matching across critical fibers — the weakest verdict in the plan attached to
one of the largest components.** It has zero counterpart in the prior art, its invariants are
weak (counts consistent across adjacent intervals, Bézout-style bounds, matching is a
bijection), and *the specification is most of the work*. Its only strong grader is T5, the
independent rational-witness route, which is why T5 is built first even though it will never
be the production path. Mitigation: the split above — T6a (critical abscissas) is just
`Res_y(f, ∂f/∂y)` plus isolation and is well graded; T6b is human-specified,
agent-implemented, against a **hand-authored corpus of curves with known topology** (nodal
cubics, cusps, tangential pairs, vertical components, vertical asymptotes).

**K5, van Hoeij — the hardest correctness lane in the library and the least agent-friendly.**
No permissive reference implementation exists *anywhere*: sympy (BSD) is plain Zassenhaus and
its own docs note LLL-based techniques are not implemented; FLINT/PARI/Magma are
LGPL/GPL/proprietary. It must be built from van Hoeij (2002), Klüners' exposition, and
Hart–van Hoeij–Novocin (ISSAC 2011). Its certificate is partial. It depends on an LLL at a
precision `lll-rs` is unproven at. **And its failure mode — returning a *coarse*
factorization — passes the multiply-back check**, and is reachable through a too-small
Landau–Mignotte bound, which is why K2's bound now has its own certificate row. Mitigation:
keep Zassenhaus as the production path with an explicit `r` threshold for as long as
possible; treat K5 as a research lane with a human reviewer; land K4 (LLL) as a separate,
fully certified lane first, because LLL's output conditions *are* directly checkable and
separating it removes the precision question from the recombination question.

**G3, sparse `GF(p)` row reduction — the hardest score lane, and it lost the oracle it was
promised.** It is 73–91 % of an F4 run, it converges over months, it needs a pinned machine,
and it is exactly the lane where two agents in parallel each optimize against a baseline the
other is moving. It also cannot be graded by `groebner_certified`, because that mode's
reducer is over ℚ and never executes a line of it (ADR-010 §5). Staff it with **one** agent,
with a frozen baseline, a change-point-tracked series, G2r as its internal oracle, and
external differential testing as its **primary** verdict.

**G5, modular Gröbner — everything it returns is `Probable`.** Its cross-check against G4
shares matrix construction, symbolic preprocessing and the whole monomial layer, so it is
materially weaker than the resultant lane's three-route agreement, and the verification
spine already grades it that way. It cannot return `Proved` at all until the
Idrees–Pfister–Steidel / Noro–Yokoyama question is settled. **External differential testing
is its primary verdict, not its secondary one** — an inversion of the normal rule that is
written into the lane brief.

**G7, FGLM — re-sized from L to XL, and the reason matters.** It was scoped as an ordinary
certificate lane on the strength of a sentence saying re-interning "is what FGLM does
anyway". FGLM does not do that: it walks monomials in lex order while computing normal forms
modulo the **drl** basis, so both orders are live on the same monomials for the whole
computation. The lane now delivers the dual-key pair ring *and* the linear algebra, and its
certificate is three clauses, not one (ADR-009, ADR-023 §5).

**H5 and benchmark calibration generally.** Deciding what counts as a regression is judgment
about noise. An agent will either set the threshold so tight the series flaps or so loose it
detects nothing. Calibrate per-series against measured run-to-run noise, once, by hand, and
commit the calibration.

**Anything requiring an e-graph adapter.** There is no verdict function for "is this seam
right", and ADR-017 §6 defers the whole rewriting surface until a consumer asks. Do not staff
it.

### Which are excellent agent targets

Z3 (`resolvent-modular`) — small, exhaustively certifiable, cleanly decomposable.
U1/U2 (`UPoly` arithmetic) — the naive reference is part of the deliverable, so the oracle
ships with the code. U4 (Sturm) — tiny, and it grades a much bigger lane. G2r (naive dense
reducer) — one session, and it is the only internal oracle the library's hottest lane will
ever have. K1 (`GF(p)` factorization) — the one factorization lane with a complete
certificate. K4 (LLL) — output conditions directly checkable. P1/P2 (monomials) — order
axioms are mechanical. A1–A5 (Layer 3) — the property suite is already written. X1/X3 (L4
container) — self-contained and blocked by nothing. All of the harness lanes. All of the
generator work. All of the mutant sets (ADR-023 §1), which are unusually good agent work:
small, adversarial, and with an unambiguous pass condition.

---

## 4. Where the long pole actually is

The source spec's own estimate is 12–24 months to be useful, and nothing in the research
contradicts it. What the research changes is *where usefulness arrives*: **at M4, not at M6.**
The geometry consumer needs none of Gröbner. That reordering is the single largest schedule
win available and it is free.

Beyond that, the honest assessment — in the order these will actually hurt.

**1. Score lanes, structurally, and there are eleven of them.** Certificate lanes converge in
days and are safe to fan out; score lanes converge over months, are non-monotone, have no
completion condition, and cannot be parallelized. Roughly a third of the lane count is score
lanes and they will be well over half the calendar. Every schedule estimate in §2 that omits
them is an estimate of the *cheap* half. This is the structural long pole and no amount of
fan-out touches it.

**2. G3 — sparse `GF(p)` row reduction.** One lane, one agent, 73–91 % of the flagship
benchmark, months of convergence, and now with a weaker oracle structure than the plan
originally believed. The published *Competitive* rung is only reachable with the SIMD leaf
(ADR-022), and even then it is ≈2× SOTA against implementations with a decade of tuning.

**3. The bignum reconstruction question, which may be a surprise.** ADR-002's original
argument — that modular methods make megabit integers irrelevant — is false: they
*concentrate* large integers into the CRT modulus and rational reconstruction, and
reconstruction is `gcd_ext` at 58–70 kbit on instances the corpus contains deliberately. The
one identified structural pure-Rust deficit (Lehmer vs half-GCD) sits exactly there, on the
**default certified path**. Lane Z2 now measures the regime that matters, and if the trigger
fires, a half-GCD lane lands in M1. If it fires and is ignored, it will surface as "Gröbner
over ℚ is inexplicably slow" six months later, which is the most expensive possible time to
learn it.

**4. T6b and K5 — two large lanes with the weakest verdicts in the library.** Both are
months, both need human specification, and both have failure modes that pass their own
checks. They are the reason M4 and M5 are marked "months" rather than "weeks" despite modest
session counts.

**5. The freeze itself, if ratification is slow.** Three global barriers must clear before
any trunk opens, and one of them is a human read of twenty-five ADRs. That is a day or two if
batched (see `NEXT.md` day 0) and a month if it is not.

**What is *not* the long pole**, despite looking like it: monomial packing (worth ~15 %, and
comparison largely disappears inside F4), and the multivariate trunk generally, which does not
gate the first consumer at all. A lane brief that says "optimize monomial comparison" buys
15 % and misses the divisor index's 10–20× and the S-pair criteria's four orders of magnitude.
Write the ranking into the brief or a week goes into the wrong 15 %.

### Effort, honestly

Sizes are estimates in *agent-sessions* — one focused run ending in a green gate — calibrated
against how much specification each lane has, not against measured throughput.

| Milestone | Certificate work | Score/measurement work | Human decisions | Realistic elapsed |
|---|---|---|---|---|
| M0 | ~11 sessions, 5 parallel | 3 lanes | — | days |
| M1 | ~17 sessions, 6 parallel | 2 score + 1 measurement | **The freeze. 25 ADRs.** | weeks — the ADRs are the long pole here, not the code |
| M2 | ~22 sessions, 4 parallel | 3 score, open-ended | — | weeks |
| M3 | ~25 sessions, 4 parallel | 1 score | the `Ord` ceiling, from Y1's measurement | weeks |
| M4 | ~38 sessions, 4 parallel | 2 score | multiplicity semantics; ADR-025 already pins the conventions | **months** — T6b dominates |
| M5 | ~32 sessions, 3 parallel | 1 score | — | **months** — K5 dominates |
| M6 | ~55 sessions, 3 parallel | 5 score, all open-ended | — | **many months.** The published gap between a real F4 (OpenF4) and the state of the art is 4–21× |
| M7 | ~15 sessions, 2–3 parallel | — | — | weeks; X1+X3 fully parallel, X2/X4 are not |
| M8 | ~35 sessions | 1 score | — | months; M8-N is a lane, not an instantiation |

---

## 5. Consumer value ladder

Consumers are context, not dependencies (founding constraint #1): each writes its own thin
adapter, and resolvent names none of them anywhere in a published crate.

| Milestone | Consumer class | What becomes possible |
|---|---|---|
| M1 | CAD constraint solving | Generic rank of a constraint Jacobian over `GF(p)` — evaluate at a seeded random point, row-reduce mod `p`. The one place exact algebra is not a new capability but a strictly better implementation of something that already ships: faster *and* more correct. It falls out of `resolvent-modular` plus a row-echelon returning rank, pivot rows, dependent rows **and the transform** — the same object as a Gröbner cofactor representation, one layer down |
| M2 | DAE integration | Certified event-crossing counts per integrator step: lift dense-output coefficients to ℚ, isolate under a budget, return a decline rather than hanging. ~70 lines of adapter. *Caveat: that integrator is not written yet* |
| M3 (v0.1) | exact 2-D arrangements | The Layer-3 API exists and four duplicated `Rc<RefCell<RealRoot>>` blocks can go. Degree still ≤ 4 in practice — this release proves the API rather than lifting the ceiling |
| **M4 (v0.2)** | **exact 2-D arrangements** | **The unlock.** Arbitrary-degree curves: one `Res_y` call replaces three hand-rolled resultants, the lossy double-squaring elimination and its spurious-root filter disappear, and ~150 lines/family of by-hand radical-ladder derivation collapses into `sign_radical_tower` at arbitrary depth |
| M3–M4 | exact solid modelling | The strongest early consumer: it needs resolvent **total, certified, and honest about refusal** — speed secondary — which makes it the best proving ground for the fail-closed and sharpness machinery. Its named refusal sites exist precisely because its substrate stops at degree-≤4 univariate ℚ |
| M5 | all of the above, and SMT | Intersection multiplicity beyond the parity heuristic; minimal polynomials, hence a defensible `canonicalize()`; the irrational-factor refusals close |
| M6 | exact medial axis | Trivariate elimination, ideal saturation, 0-dimensional solving |
| M7 | build-time symbolic differentiation, FEM | Forcing-term generation; Pantelides index reduction. Both currently blocked in code; both build-time, so bignum cost is free. **X1+X3 are independent of everything — staff them from the start** |
| M8 | SMT NRA | Full subresultant chains, principal subresultant coefficients, root isolation over ℚ(α₁..α_k), cofactor/certificate return for external proof production — **and M8-N, without which all of that is correct and too slow** |

The asymmetry that makes M4-before-M6 correct: the geometry consumer **never does
algebraic-number arithmetic** — its root type has no `add`, `mul` or `div` and four curve
families ship without them. SMT NRA requires it. That asymmetry is the single biggest scoping
win available and it is why the geometry trunk is short and the SMT trunk is long.

---

## 6. Risks and anti-goals

| Risk | Mitigation |
|---|---|
| **Scope declared but not delivered** | A README claiming a general-purpose CAS over an empty workspace is worse than a narrow claim. Mitigation: **"in scope" and "specified" are different states**, an unspecified capability's lane cannot open (ADR-021 §3), M9's table lists every unwritten ADR by name, and `README.md` §Status says plainly that no implementation exists. M4 is still the release that matters |
| **The conformance grade becomes an escape hatch** *(added 2026-08-08)* | A capability with an available self-certificate graded by oracle comparison instead. Mitigation is mechanical and in ADR-030 §2–§3: soundness is **never** conformance-graded, a conformance lane gates nothing and is an oracle for nothing, `self_certifying = false` is a required field, and a lane proposing the grade where an inverse operation exists is a review defect — the reviewer's question is "what is the inverse operation?" |
| **A domain-restricted rewrite rule fires on an undischarged side condition** *(added 2026-08-08)* | This is where essentially all real CAS unsoundness lives — `√(x²) → x`, `log(ab) → log a + log b` on the negative reals. Mitigation: ADR-033 §3's class D carries a machine-checkable side condition and **firing without discharging it is a bug, not a heuristic**; `RuleSet::ring_identities()` is class-R only; and `simplify` returns `Proved` only when every rule that actually fired was class R |
| **The exactness lattice is retrofitted rather than built in** *(added 2026-08-08)* | It is a field in the hash-consed node (ADR-031), so adding it after X1 ships is a rewrite of the `Store` and invalidates every golden file X3 committed. Mitigation: it is in M7's *Lands* list and in X1's brief, and if X1 has started when ADR-031 ratifies, X1 stops and is re-briefed |
| **Arbitrary-precision numeric evaluation enters through the analytic surface** *(added 2026-08-08)* | The single place M9 touches ADR-012 §6. `API.md` §4.2 admits special functions as *symbolic objects* and explicitly excludes their numeric evaluation; that capability has no ADR and its lane cannot open until one answers how a certified enclosure is produced and why it can never be a decision input |
| **Lanes start against unratified decisions** | Their crates are absent from the workspace (ADR-021 §3). Mechanical, not cultural |
| **Score lanes start before their oracle** | A score lane's CI job does not exist until its `oracle` list is green and frozen. Same mechanism |
| **Two documents specify two libraries** | ADRs are normative; the contradiction register is a table with twelve closed rows; a CI grep gate fails on divergent definitions of a headline type (ADR-021) |
| **A certificate accepts everything and every real bug is triaged as a convention** | Every certificate ships a mutant set and is observed rejecting a wrong answer (ADR-023). This was the largest hole in the plan |
| **Gate 0 grows until the determinism matrix is dropped** | The corpus is tiered on day 1, `fast` has a hard budget, and the tier census prints every run (ADR-024) |
| **Sharpness gates that are policies rather than numbers** | The ratchet: measured at API-landing time, lowering free, raising counted, `TBD` fails Gate 1 |
| **An agent optimizes monomial comparison** | It is worth 15 %. The divisor index is worth 10–20× and S-pair criteria four orders of magnitude. The ranking is in the lane brief |
| **Silent exponent overflow** | Guard bits in release; the narrow-field sweep as a **distribution assertion** at three widths, with a width at which nothing completes counted as a failure |
| **Parallel interning smuggles thread-arrival order into ids** | Content-derived ids plus INV-M1 (no tie-break consults id order) |
| **A shared refinement cache makes declines schedule-dependent** | INV-AR1, plus bound-derived budgets on every `AlgebraicReal` operation |
| **`Ord` hangs on a pathological pair** | `try_cmp` alongside, a diagnostic ceiling set from a measured distribution, and no `Equal` from bound exhaustion |
| **The bignum gap surfaces at M6 instead of M1** | Z2's ladder now reaches 256 kbit and includes reconstruction at Hexapod's modulus, with a pre-committed trigger |
| **Publishing a performance target the policy forbids reaching** | ADR-022: either the audited SIMD leaf, or the published rung drops to 3–4× SOTA in the same commit |
| **A benchmark generator transcribed from a GPL test suite** | Every family carries a Tier-A citation or is dropped (ADR-001 gate 5, ADR-016 §8) |
| **Consumer integration creeps back in** | resolvent depends on nothing local, imports no consumer traits, exposes no float interval type, takes no tolerance parameter anywhere, and owns no geometric concept. Grep gates L4 and L5 |
| **A tolerance parameter appears somewhere** | Treat it as a defect in review. The first consumer refuses tolerance by construction — its exact families declare `type Error = Infallible` — and a tolerance argument would make resolvent unusable by the consumer it exists for |
| **Reading the wrong source** | Tier C is a blocklist, not a preference. Every non-obvious module carries a `Derivation:` line citing a **paper and a committed research note**, and CI resolves the note's path |
| **Copied tuning thresholds** | Every threshold is re-derived by measurement on resolvent's own corpus, with the measurement committed. Simultaneously a licensing rule and a correctness rule, which is why it holds |

---

## Sources

Normative: `docs/decisions/ADR-001…025`, `API.md`. Working notes carried forward:
`plans/verification.md`, `plans/architecture.md`, `plans/roadmap.md`, `plans/api-shape.md`.
Research: `docs/research/prior-art-and-licensing.md`, `consumer-requirements.md`,
`algorithms-and-representation.md`, the three consumer evaluations, and the two adversarial
critiques `critique-engineering.md` and `critique-plan.md`, whose findings are folded into
the ADRs and into this document rather than left as a reading list. Every external citation
is carried from those, where it is sourced.
