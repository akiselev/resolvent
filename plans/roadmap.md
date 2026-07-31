# Roadmap and agent fan-out plan

Status: **plan, for ratification.** Lane D2 deliverable 2 of 2. Companion:
`plans/verification.md`, which defines every verdict function referenced here.

Inputs: `docs/research/prior-art-and-licensing.md` (R1),
`docs/research/consumer-requirements.md` (R2),
`docs/research/algorithms-and-representation.md` (R3),
`docs/research/consumer-sinbad.md` (E1), `docs/research/consumer-cadabra2.md` (E2),
`docs/research/consumer-solverang.md` (E3), `/home/dev/projects/IDEAS-crates.md` §4.
Ratified decisions: `docs/decisions/ADR-001…009`; declared decisions:
`plans/architecture.md`'s ADR table; API surface: `plans/api-shape.md`.

This document exists to answer one question: **what can be worked in parallel by
separate agents, and what cannot.** Everything else here — milestones, exit gates,
effort estimates — is in service of that answer.

---

## 0. How to read this

**Lane grades** (defined in `plans/verification.md` §0):

- **`certificate-graded`** — CERT/INV/PROP primary verdict. Converges in days.
  Monotone. **Safe to fan out aggressively.** A green gate means done.
- **`score-graded`** — the success criterion is a *number to optimize*, not a
  certificate to check. **Converges over months. Non-monotone. No completion
  condition.** Requires a pinned machine and a frozen reference implementation to
  regress against. **Unsafe to fan out** — parallel agents on the same score lane
  thrash against each other's baselines.
- **`decision`** — the deliverable is a ratified ADR, not code. **Not an agent lane.**
  Judgment, tradeoffs, and irreversibility; no verdict function exists.

**The freeze rule.** Founding constraint #5: representation is a one-way door. Packed
exponent vectors and modular-methods-everywhere are structural, not retrofittable. No
Layer-2 lane starts before the ADRs it inherits are ratified. §2 says exactly which ADR
gates which lane, so "the freeze" is not one event but a *dependency edge per lane*.

**The sequencing gift.** R3 §2.5: the first consumer touches none of the multivariate
machinery. Its `QPoly` is a dense `Vec<Rational>`
(`arrangements/crates/lazy-exact/src/roots.rs:41-45`), its resultants are hand-rolled
2×2 conic determinants (`conics.rs:272-287`) and a double-squaring hack
(`sine_radical.rs:614`), and its `RealRoot` is exactly `AlgebraicReal`
(`roots.rs:317-322`). So **after the shared ADRs land there are two trunks, not one**,
and the consumer-unblocking trunk never waits on the Gröbner one-way doors — *provided*
`UPoly<C>` is defined standalone and first, and `MPoly` converts down to it
rather than the reverse. That is a cheap decision now and an expensive one later.

---

## 1. Milestones

Each milestone states what lands, what it unlocks, its exit gate as a checkable
condition, and its dependencies. Exit gates are conditions a CI job evaluates, not
judgements.

### M0 — The harness (no algebra)

**Lands.** Workspace with the two-category rule (`publish = true` crates gated by
`cargo-deny`; `publish = false` crates may carry LGPL dev-deps and shell out to GPL
binaries). CI Gate 0. License gate plus its three-case regression corpus. Determinism
and canonical-bytes harness. Corpus format, generator interface, seed schedule,
minimizer, score reporter. Tier-0 sympy oracle adapter with the S-expression protocol
and the triage classifier. Benchmark runner skeleton with change-point reporting.

**Unlocks.** Every subsequent lane. Nothing can be graded before this exists.

**Exit gate.**
- Gate 0 green on a workspace containing no algebra.
- The license gate **fails** on all three planted cases (`malachite`, `polynomen`, a
  synthetic Apache-only crate depending on `rug`). A gate that passes what it must
  reject is not a gate.
- The score harness runs end-to-end against a deliberately-buggy stub and reports a
  falsification within budget; and reports the budget as survived when the stub is
  fixed.
- The sympy adapter round-trips a polynomial through the S-expression protocol and back.
- The minimizer reduces a planted 20-term counterexample to its minimal form
  automatically.

**Depends on.** Nothing.

---

### M1 — Layer 0 and the representation freeze

**Lands.** Two halves that must both complete.

*Code:* `resolvent-int` (the `Integer`/`Rational` newtype wall over `dashu`, with `dashu`
in no public signature and no re-export); `resolvent-modular` (`Fp` for word primes with
Barrett/Shoup and a benchmarked Montgomery path, `Zn`, `GF(p^k)`, bulk vector ops
first-class, the batched tuple ring); CRT, rational reconstruction, deterministic prime
selection; the `Certificate` type; the budget/decline error taxonomy.

*Decisions (ADRs, owned by lane D1):* the list in §2. This is the freeze.

*Measurements that must be committed before the freeze:* `bigint-benchmark-rs` re-run
with `dashu` 0.5.2 pinned (every published figure used 0.4.2, one release **before** NTT
landed in 0.4.3, so the widely-cited number is stale); and `gcd`/`gcd_ext` at 64 / 256 /
1k / 4k / 16k bits against `rug`, because Lehmer-vs-half-GCD is the one identified
structural pure-Rust deficit and it sets how aggressive the ℤ-primitive discipline must
be.

**Unlocks.** Both trunks. Nothing above Layer 0 may start before the ADRs it inherits
are ratified.

**Exit gate.**
- `Fp` exhaustively certified against `i128` for every prime `p < 2^10`; random-certified
  for `p < 2^63`; bulk path componentwise-equal to the scalar path including tails and
  misalignment.
- Batched tuple ring componentwise-equal to N independent scalar runs.
- CRT and rational reconstruction certificates 100% `Proved` on the fleet, including the
  **Hexapod** reconstruction-bound instance (1102 primes for a 0.00 s modular run — the
  instance that finds CRT and reconstruction bugs).
- Prime generation deterministic and reproducible across runs, processes, and thread
  counts.
- `dashu` measurement notes committed to `docs/research/`.
- **Every ADR in §2's "gates everything" and at least one trunk's list is ratified and
  merged in `docs/decisions/`.**

**Depends on.** M0.

---

### M2 — The univariate engine over ℤ

**Lands.** `UPoly<C>` standalone (`Vec<C>`, no monomial type, no order) with
content/primitive part, canonical associate normalization, Horner evaluation,
derivative, `divrem`, pseudo-division, **public** reciprocal transform `xⁿ·p(1/x)`
(private in the prior art at `roots.rs:242`, and consumers need it to move ∞ to 0);
modular gcd with the two-part certificate; `gcd_ext`; Yun square-free decomposition;
Sturm sequences **as an oracle**; Descartes/VCA isolation over ℤ on dyadic intervals.

**Unlocks.** Layer 3. Also sinbad's `solverang` event-detection shape (consumer-sinbad §1.3) — dense
output polynomial coefficients lifted from `f64` to ℚ, isolated with a budget.

**Exit gate.**
- `count_sturm(f,a,b) == len(isolate_descartes(f,a,b))` on the entire generator fleet
  including Mignotte instances up to the degree where Sturm remains affordable.
- gcd certificate 100% `Proved`: `H|A`, `H|B`, and `deg H == deg gcd(A mod p, B mod p)`
  for a certified-good prime. **The degree half is mandatory** — divisibility alone
  accepts any common divisor.
- Yun: `Π fᵢ^i == f`, factors pairwise coprime, each square-free.
- Isolation invariants: disjoint, ordered, `f(lo) ≠ 0 ≠ f(hi)`, Descartes variation
  exactly 1 per interval, all within the Cauchy bound, multiplicities summing correctly,
  round-trip from constructed roots.
- Tier-0 differential green on gcd, square-free, and isolation with §4.3 normalization.
- Isolation "Correct" threshold: degree ≤ 20 random and Mignotte instances verified
  against Sturm counts.

**Depends on.** M1 shared ADRs (§2.1: 002, 004, 006, 007, 010, 011, 012).

---

### M3 — Layer 3: algebraic numbers — **v0.1**

**Lands.** `AlgebraicReal` (square-free defining polynomial over ℤ, isolating rational
interval) with `refine`, total `cmp` with equality decided by gcd plus a sign-change
certificate, `cmp_rational`, exact `sign_of(P)`, and an outward-correct `(f64, f64)`
enclosure pair — **no float interval type in the public API**. `SqrtExt`-equivalent
`a + b√r` as a first-class type, not subsumed. Radical-tower sign at arbitrary depth,
generalizing the depth-1 and depth-2 ladders. Separation bounds (Mignotte/Davenport
class), converting "terminates eventually" into "terminates in a computable number of
steps". `rational_between`. Bernstein/de Casteljau certified range enclosure.

**Unlocks.** The consumer's whole predicate surface *at degree ≤ 4*, which is a
usable-but-not-yet-interesting release. Ship it as v0.1 anyway: it makes the API real,
forces the adapter question, and gives the score lanes a public baseline.

**Exit gate.**
- All eleven Layer-3 properties in `plans/verification.md` §2.6 green under an explicit
  **step budget**, with budget exhaustion counted as failure. Transitivity, sort
  stability, and the no-hang budget are the three that matter.
- Radical-tower sign agrees with the materialized `AlgebraicReal` route on the whole
  fleet (the strongest free internal oracle in this layer).
- `SqrtExt` sign-by-squaring agrees with the general route; cross-root comparison total.
- Separation bound validity: every verdict reached under the bound equals the verdict
  reached by unbounded refinement.
- Bernstein: soundness certificate green **and** the Unknown-rate ceiling met (a
  fail-closed API that always says `Unknown` passes soundness and is worthless).
- Constructive generators present for equal-value/different-representation pairs,
  deliberately-close triples, overlap-endpoint-on-a-root pairs, and `sign_of` at zero.
  Random generation finds none of these.
- Zero panics and zero unbounded runs across the fuzz targets.

**Depends on.** M2; ADRs 013, 014, 015 (§2.2) plus the verdict-vocabulary and `SqrtExt` gaps in §2.4. **ADR-013 is contradicted and must be resolved by the §2.5 experiment before A1 starts.**

---

### M4 — Elimination — **v0.2, the consumer unlock**

**Lands.** `RecursiveView` as a borrowed view; subresultant PRS (Ducos) over ℤ; modular
evaluation–interpolation resultant for the bivariate case; a Bareiss/Bézout determinant
route as a third independent implementation; `Res_y(f,g)` for general bivariate `f,g`
with correct multiplicity semantics and no spurious roots; bivariate gcd and
common-component detection; curve analysis (critical abscissas, per-interval branch
counts, branch-index-to-root maps) with **no geometric type in any signature**; cheap
sharing of analysis results.

**Unlocks.** This is the real thing. Arbitrary-degree curves for the geometry consumer:
three hand-rolled resultants collapse into one call, the lossy double-squaring
elimination and its spurious-root filter disappear, and the degree-4 ceiling lifts.

**Exit gate.**
- Three resultant routes agree on the whole fleet. Route independence audited: the
  Bareiss/Bézout route shares only `Integer` arithmetic with the other two and is worth
  building for exactly that reason.
- Resultant cofactors: `u·f + v·g == Res` exactly. Degree bound
  `deg_x Res_y ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)` holds on every output.
- `Res == 0 ⇔ deg gcd(f,g) > 0`, cross-checked against the M2 gcd lane. Identically-zero
  resultants are returned as a **distinguishable** result, never a silently-empty root
  list.
- Subresultant chain specialization property holds at random good primes and evaluation
  points.
- Curve analysis agrees with the independent rational-witness route (isolate the roots of
  `f(α,y)` at a rational abscissa strictly inside each interval) on the whole corpus.
  Branch counts consistent across adjacent intervals; branch matching a bijection.
- Tier-1 differential (PARI `polresultant`, sympy `subresultants` for the whole chain)
  green with §4.3 normalization — including the explicit `(−1)^(mn)` argument-swap sign
  rule, not a blanket "up to sign".
- **Score lane, tracked separately:** modular bivariate resultant ≥ 100× the Ducos route
  on ℤ[x,y] degree ~20.

**Depends on.** M2, M3; the shared ADRs. Does **not** depend on the monomial layer being
implemented — only on a bivariate representation, which `RecursiveView` over a
two-variable `MPoly` or a dedicated bivariate type can provide. Which of those it
is, is an ADR.

---

### M5 — Factorization over ℤ

**Lands.** Cantor–Zassenhaus / Berlekamp over `GF(p)`; Hensel lifting to the
Landau–Mignotte bound; Zassenhaus recombination with an explicit `r` threshold (~10, i.e.
≤1024 subsets); LLL; van Hoeij lattice recombination above the threshold.

**Unlocks.** Intersection multiplicity beyond the parity heuristic; minimal polynomials,
hence a defensible `canonicalize()` and `Hash` for `AlgebraicReal`; SMT NRA projection-set
control.

**Exit gate.**
- `GF(p)` factorization: multiply-back **and** the complete irreducibility test per
  factor. Over a finite field both halves are decidable and cheap; this is the one place
  factorization is fully certified.
- Over ℤ: multiply-back on every instance; the modular irreducibility certificate
  wherever it exists, with its **success rate tracked** — a falling rate means the
  implementation got coarser or the corpus got harder, and both need a look.
- Zassenhaus and van Hoeij agree for `r ≤ 20`.
- **Swinnerton–Dyer ladder:** degree 32 (`r ≈ 16`) completes under Zassenhaus. Degree 64
  (`r ≈ 32`) separates van Hoeij from Zassenhaus. Degree 256 is the "van Hoeij is really
  working" mark. A single Swinnerton–Dyer instance is worth more than a thousand random
  ones, because it is irreducible, has no modular irreducibility certificate at any
  prime, and a coarse implementation returns a nontrivial factorization.
- LLL output satisfies the Lovász and size-reduction conditions, with a unimodular
  transform.

**Depends on.** M2 (gcd, square-free), M1 (`GF(p^k)`).

---

### M6 — Multivariate and Gröbner

**Lands.** The monomial layer (packed order-normalized comparison key, raw exponents,
divmask, interning arena with a multiplicative hash, guard-bit overflow detection,
widen-and-restart driver); `MPoly` with heap-based multiply/divide;
Buchberger + Gebauer–Möller **as the oracle**; the divisor-query index; F4 matrix
construction and sparse `GF(p)` row reduction; modular Gröbner with tracing, majority
vote and stabilization; `groebner_certified` with cofactor retention; FGLM.

**Unlocks.** Exact medial axis (#27) and anything needing ideal theory. **Not** the
geometry consumer, which needs none of it.

**Exit gate — staged, because this milestone has three different verdict types.**

| Stage | Gate |
|---|---|
| **Monomial layer** | Order axioms including multiplicative compatibility; agreement with a naive `Vec<u32>` comparator per order; encode/decode round-trip **at and past the capacity boundary**; overflow always detected, never wrapped; widen-and-restart produces the same answer as starting wide |
| **Overflow sweep** | The entire Gröbner corpus re-run at 4-bit exponent fields: every instance either matches the wide run or reports overflow. **Zero silent divergences.** Wraparound yields a correct basis of a *different ideal* and every other certificate passes, so this sweep is the only detector |
| **Buchberger (oracle)** | Correct on Cyclic-7, Katsura-8, Eco-10; agrees with Singular; cofactor certificate for both ideal inclusions |
| **F4 correctness** | Agrees with Buchberger on every instance Buchberger can reach; agrees with `groebner_certified` |
| **FGLM** | Lex basis reduces the drl basis to zero and vice versa; same dimension and degree |
| **F4 performance** (score) | *Working*: Cyclic-8 < 60 s, Katsura-11 < 500 s, Eco-13 < 500 s. *Competitive*: Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s. **Do not plan for state of the art** (within 1.5× of msolve/Maple/Groebner.jl) |
| **Modular over ℚ** | Katsura-10/11, Cyclic-8, Chandra-13, Reimer-8 complete; Hexapod completes; `groebner` agrees with `groebner_certified` on every regression instance |

**Depends on.** M1 multivariate ADRs (P, Q, R, S, T), M1 Layer 0, M2 for the coefficient
machinery. **Buchberger must be green and frozen before F4 starts** — that is a CI-
enforced edge, not a suggestion.

---

### M7 — Layer 4: the expression DAG (parallel lane, starts at M1)

**Lands.** Hash-consed DAG over an owned `Store` (never a thread-local or `static`, which
would break determinism and content addressing); node set
`{ Const(L0 element), Symbol(interned), ring ops, Apply(FuncId, args) }` with a `FuncId`
table carrying arity and a derivative rule; `diff` **and `diff_with(expr, sym, leaf_rule)`**;
constant folding; `walk_topological` with stable ids; `is_polynomial_in(&syms) -> Option<MPoly>`
as the bridge down to Layer 1; canonical bytes with a schema version. **No code emitter.**
**No transcendental zero-test, at any layer, ever.**

**Unlocks.** sinbad's MMS forcing generation and plexus's Pantelides index reduction —
both build-time, both currently blocked (`plexus/src/index_reduction.rs:3-6` is an
identity pass explicitly waiting on symbolic `d/dt`). FEM form compilation (#34).

**Exit gate.**
- On the polynomial subset, `diff` equals `UPoly::derivative` **exactly** — an exact
  cross-layer oracle covering chain, product, and power rules.
- Hash-consing injective; canonical bytes byte-identical across insertion orders, thread
  counts, processes, and feature combinations, with golden files.
- `is_polynomial_in` sound in both directions.
- The two adapter sketches in consumer-sinbad §4.1 and §4.2 compile and run against the real API in
  under 200 lines each. That is the acceptance test and it is a real one: `diff_with`'s
  leaf-rule callback is the difference between the plexus adapter existing and not.

**Depends on.** M1 Layer 0, ADRs 010/012/017, and the L4 node-set gap in §2.4. **Independent of M2–M6.** This is the
cleanest parallel trunk in the plan and it should be staffed from the beginning, because
it serves a different consumer with a different latency class (build-time, where bignum
cost is free) and it cannot block or be blocked.

---

### M8 — Towers, CAD support, and 0-dimensional solving

**Lands.** `UPoly<NumberField>` as an added instantiation (not a rewrite, because
`UPoly<R>` was generic from day zero); subresultant chains and principal subresultant
coefficients as first-class returned data; ideal saturation; RUR or triangular
decomposition for 0-dimensional real solving; cofactor/certificate return plumbed through
gcd, resultant and Gröbner as a *public* output.

**Unlocks.** SMT NRA (#12) and exact medial axis (#27).

**Exit gate.** Subresultant chain matches sympy's `subresultants` after the pinned scalar
convention; root isolation over `ℚ(α₁..α_k)` passes the full Layer-3 property suite with
tower coefficients; saturation cross-checked against a Rabinowitsch-trick computation in
one extra variable.

**Depends on.** M4, M5, M6.

---

### Milestone dependency graph

```
M0 harness
 └─ M1 Layer 0 + FREEZE ──────────────────────────────────┐
     ├─ (univariate trunk)                                │
     │   M2 univariate over ℤ                             │
     │    ├─ M3 algebraic numbers  ── v0.1                │
     │    │    └─ M4 elimination   ── v0.2  ← the unlock  │
     │    └─ M5 factorization ───────────────┐            │
     │                                       │            │
     ├─ (multivariate trunk)                 │            │
     │   M6 monomials → Buchberger → F4 → modular Gröbner │
     │                                       │            │
     ├─ (expression trunk, fully parallel)   │            │
     │   M7 Layer 4 DAG ──────────────────────────────────┘
     │
     └─ M8 towers / CAD / RUR   ← needs M4 + M5 + M6
```

---

## 2. The freeze: which ADR gates which trunk

Lane **D1 owns these ADRs.** D2 owns only the statement of which lanes each one gates,
and the flagging of gaps and contradictions. As of writing, ADR-001…009 exist as files in
`docs/decisions/`; ADR-010…018 are declared in `plans/architecture.md`'s ADR table but not
yet written; and the five decisions in §2.4 have no ADR at all yet. **A lane may not start
against a declared-but-unwritten ADR** — that is the difference between a freeze and an
intention.

### 2.1 Gates everything — no lane above Layer 0 starts before these

| ADR | Decision | Status |
|---|---|---|
| **ADR-001** | License posture: Tier A / B / C reading discipline; mechanical `cargo-deny` gate | ratified |
| **ADR-002** | `dashu` behind the `resolvent-int` newtype wall; no re-export | ratified |
| **ADR-003** | Hand-roll `resolvent-modular`; reject `ark-ff`, `crypto-bigint`, `num-modular` | ratified |
| **ADR-004** | ℤ-primitive, not ℚ-primitive; ℚ is a boundary façade | ratified |
| **ADR-005** | Workspace crate split; seven published + three unpublished, lockstep versioned | ratified |
| **ADR-006** | Generics cross crate boundaries, never inner loops; closed instantiation set; **`LANES` kept open** — this is the tuple-ring/batched-multi-modular door (R3 §3.5) | ratified |
| **ADR-007** | Three representations; **`UPoly<C>` defined first and standalone** — the decision that makes the two-trunk fan-out possible | ratified |
| **ADR-010** | Modular everywhere; `Certified<T>` with `Certainty::{Proved, Probable}`; two Gröbner modes | **declared, unwritten** |
| **ADR-011** | Fail at construction not at query; no panics; structured `Unsupported`; step budgets | **declared, unwritten** |
| **ADR-012** | Counter-based seeded RNG; index-addressed primes; ordered combination; replayable traces | **declared, unwritten** |
| **ADR-015** | No float interval in the public API; rational bounds + outward `(f64, f64)` | **declared, unwritten** |
| **ADR-016** | Subprocess-only oracles; two-category workspace; no exception process | **declared, unwritten** |

ADR-011 and ADR-012 gate the *harness itself*, not just the algebra: `plans/verification.md`
§1.2 and §1.3 derive the budget/decline contract and the determinism contract from them, and
§3.10's cross-run byte-comparison is unimplementable until ADR-012 fixes the RNG and prime
addressing. **Write ADR-011 and ADR-012 first among the unwritten set.**

### 2.2 Gates the univariate / Layer-3 trunk only

| ADR | Decision | Status |
|---|---|---|
| **ADR-013** | `AlgebraicReal` mutability and thread-safety | **declared, unwritten, and contradicted — see §2.5** |
| **ADR-014** | No `Hash`, no general arithmetic; `canonicalize()` opt-in; multiplicity is not a field of the number | **declared, unwritten** |
| **ADR-006/007** | `UPoly<R>` generic from day zero, so `UPoly<NumberField>` is an added instantiation rather than a rewrite; `AlgebraicReal`'s coefficient domain is ℤ-only *now* | ratified (the coefficient-domain half should be restated explicitly in ADR-013 or ADR-014) |

### 2.3 Gates the multivariate trunk only

| ADR | Decision | Status |
|---|---|---|
| **ADR-008** | Interned arena + packed key + divmask; guard-bit overflow detection; widen-and-restart. Marked *one-way (interning), cheap (field width)* | **ratified, and contradicted — see §2.5** |
| **ADR-009** | Order is runtime ring data, normalized into the comparison key at intern time | ratified |
| **ADR-010** | Two Gröbner modes | declared, unwritten |

### 2.4 Decisions with no ADR yet — D2 is flagging these as gaps

None of these blocks Wave 0 or Wave 1, but each blocks a specific later lane, and none is
covered by ADR-001…018 as declared.

| Gap | Blocks | Why it needs a decision, not a default |
|---|---|---|
| **The fail-closed verdict vocabulary.** Where `Uncertain<Sign>` ends and `Sign` begins. If Bernstein range-bounding returns `Uncertain<Sign>` while `AlgebraicReal` returns `Sign`, consumers get two verdict vocabularies that silently disagree at the adapter boundary | A5, and every consumer adapter | R2 open question 8 says explicitly this needs a decision doc, not an experiment, and it must land *before* the API is written |
| **`SqrtExt` is first-class and not subsumed** by defining-polynomial + interval machinery | A2 | `circle_segments.rs` (931 LOC) uses `SqrtExt` exclusively and never imports `RealRoot` or `QPoly`. Routing degree-2 radicals through the general machinery would be a large, silent performance regression, and nothing in ADR-001…018 forbids it |
| **grevlex + FGLM, never direct lex** | G1, G7 | Computing a lex basis directly on a system where drl+FGLM takes seconds routinely does not terminate. A lex Gröbner path is not a Gröbner lane; it is `drl-GB → FGLM`, and FGLM is its own lane with its own certificate |
| **The Gröbner row representation supports an optional cofactor block** sharing the same reduction code | G2, G4 | `plans/verification.md` §1.1. Two separate reduction implementations would defeat the certified-vs-fast cross-check that is the fast mode's only internal oracle |
| **L4's node set includes `Apply(FuncId, args)` with derivative rules and a `diff_with` leaf callback** | X1, X2 | Without `diff_with`, the plexus adapter cannot express `d/dt` of an implicitly time-dependent unknown and must reimplement the chain rule — the sketch in consumer-sinbad §4.2 fails. ADR-017 covers the e-graph seam but not the node set |

### 2.5 Two live contradictions inside D1's own output

Both are on decisions marked **one-way**. Neither can be resolved by reading more; each
needs a named experiment. Flagging them is the highest-value thing this section does.

**Contradiction 1 — `AlgebraicReal` mutability (ADR-013).**

| Source | Says |
|---|---|
| `plans/architecture.md` ADR table, and `architecture.md:589-591` | `Arc<Inner>`, `&self` monotone refinement, **`Send + Sync`**, total `Ord` via separation bound |
| `plans/api-shape.md` §1.3 (written later) | `poly: Arc<..>` + **`cache: RefCell<Isolation>` inline**, `Send + !Sync`, self-comparison guarded by `std::ptr::eq` inside resolvent, cheap `Clone` for per-thread copies |

These are materially different types with different consumer contracts: one is
`BTreeMap`-able across threads, the other is not. The api-shape version's argument is
strong — the guard is exactly correct precisely *because* the cache is inline rather than
`Rc`-shared, which is what makes address equality mean value identity — but it gives up
`Sync` on the headline type, and R2 D4's motivating evidence was a consumer drowning in
shared-refinement boilerplate.

Note also that both differ from R3 §8.2 F6's recommendation (explicit context
`ctx.cmp(&a,&b)`), and that R2 D4's lock-free monotone `Arc<Inner>` is a *fourth* option
R3's table does not enumerate: the read path never blocks and a stale read is merely a
wider valid enclosure, so there is no self-comparison hazard and no atomic per compare.

**Deciding experiment, before M3 (this is lane A1's blocker):** prototype all four behind
one trait — pure recompute, lock-free monotone `Arc<Inner>`, inline `RefCell` + `ptr_eq`
guard, explicit context. Grade each on: `cmp(&a, &a)`; sort stability under shuffling;
the transitivity suite; `Send`/`Sync` compilation; and the cost that actually decides it,
sorting `n = 10³` algebraic numbers of degree 8 with 200-bit coefficients. Record the
measurement in ADR-013. **It is visible in every signature and cannot be deferred.**

**Contradiction 2 — interned monomials vs inline packed keys (ADR-008).**

| Source | Says |
|---|---|
| `docs/decisions/ADR-008`, per the architecture ADR table | "Interned arena + packed key + divmask", marked **one-way (interning)** |
| `plans/api-shape.md` L1-4 (written later) | "**No global monomial interner.** Packed exponent keys live inline in the term… Terms are `(PackedMon, C)`… `MPoly` stays a self-contained `Send + Sync` value" |

This is a contradiction on the decision R3 §1.6 identifies as **the actual one-way door**:
not compare speed, but whether everything above inherits `(MonomialId, Coeff)` into a
shared arena or `(PackedMon, Coeff)` inline. Interning is what makes the divisor-query
index and the multiplicative hash `h(u)+h(v) = h(uv)` possible — worth 10–20× in the
reduction path against bit-packing's 15%. Inline keys are what make `MPoly` a
self-contained `Send + Sync` value with no ambient state, which is api-shape's INV-1 and
is downstream of the determinism contract.

They may be reconcilable: "no *global* interner" does not forbid an arena owned by a ring
context passed explicitly, which is ambient-state-free and still gives interning. If that
reading is intended, ADR-008 and api-shape L1-4 must say so in the same words, because as
written they specify different term types.

**Deciding experiment, before Wave 2's multivariate trunk (lane P2's blocker):** R3's own
open question — does the interned design defeat its own comparison key, since comparing by
id requires a random arena load whose cache miss may dominate the `u64` compare? Microbench
(a) inline packed monomials in terms against (b) ids plus arena lookup, on a realistic
S-pair queue workload, and separately measure the divisor-query index's speedup under each.
**Do not start P1/P2/P3 until this resolves**, because it changes the term type and
therefore every signature above it.

### 2.6 Background on contradiction 1: the underlying research disagreement

**R2 D4** recommends `AlgebraicReal = Arc<Inner>` with `&self` methods, `Send + Sync`, and
monotone refinement, copying the concurrency protocol of `lazy-exact::Real` where even a
torn read of the cached interval is a valid enclosure. Motivation: the shipping consumer
carries **four independent copies** of `type SharedRoot = Rc<RefCell<RealRoot>>` plus a
`Rc::ptr_eq` self-deadlock guard, purely because `refine` takes `&mut self` while every
`Geometry` predicate takes `&self`.

**R3 §8.2 F6** recommends the opposite primary API: an explicit context,
`ctx.cmp(&a, &b)`, with refinement state in a side table and a pure `Ord`-implementing
wrapper for callers who accept the recompute cost. Motivation: CGAL explicitly chose the
pure route and documented why ("this would impose a state to every object of an Algebraic
kernel"); R3's table rates every interior-mutability exit as either `!Sync` (panics on
self-comparison) or lock-based (deadlocks on self-comparison, pays an atomic per
comparison).

**These are not quite the same axis, and the synthesis matters.** R2's proposal is not
the lock-based row of R3's table: it is a *lock-free monotone* refinement cache, where
the read path never blocks and a stale read is merely a wider valid enclosure. That is a
fourth option R3's table does not enumerate, and if it works it dominates — it is
`Send + Sync`, it has no self-comparison hazard on the read path, and it preserves the
refinement cache that makes sorting `n` algebraic numbers affordable.

The four candidates are therefore: pure recompute (CGAL's choice), lock-free monotone
`Arc<Inner>` (R2 D4), inline `RefCell` + address guard (api-shape §1.3), and explicit
context (R3 F6). §2.5 states the experiment that ranks them.

---

## 3. The fan-out plan

This is the point of the document.

Legend: **grade** per §0. **Wave** is the earliest wave a lane may start. **Par** is how
many agents can usefully work the lane simultaneously. **Size** in agent-sessions
(S = 1–2, M = 3–6, L = 7–15, XL = 16+); score-graded lanes get **no completion estimate**
by definition.

### Wave 0 — before any freeze (no ADR dependencies)

| Lane | What | Grade | Par | Size | Verdict function |
|---|---|---|---|---|---|
| **H1** | Workspace, two-category rule, Gate 0, `cargo-deny` + `cargo-about` | certificate-graded | 1 | S | Gate fails on all three planted cases |
| **H2** | Determinism + canonical-bytes harness, golden-file machinery | certificate-graded | 1 | S | Byte-identical across runs/processes/threads/features |
| **H3** | Corpus format, generator interface, seed schedule, minimizer, score reporter | certificate-graded | 1 | M | Falsifies a planted stub; minimizes a planted 20-term case |
| **H4** | Tier-0 sympy adapter, S-expression protocol, triage classifier | certificate-graded | 1 | M | Round-trip; correct Class A/B classification on planted disagreements |
| **H5** | Benchmark runner, pinned-machine protocol, change-point reporter | score-graded (infra) | 1 | M | Detects a planted level shift; does not flap on measured noise |
| **Z2** | `dashu` 0.5.2 measurement: `bigint-benchmark-rs` re-run + `gcd`/`gcd_ext` ladder vs `rug` | score-graded (measurement) | 1 | S | Numbers committed to `docs/research/` |
| **Y1** | Consumer-workload measurement: degree 3–8 curve pairs, `Res_y` degree and coefficient bit-length, `isolate_roots` + `sign_of` wall time against the existing `QPoly` | score-graded (measurement) | 1 | S | Numbers committed. **Every geometry performance requirement currently rests on a guess and this removes it** |

Wave 0 is 5–7 lanes of genuinely independent work with no shared state. Fan out fully.

### Wave 1 — Layer 0, after the §2.1 ADRs (including 010, 011, 012 being written)

| Lane | What | Grade | Par | Size | Verdict function |
|---|---|---|---|---|---|
| **Z1** | `resolvent-int` newtype wall; `Integer`/`Rational`; `from_f64` fail-closed; bit-length and rounding accessors | certificate-graded | 1 | M | Ring/field axioms; `rug` oracle; round-trip |
| **Z3** | `resolvent-modular`: `Fp` Barrett/Shoup + Montgomery, `Zn`, `GF(p^k)` | certificate-graded | **2–3** | M | Exhaustive small-`p` vs `i128`; field axioms; Frobenius closure |
| **Z4** | Bulk/vector `GF(p)` ops (correctness only) | certificate-graded | 1 | S | Componentwise equality with the certified scalar path |
| **Z5** | Batched tuple ring `Zp4` (correctness only) | certificate-graded | 1 | S | Componentwise equality with N scalar runs — a free complete oracle |
| **Z6** | CRT, rational reconstruction, deterministic prime selection, good-prime predicates | certificate-graded | 1 | M | Congruence + range certificates; Hexapod |
| **Z7** | `Certificate` type, error taxonomy, budget plumbing | certificate-graded | 1 | S | Compile-level; plus a decline test per entry point |
| **Z8** | `Fp` scalar throughput; Barrett vs Montgomery decision by measurement | **score-graded** | 1 | — | Throughput on the pinned machine. **Blocked on Z3 being frozen** |

Z3 is the ideal agent lane in the whole project: small, extremely well specified,
exhaustively certifiable, and it decomposes cleanly into `Fp` / `Zn` / `GF(p^k)`.

### Wave 2 — the two trunks open

**Univariate trunk** (after ADRs H, K–O):

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **U1** | `UPoly<C>` arithmetic, content/primitive, associate normalization, `divrem`, pseudo-division, reciprocal transform | certificate-graded | **2** | M | Naive `O(n²)` reference is part of the deliverable |
| **U2** | Taylor shift, evaluation, Horner, derivative | certificate-graded | 1 | S | Round-trip + evaluation homomorphism |
| **U3** | Modular gcd + `gcd_ext` + Yun square-free | certificate-graded | 1 | M | Both halves of the gcd certificate; the degree half is the one people forget |
| **U4** | **Sturm — as an oracle** | certificate-graded | 1 | S | Exact distinct-root count. Will never be the production isolator. **Build it because it grades U5** |
| **U5** | Descartes/VCA over ℤ on dyadic intervals | certificate-graded | 1 | M | Graded by U4. Dyadic endpoints are the whole constant factor: `x→x+1` and `x→2^k x` stay integral, where arbitrary rational endpoints multiply denominators every subdivision |
| **U6** | Separation bounds | certificate-graded | 1 | S | Validity + a tracked tightness distribution |
| **U7** | ANewDsc Newton acceleration | **score-graded** | 1 | — | **Blocked on U5 frozen.** Mignotte ladder. Re-derive U5's interval invariants for the Newton path first — they were written for bisection and Newton steps jump |
| **U8** | Abbott QIR refinement | **score-graded** | 1 | — | Blocked on M3's `AlgebraicReal` |
| **U9** | Fast Taylor shift (middle-product/FFT) | **score-graded** | 1 | — | Crossover is around degree 512; below that, do not bother |

**Multivariate trunk** (after ADRs 008/009 and **the §2.5 contradiction-2 experiment**) — may run fully concurrently with the above:

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **P1** | Monomial encode/decode/compare/multiply, guard-bit overflow, widen-and-restart driver | certificate-graded | 1 | M | Order axioms + naive comparator + capacity-boundary generators |
| **P2** | Term type + `divmask`: either an explicitly-owned interning arena with a multiplicative hash, or inline packed keys — **blocked on the §2.5 contradiction-2 experiment** | certificate-graded | 1 | M | Injectivity, hash multiplicativity `h(u)+h(v)=h(uv)`, one-sided divmask soundness. The *verdict* is the same either way; the *term type* is not |
| **P3** | `MPoly` arithmetic with heap-based multiply/divide | certificate-graded | 1 | M | Representation invariants + naive reference |
| **P4** | Kronecker substitution utility | certificate-graded | 1 | S | Round-trip |
| **P5** | Divisor-query index (kd-tree or equivalent) | **score-graded** | 1 | — | Worth 10–20× in the reduction path — more than bit-packing's 15%. Blocked on P2 |

**Expression trunk** (after ADR-017 and the §2.4 L4 node-set decision) — fully independent of both:

| Lane | What | Grade | Par | Size |
|---|---|---|---|---|
| **X1** | Hash-consed `Store`, node set, `FuncId` table | certificate-graded | 1 | M |
| **X2** | `diff` / `diff_with`, constant folding | certificate-graded | 1 | M |
| **X3** | `walk_topological`, canonical bytes, schema version | certificate-graded | 1 | S |
| **X4** | `is_polynomial_in` bridge to Layer 1 | certificate-graded | 1 | S |

### Wave 3 — Layer 3 and elimination (univariate trunk continues)

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **A1** | `AlgebraicReal` core: construction, `refine`, `cmp`, `cmp_rational`, `sign_of`, enclosure | certificate-graded | 1 | L | The eleven properties are the verdict. **ADR-013 must be resolved first (§2.5)** |
| **A2** | `SqrtExt` | certificate-graded | 1 | M | Graded against A1's general route |
| **A3** | Radical-tower sign at arbitrary depth | certificate-graded | 1 | M | Graded against A1 materialization — a strong free oracle |
| **A4** | `rational_between` | certificate-graded | 1 | S | |
| **A5** | Bernstein certified range enclosure | certificate-graded | 1 | M | Soundness cert **plus** an Unknown-rate ceiling |
| **T1** | Subresultant PRS (Ducos) over ℤ | certificate-graded | 1 | L | |
| **T2** | Modular evaluation–interpolation resultant | certificate-graded | 1 | L | |
| **T3** | Bareiss / Bézout determinant route | certificate-graded | 1 | M | **The most independent of the three routes** — shares only `Integer` with the others |
| **T4** | Bivariate gcd, common-component detection | certificate-graded | 1 | M | |
| **T5** | Rational-witness fiber oracle (isolate `f(α,y)` at a rational abscissa) | certificate-graded | 1 | M | **Build before T6.** It is T6's only strong verdict |
| **T6** | Curve analysis: critical abscissas, branch counts, branch matching | certificate-graded *(weak)* | 1 | XL | **Poor agent target — see §5** |
| **T7** | Analysis-result sharing / caching | **score-graded** | 1 | — | The consumer today recomputes the resultant and re-isolates on every predicate call |
| **T8** | Modular resultant throughput | **score-graded** | 1 | — | ≥100× Ducos at degree ~20. Blocked on T1+T2 frozen |

### Wave 4 — factorization and Gröbner

| Lane | What | Grade | Par | Size | Notes |
|---|---|---|---|---|---|
| **K1** | Cantor–Zassenhaus / Berlekamp over `GF(p)` | certificate-graded | 1 | M | **Fully certified** — irreducibility over a finite field is decidable and cheap |
| **K2** | Hensel lifting + Landau–Mignotte bound | certificate-graded | 1 | M | |
| **K3** | Zassenhaus recombination | certificate-graded | 1 | M | Also the oracle for K5 at `r ≤ 20` |
| **K4** | LLL | certificate-graded | 1 | M | Output conditions are directly checkable. **A better agent target than it looks** |
| **K5** | van Hoeij lattice recombination | partial-certificate | 1 | XL | **Worst agent target in the plan — see §5** |
| **G1** | Buchberger + Gebauer–Möller | certificate-graded | 1 | L | The oracle. Criteria eliminate 99.99% of pairs; the coprime criterion is nearly worthless once the chain criterion is in |
| **G2** | F4 symbolic preprocessing + matrix construction | certificate-graded | 1 | L | Graded by agreement with G1 |
| **G3** | Sparse `GF(p)` row reduction | **score-graded** | 1 | — | **73–91% of an F4 run.** The hardest score lane. Blocked on G1+G2 frozen |
| **G4** | Cofactor tracking (`groebner_certified`) | certificate-graded + memory score | 1 | L | **Prototype the multiplier before committing** (§4) |
| **G5** | Modular Gröbner: tracing, majority vote, stabilization | **score-graded** | 1 | — | Returns `Probable`. Graded against G4 |
| **G6** | Batched multi-modular application | **score-graded** | 1 | — | Up to ~2.7× amortized at N=4; N=8/16/32 gave no further gain in the published work. Blocked on Z5 |
| **G7** | FGLM change of order | certificate-graded | 1 | L | |

### Fan-out summary

| Wave | Certificate lanes | Score lanes | Max useful concurrent agents |
|---|---|---|---|
| 0 | 5 | 3 (all measurement) | **7** |
| 1 | 6 | 1 | **6** |
| 2 | 13 across three trunks | 4 | **10–13** |
| 3 | 9 | 3 | **8** |
| 4 | 6 | 4 | **5** |

**The critical serialization, stated plainly:** the ADR freeze in §2 is the only true
global barrier. Before it, everything is harness work. After it, three trunks run
concurrently and never touch each other's one-way doors. The barrier is not "Layer 0 and
Layer 1 must be *built* first" — it is "**Layer 0 and Layer 1 must be *decided* first**",
and the univariate trunk inherits a strictly smaller set of decisions than the
multivariate one, which is why it can and should start earlier.

**Within each trunk the serialization is oracle-first:** Sturm before Descartes,
Descartes before ANewDsc, Ducos before modular resultants, Zassenhaus before van Hoeij,
Buchberger before F4, the rational-witness fiber oracle before curve analysis. Each of
these is a CI-enforced edge: a score lane's CI job **does not exist** until its oracle
lane is green and frozen. Without that, a score lane has no verdict function at all, and
constraint #3 requires every lane to have one.

---

## 4. Effort, honestly

Sizes are estimates in *agent-sessions* — one focused agent run ending in a green gate.
They are guesses calibrated against the amount of specification each lane has, not
against any measured throughput. **Score-graded lanes have no size**, and writing one
would be dishonest: the completion condition is a number, not a state.

| Milestone | Certificate work | Score work | Human decisions | Realistic elapsed |
|---|---|---|---|---|
| M0 | ~10 sessions, 5 parallel | 1 infra lane | — | days |
| M1 | ~15 sessions, 6 parallel | 2 measurement + 1 tuning | **The freeze. ~20 ADRs.** | weeks — and the ADRs are the long pole, not the code |
| M2 | ~20 sessions, 4 parallel | 3 lanes, open-ended | — | weeks |
| M3 | ~25 sessions, 4 parallel | 1 lane | ADR-013, decided by the §2.5 experiment | weeks |
| M4 | ~35 sessions, 4 parallel | 2 lanes | multiplicity semantics (§5) | **months** — T6 dominates |
| M5 | ~30 sessions, 3 parallel | 1 lane | — | **months** — K5 dominates |
| M6 | ~50 sessions, 3 parallel | 4 lanes, all open-ended | — | **many months.** The published gap between a real F4 (OpenF4) and the state of the art is 4–21× |
| M7 | ~15 sessions, 3 parallel | — | — | weeks, fully parallel to everything |
| M8 | ~30 sessions | 1 lane | — | months |

The IDEAS spec's own estimate is 12–24 months to be useful, and nothing found in the
research contradicts it. **What the research does change is *where* usefulness arrives:
at M4, not at M6.** The geometry consumer needs none of Gröbner. That reordering is the
single largest schedule win available and it is free.

### 4.1 Which milestones are NOT good agent targets

Said plainly, because pretending otherwise wastes months.

**M1's ADR freeze — not an agent lane at all.** The deliverable is judgment about
irreversible tradeoffs with no verdict function. An agent can *draft* an ADR from the
research and can *run the experiment* an ADR needs (ADR-013's sorting benchmark, Z2's
bignum ladder), but ratification is a human decision. Trying to grade it automatically
produces an ADR that argues for whatever is easiest to test.

**T6, curve analysis — the weakest verdict in the plan attached to the largest
component.** It has zero counterpart in the prior art, its invariants are weak (branch
counts consistent across adjacent intervals, Bézout-style bounds, matching is a
bijection), and *the specification is most of the work*. Its only strong grader is T5,
the independent rational-witness route, which is why T5 must be built first even though
it will never be the production path. Mitigation: split T6 into (a) critical-abscissa
extraction, which is just `Res_y(f, ∂f/∂y)` plus isolation and is well graded, and (b)
branch matching across critical fibers, which is the genuinely hard part — and treat (b)
as a human-specified, agent-implemented lane with a hand-authored corpus of curves with
known topology (nodal cubics, cusps, tangential pairs, vertical components, vertical
asymptotes).

**K5, van Hoeij — the hardest correctness lane in the library, and the least agent-
friendly.** No permissive reference implementation exists *anywhere*: SymPy (BSD) is
plain Zassenhaus and its own docs note LLL-based techniques are not implemented;
FLINT/PARI/Magma are LGPL/GPL/proprietary. It must be built from van Hoeij (2002),
Klüners' exposition, and Hart–van Hoeij–Novocin (ISSAC 2011). Its certificate is partial.
It depends on an LLL at a precision `lll-rs` is unproven at. And the failure mode —
returning a *coarse* factorization — passes the multiply-back check. Mitigation: keep
Zassenhaus as the production path with an explicit `r` threshold for as long as possible;
treat K5 as a research lane with a human reviewer; and make K4 (LLL) a separate, fully
certified lane first, because LLL's output conditions *are* directly checkable and
separating it removes the precision question from the recombination question.

**G3, sparse `GF(p)` row reduction — the hardest score lane.** It is 73–91% of an F4 run,
it converges over months, it needs a pinned machine, and it is exactly the kind of lane
where two agents working in parallel each optimize against a baseline the other is
moving. Staff it with **one** agent at a time, with a frozen baseline and a
change-point-tracked series. Do not fan it out. Also note: msolve reports AVX2 halving
the linear-algebra time, so a meaningful fraction of the achievable win is SIMD work,
which sits behind the `unsafe` confinement rule and needs its own audit.

**G5, modular Gröbner — everything it returns is `Probable`.** Its oracle (G4, the
certified mode) shares matrix construction, symbolic preprocessing, and the whole
monomial layer with it, so the cross-check is materially weaker than the resultant lane's
three-route agreement. It also cannot return `Proved` at all until the
Idrees–Pfister–Steidel / Noro–Yokoyama question is settled. Treat it as a lane that needs
external differential testing (Singular, msolve) as its *primary* verdict, not its
secondary one — an inversion of the normal rule that must be written into the lane brief.

**H5 and the benchmark calibration generally.** Deciding what counts as a regression is
judgment about noise. An agent will either set the threshold so tight the series flaps or
so loose it detects nothing. Calibrate per-series against measured run-to-run noise, once,
by hand, and commit the calibration.

**The L4 e-graph adapters (deferred).** A seam trait designed before any consumer exists
risks being wrong in a way a later `egglog` adapter cannot bridge. There is no verdict
function for "is this seam right". Ship the resolvent-owned trait, ship no adapter, and
wait for a consumer to complain.

### 4.2 Which are excellent agent targets

Z3 (`resolvent-modular`) — small, exhaustively certifiable, cleanly decomposable.
U1/U2 (`UPoly` arithmetic) — a naive reference is part of the deliverable, so the
oracle ships with the code. U4 (Sturm) — tiny, and it grades a much bigger lane.
K1 (`GF(p)` factorization) — the one factorization lane with a complete certificate.
K4 (LLL) — output conditions directly checkable. P1/P2 (monomials) — order axioms are
mechanical. A1–A5 (Layer 3) — the property suite is written already, in
`plans/verification.md` §2.6. All of the harness lanes. All of the generator work.

---

## 5. Consumer value ladder

What each milestone actually unlocks for a named consumer. Consumers are context, not
dependencies (founding constraint #1): each writes its own thin adapter.

| Milestone | Consumer | What becomes possible |
|---|---|---|
| M1 | **solverang** | Generic rank of the constraint Jacobian over `GF(p)` — evaluate at a random point, row-reduce mod `p`. E3 calls this the one place exact algebra is not a new capability but a strictly better implementation of something solverang already ships: faster *and* more correct than the incumbent. It falls out of `resolvent-modular` plus a row-echelon returning rank, pivot rows, dependent rows, **and the transform** — the same object as a Gröbner cofactor representation, one layer down |
| M2 | sinbad `solverang` (integrator) | Certified event-crossing counts per integrator step: lift dense-output `f64` coefficients to ℚ, isolate with a budget, return a decline rather than hanging. ~70 lines of adapter. *Caveat: that integrator is not written yet* |
| M3 (v0.1) | `arrangements` | The Layer-3 API exists and the four duplicated `Rc<RefCell<RealRoot>>` blocks can go. Degree still ≤ 4 in practice — this release is about proving the API, not lifting the ceiling |
| **M4 (v0.2)** | **`arrangements`** | **The unlock.** Arbitrary-degree curves `f(x,y)=0`: one `Res_y` call replaces three hand-rolled resultants, the lossy double-squaring elimination and its spurious-root filter disappear, and ~150 lines/family of by-hand radical-ladder derivation collapses into `sign_radical`-at-arbitrary-depth |
| M3–M4 | **cadabra2** | The strongest consumer found: `lazy-exact` is already a production dependency in five of nine crates, and `AlgebraicNumber` is a one-tuple newtype over `RealRoot` (`cadabra-core/src/exact/algebraic.rs:54`). Resolvent's proposition here is *substitution plus extension*, and the extensions are named, code-resident refusal sites that exist specifically because the substrate stops at degree-≤4 univariate ℚ: the Steinmetz plane-pair factorization that refuses on irrational factors, the plane×torus spiric quartic with no home, and the unbuilt torus lane, which E2 describes as **pure resultant work**. cadabra2 needs resolvent **total, certified, and honest about refusal** — speed is secondary — which makes it the best early consumer for the fail-closed and sharpness machinery |
| M5 | `arrangements`, cadabra2, SMT | Intersection multiplicity beyond the parity heuristic; minimal polynomials, hence a defensible `canonicalize()`; the irrational-factor refusals close |
| M6 | exact medial axis (#27) | Trivariate elimination, ideal saturation, 0-dimensional solving |
| M7 | sinbad `testkit`/`plexus`, FEM (#34) | MMS forcing generation; Pantelides index reduction. Both currently blocked in code. Both build-time, so bignum cost is free. **Independent of M2–M6 — staff it from the start** |
| M8 | SMT NRA (#12) | Full subresultant chains, principal subresultant coefficients, root isolation over ℚ(α₁..α_k), cofactor/certificate return for external proof production |

Note the asymmetry that makes M4-before-M6 correct: the geometry consumer **never does
algebraic-number arithmetic** — `RealRoot` has no `add`, `mul`, or `div` and four curve
families ship without them. SMT NRA requires it. That asymmetry is the single biggest
scoping win available and it is why the geometry trunk is short and the SMT trunk is long.

---

## 6. The first week

The goal is not to write algebra. **The goal is that the oracle loop exists before the
algorithms do**, so that every subsequent line of code arrives with a verdict function
already waiting for it. By day 7 there is a running, self-checking artifact that
falsifies its own bugs without a human looking.

### Day 1 — the gate that costs nothing later and everything if skipped

- Workspace skeleton. Two crate categories: `publish = true` (gated) and
  `publish = false` (`oracles/`, `fuzz/`, `bench/`). No third category, no exception
  process.
- `LICENSE-MIT` + `LICENSE-APACHE` (already present), `cargo-deny` config with an
  explicit `[licenses] allow` list and every copyleft SPDX id denied, running over the
  **published** dependency graph.
- The three planted regression cases for the license gate. **Verify the gate fails on all
  three.** A gate that has never been observed to fail is not known to work.
- CI Gate 0: build, clippy `-D warnings`, fmt, deny, `#![forbid(unsafe_code)]` inventory.
- `docs/decisions/0001-license-posture.md` (ADR G) — Tier A/B/C rules verbatim, including
  the Tier C entry for Symbolica **with its reason stated**, because an agent will
  otherwise reasonably infer that "source-available" is safer to read than GPL and the
  inference runs backwards.

### Day 2 — determinism, and the bignum measurement that gates the freeze

- Determinism harness: run any registered instance twice in-process, twice
  cross-process, at 1/2/8 threads, across feature combinations; compare canonical bytes.
  Golden-file machinery with a schema version, and a CI rule that a golden change without
  a version bump fails.
- **Start lane Z2 in the background:** clone `tczajka/bigint-benchmark-rs`, pin `dashu`
  0.5.2, run locally, and run the `gcd`/`gcd_ext` ladder at 64/256/1k/4k/16k bits against
  `rug`. This must land before Layer 0 is written, because a negative result strengthens
  the case for an optional GMP feature flag — cheap to design now, expensive later.

### Day 3 — the corpus and the score

- Corpus format: regression corpus (append-only, 100% gate), generator fleet (versioned),
  benchmark corpus (generated by committed generators with committed invariant
  assertions).
- Generator trait, committed seed schedule, score reporter emitting
  `(fleet_version, seconds_survived)`.
- The minimizer: delta-debug by dropping terms → halving coefficient bit-length →
  reducing degree → reducing variable count → reducing generator count → shrinking the
  query interval.
- **Validate the harness against a deliberately buggy stub**: it must falsify within
  budget, minimize the counterexample, and report the budget as survived once fixed.

### Day 4 — the first oracle and the first algebra

- Tier-0 sympy adapter: canonical S-expression protocol in and out, triage classifier
  producing Class A (self-certificate also fails — resolvent bug, certain) vs Class B
  (normalization/convention/oracle limitation).
- `resolvent-int`: `Integer` and `Rational` newtypes over `dashu`, with `dashu` in no
  public signature and no re-export. `from_f64 -> Option` fail-closed; `to_f64` with a
  documented rounding mode; `num_bits()`/`den_bits()`.
- `rug` as a dev-dependency oracle. Property tests: ring/field axioms, inverse-op
  round-trips, `gcd` with the coprimality certificate, generators targeting word
  boundaries.

### Day 5 — `Fp`, exhaustively

- `Fp` for word primes with Barrett/Shoup `mulmod`. Exhaustive certification against
  `i128` for every prime `p < 2^10`; random for `p < 2^63`; `a·a⁻¹ == 1` for every unit.
- The `Certificate` enum and the budget/decline error taxonomy, so every later signature
  is shaped correctly from the first call site.

### Day 6 — `UPoly` and the naive reference

- `UPoly<Integer>`: add, sub, mul, `divrem`, Horner, derivative, content/primitive,
  associate normalization, reciprocal transform.
- The naive `O(n²)` reference implementation, in the same crate, as the oracle.
- Certificates wired: `(a·b)/b == a`, degree additivity, evaluation homomorphism at
  random points in a large `GF(p)`.

### Day 7 — close the loop

- **Sturm sequences** (naive, over ℚ, low degree — it is an oracle, not a product).
- **Descartes/VCA isolation** over ℤ on dyadic intervals, degree ≤ 8, small coefficients.
- The verdict that makes this week worth it:
  `count_sturm(f, a, b) == len(isolate_descartes(f, a, b))` on the generator fleet, with
  a step budget, running in Gate 1.
- First score report: `(fleet_version=1, seconds_survived=N)`.
- Draft the ADRs for the freeze, and draft the ADR-K experiment (three mutability
  prototypes behind one trait, graded by self-comparison, sort stability, transitivity,
  `Send + Sync`, and the `n=10³` sorting benchmark).

**End-of-week artifact:** a workspace where two independently-written algorithms grade
each other automatically, on generated adversarial input, under a step budget, with a
minimizer that reduces any disagreement to its smallest form, a license gate that has
been observed to reject three real-world hazards, and a determinism check that every
future artifact depends on. **No algebra of consequence exists yet, and that is correct.**

---

## 7. Risks and anti-goals

| Risk | Mitigation |
|---|---|
| **Scope creep into "a general-purpose CAS."** The spec names this itself | Refuse a clever `simplify()`. Build strictly what the geometry consumer needs, then what FEM needs, in that order. M4 is the release that matters |
| **The freeze slips and lanes start against unratified decisions** | Wave 2 CI jobs do not exist until the ADRs they inherit are merged. Make it mechanical, not cultural |
| **Score lanes started before their oracle** | A score lane's CI job does not exist until its oracle lane is green and frozen. Also mechanical |
| **An agent optimizes monomial comparison** | It is worth 15%. The divisor-query index is worth 10–20× and S-pair criteria are worth 4 orders of magnitude. Write the ranking into the lane brief, or a week goes into the wrong 15% |
| **Fail-closed APIs pass every test and are useless** | Sharpness gates with committed ceilings on every "don't know" and "probably" outcome (`plans/verification.md` §3.13) |
| **Silent exponent overflow** | Guard bits in release builds; the 4-bit-field corpus sweep; zero silent divergences |
| **Cofactor tracking turns out to be unaffordable**, and `groebner_certified` cannot be the oracle | Prototype the multiplier on Katsura-8 / Cyclic-7 **before** committing. Fallback: Buchberger-with-cofactors on small instances only, and external differential testing promoted to primary for the fast mode |
| **The consumer workload is nothing like the guess** | Lane Y1 in wave 0 measures it before anything depends on the guess |
| **`arrangements` integration creeps back in** | resolvent depends on nothing local, imports no consumer traits, exposes no float interval type, takes no tolerance parameter anywhere, and owns no geometric concept. Curve analysis ships with `MPoly` in and algebraic data out |
| **A tolerance parameter appears somewhere** | The first consumer refuses tolerance by construction — its exact families declare `type Error = Infallible`. A tolerance argument anywhere in resolvent would make it unusable by the consumer it exists for. Treat any such parameter in review as a defect |
| **Reading the wrong source** | Tier C is a blocklist, not a preference. Every non-obvious module carries a `Derivation:` line citing a **paper**. A module that cannot cite one is a signal it was written from a source tree |
| **Copied tuning thresholds** | Every threshold is re-derived by measurement on resolvent's own corpus, with the measurement committed. This is simultaneously a licensing rule and a correctness rule, which is why it holds |

---

## Sources

`docs/research/prior-art-and-licensing.md`, `docs/research/consumer-requirements.md`,
`docs/research/algorithms-and-representation.md`, `docs/research/consumer-sinbad.md`,
`/home/dev/projects/IDEAS-crates.md` §4, and `plans/verification.md`. Every external
citation is carried from those, where it is sourced.

Consumer code read directly for grounding (context only; resolvent does not depend on it):
`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:1-16, 41-45, 317-322`,
`/home/dev/projects/arrangements/crates/lazy-exact/src/bernstein.rs:135-152`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:32-46`.
