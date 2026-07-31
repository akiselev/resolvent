# Consumer evaluation E2 — cadabra2

**Target:** `/home/dev/projects/cadabra2` — clean-slate exact CAD solid-modeling kernel,
9 crates, 57,302 lines of Rust (`tokei`, 2026-07-31), MIT OR Apache-2.0
(`/home/dev/projects/cadabra2/Cargo.toml:16`).

**Verdict: strong-consumer.** Not hypothetical and not aspirational: cadabra2 already
ships a production dependency on a narrower proto-resolvent (`lazy-exact`) in **five of
its nine crates**, including its trusted computing base. It has named, code-resident
fail-closed sites that exist *specifically* because that substrate stops at degree-≤4
univariate ℚ. Those refusals are the strongest evidence in this evaluation, and they are
in landed code, not in a plan.

**Why not `blocked-today`.** cadabra2 is not waiting on resolvent — it has a working
substrate. Resolvent's proposition here is *substitution plus extension*, not unblocking.
The honest framing: several shipped code paths refuse today for want of operations
resolvent's L2/L3 would supply, and the un-built torus lane is pure resultant work, but
cadabra2's current critical path (ROADMAP item B, topology publication —
`/home/dev/projects/cadabra2/ROADMAP.md:35-63`) does not touch any of them.

---

## 1. The decisive fact: cadabra2 is already a consumer

`lazy-exact` (`/home/dev/projects/arrangements/crates/lazy-exact`) is a production
dependency of `cadabra-core`, `cadabra-geom`, `cadabra-arrange`, `cadabra-check`, and
`cadabra-algorithms`. 37 `use lazy_exact` sites across the workspace.

Ratified by name, with the alternative considered and rejected
(`/home/dev/projects/cadabra2/docs/notes/design/ssi-boolean-plan.md:515-521`):

> **Adopt `lazy-exact` (+ transitively `dashu`) as the exact-arithmetic substrate** for
> the quadric decision path and as `cadabra-check`'s enclosure kernel … the alternative
> (an in-tree rational tower) re-builds months of landed, tested work.

The type that carries it is a one-tuple newtype over resolvent's L3 target shape:

```rust
// cadabra-core/src/exact/algebraic.rs:54
pub struct AlgebraicNumber(RealRoot);
```

where `RealRoot` is `{ defining_poly: QPoly, isolating interval, multiplicity }`
(`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:317-443`). That is
verbatim the spec's `AlgebraicReal { defining_poly, isolating_interval }`, and the spec's
claim that "this type is the whole bridge to computational geometry" is empirically
confirmed: it is the type cadabra2's arc endpoints, seam events, sheet junctions, and
p-curve split parameters are all made of.

**What this buys resolvent that a greenfield consumer cannot:** a measured API, a measured
cost, and a list of exactly where a degree-≤4-univariate-ℚ ceiling bites in a real CAD
kernel. The rest of this document is that list.

---

## 2. The dual-path posture, and what it means for resolvent's cost budget

`/home/dev/projects/cadabra2/docs/notes/design/dual-path-architecture.md` makes the
slow-exact / fast-approx split a system-wide principle (§1, lines 15-49). Four invariants
bind it; the two that constrain resolvent:

1. *"The fast face never silently substitutes for the slow face"* (line 32) — a fast
   result either grades as diagnostic or **escalates**; an unbuilt capability returns
   `NotImplemented`.
2. *"The slow face is the fast face's oracle"* (line 35) — escalation happens exactly
   where a certificate cannot be minted.

The measured split, from cadabra2's own microbench
(`/home/dev/projects/cadabra2/crates/cadabra-algorithms/benches/perf_r0.rs`, result
recorded at `docs/notes/design/dual-path-architecture.md:20-24` and `STATUS.md:79`):

| Arm | Cost | Note |
|---|---|---|
| Exact QSIC classification (`classify_quadric_intersection_exact`) | **2.448 ms** median | 24-term Leibniz `det(Q₁+λQ₂)` over `Real<Rational>`, Yun + Descartes root isolation, two exact congruence inertias |
| Lowered filter (`classify_sphere_cylinder_fast`) | **1.846 µs** median | one quartic-discriminant sign, T0 `f64`+bound rung |
| Ratio | **1326×** | `p_straddle = 0` on the generic corpus |

**The implication for resolvent is the single most important budgeting fact in this
document.** On the generic corpus the exact arm is *never called* — the T0 rung certifies
every generic pair. Resolvent is called on:

- the **degenerate strata** (discriminant exactly zero, tangency, coincidence) — rare per
  model, but they are the whole point of the kernel ("the moat",
  `ROADMAP.md` §F headline);
- **carrier construction and certification**, which is per-surface-pair and unconditional
  (§4.2 below), not filtered;
- the **arrangement/trim inner loop**, which is genuinely hot (§4.4).

So: resolvent may cost milliseconds on the classification path and cadabra2 will pay it,
because the fast face already ate the volume. Resolvent may **not** cost milliseconds in
the sweep inner loop. These are different budgets on the same library, and the API has to
let a caller land in either.

A second consequence: cadabra2 does not need resolvent to be fast in order to be *useful*.
It needs resolvent to be **total, certified, and honest about refusal**. Speed is a
second-order want here — which is unusual and worth stating plainly, because it inverts
the usual CAS priority.

---

## 3. The degree ceiling, instance by instance

Every one of these is a `NotImplemented` in landed code, with the missing algebra named in
the source.

### 3.1 Irrational repeated root ⇒ no inertia ⇒ classifier declines

`/home/dev/projects/cadabra2/crates/cadabra-algorithms/src/intersection/quadric/classification.rs:78-87`:

> The **rank** is always available … The **inertia** (the signed eigenvalue split) needs
> the root's rational value to build the member over ℚ, so it is present only at a
> rational root and `None` at an irrational one (exact number-field inertia is a
> documented follow-on).

and at `:441-445` the mapping refuses outright:

```rust
// Not square-free, yet no real repeated root: the singularity sits at a
// repeated *complex* root, so the real sub-stratum … is undetermined here.
Err(KernelError::not_implemented(Capability::Intersection, Subject::Pair(pair.0, pair.1)))
```

**What lifts it:** exact linear algebra over an **algebraic extension** ℚ(α) —
congruence diagonalization / Sylvester inertia of a symmetric matrix whose entries live in
ℚ(α), α a root of the characteristic quartic. This is resolvent L0 (algebraic extensions
as a coefficient ring) plus L2 (exact linear algebra over that ring). cadabra2 hand-rolls
the ℚ case at `classification.rs:292-338` (49 lines of congruence with a zero-pivot
`Rₖ += Rⱼ` rescue); it cannot hand-roll the ℚ(α) case because it has no ℚ(α).

### 3.2 Steinmetz plane-pair factorization refuses when the factors are irrational

`/home/dev/projects/cadabra2/crates/cadabra-algorithms/src/intersection/quadric/carrier_cylinder_cylinder.rs:216-222`:

> A reducible pencil (rank-≤2 member) is the Steinmetz stratum: try the exact rational
> plane-pair factorization (F-Q8). Its success is gated on the exact ten-coefficient
> identity, so **a general rank-2 member whose factor planes need SqrtExt coordinates
> fails closed** — the documented follow-on — rather than approximating.

The "factorization" is hand-verified by an exact ten-coefficient identity check
(`factorization_holds`, `:561-596`) against a *guessed* factor pair `a₂ ± a₁`
(`:101-111`). When the true factors are not that rational guess, the row refuses.

**What lifts it:** factorization of a quadratic form (a degree-2 multivariate polynomial)
into linear factors over ℚ or over ℚ(√d) — i.e. L1 multivariate polynomials plus L2
factorization over an extension. Concretely this is "diagonalize the rank-2 form, take the
square root of the discriminant, split". cadabra2 has `SqrtExt` (`a + b√r`) available and
still cannot do it, because it has no factorization routine and no multivariate polynomial
type — only `[[Rational; 4]; 4]` matrices and a dense univariate `QPoly`.

### 3.3 The biquadratic tower is sound but incomplete

`/home/dev/projects/cadabra2/crates/cadabra-check/src/biquad.rs:1-42` builds
ℚ[α,β]/(α²−q_α, β²−q_β) — a multiquadratic field — with a hand-written multiplication
table on the basis `{1, α, β, αβ}`. Its own docs state the gap (`:30-37`):

> In those degenerate cases the rule loses only completeness: a residual that vanishes
> *because of* the dependence (e.g. `1/2 − α` with `q_α = 1/4`) has non-zero components
> and **fails closed** — the checker refuses, it never certifies wrongly.

**What lifts it:** a real number-field type that knows its own minimal polynomial and can
detect a degenerate tower (`q_α` a rational square ⇒ the tower collapses). L0 algebraic
extensions with a primitive element / minimal polynomial. Note the posture: cadabra2
chose sound-and-incomplete over complete-and-unverified. Resolvent must be able to be the
former too — see §7.

### 3.4 Generic plane×torus: the spiric quartic has no home

`/home/dev/projects/cadabra2/crates/cadabra-algorithms/src/intersection/rows/plane_torus.rs:24-27`:

> The **generic spiric quartic** — any placement outside the three circle strata — has no
> exact curve family here, so the row fails closed with a specific
> `KernelError::NotImplemented` rather than approximating it.

Refusal minted at `:374-378`, hit from three sites (`:152`, `:185`, `:207`). The three
handled strata (axis-perpendicular, meridian, Villarceau) are hand-derived closed forms
(`:9-19`).

**What lifts it:** the topology of a real algebraic plane curve of degree 4 — factorization
over ℚ (which would find the Villarceau/meridian circle strata *automatically* instead of
by three hand-coded special cases), plus discriminant / subresultant PRS to locate critical
points, plus real root isolation of the fibres. This is exactly resolvent L2's stated
resultants-and-real-root-isolation scope. It is the cleanest single lift in this document:
one general capability replaces three special cases *and* covers the generic case they
were carved out of.

### 3.5 The torus lane is unbuilt and is pure resultant work

`/home/dev/projects/cadabra2/docs/notes/design/ssi-boolean-plan.md:154-166` specifies
Regime II′ (F-T1/F-T2/F-T3): quadric×torus by "generator-line reduction (cylinder/cone
rulings substituted into the torus implicit ⇒ quartic-in-`b` per generator)" plus "the 2-D
preimage topology `G(a,b)=0` in the host domain"; torus×torus by "torus-domain implicit
preimage with characteristic points". Nothing of this exists in the tree
(`ROADMAP.md` watch list: "Torus SSI (net-new; not in the old kernel)").

`G(a,b) = 0` is a bivariate implicit curve. Deciding its topology is elimination:
`resultant_b(G, ∂G/∂b)` for critical abscissas, subresultant PRS for the fibre structure,
real root isolation at each. cadabra2 has none of it. This is the largest *net-new* demand
resolvent could satisfy.

### 3.6 Higher-order contact in the radicand

`/home/dev/projects/cadabra2/crates/cadabra-geom/src/exact/harmonic.rs:355-380`: a
radicand root of multiplicity > 2, or a tangency coexisting with a sign change, both refuse
(`unresolved_radicand_structure`). Multiplicity is available (Yun square-free
decomposition), so the algebra is *there*; what is missing is the branch decomposition, a
geometry problem, not an algebra one. **Listed here to be excluded**: resolvent does not
lift this one.

---

## 4. Operations demanded, with latency class

Latency classes are inferred from the call site, not guessed. I distinguish four:

- **inner-loop** — inside the arrangement sweep / a per-event comparison. Millions of
  calls; a bignum allocation per call is already too expensive.
- **per-operation** — once per surface pair, per carrier, per certificate. Thousands per
  model. Milliseconds are affordable, seconds are not.
- **batch** — once per SSI report / per Boolean. Tens per model.
- **build-time** — offline; corpus generation, oracle runs, tests.

### 4.1 Inner loop — L3 comparison and radical sign

The arrangement sweep that cadabra2 delegates to (`cadabra-arrange` →
`arrangements::geoms::sine_radical`) decides every event with:

- `RealRoot::cmp_root` — exact order of two algebraic numbers by interval refinement then
  gcd (`/home/dev/projects/arrangements/crates/arrangements/src/geoms/sine_radical.rs:83-87`,
  `:599`, `:687-690`, `:803`);
- `sign_radical2(ξ, α, β, h, γ, o.h)` — the exact sign of `α + β√h + γ√h'` evaluated at an
  algebraic abscissa ξ (`:384`, `:638`, `:837`).

cadabra2 reaches these through `cadabra-arrange/src/trim.rs` on every p-curve trim and
every imprint. This is the tightest budget in the consumer and it is entirely L3.

**API consequence, and it is a defect to fix rather than inherit.** `RealRoot::cmp_root`
takes `&mut self` (refinement mutates the cached interval). That forced the sweep into
`Rc<RefCell<RealRoot>>` (`sine_radical.rs:77`) and then forced cadabra2 to write an
`Rc::ptr_eq` guard purely to dodge a `RefCell` double-borrow panic when comparing a root
against itself:

```rust
// cadabra-arrange/src/trim.rs:857-862
fn cmp_shared(left: &SharedRoot, right: &SharedRoot) -> Ordering {
    if Rc::ptr_eq(left, right) { return Ordering::Equal; }
    left.borrow_mut().cmp_root(&mut right.borrow_mut())
}
```

Resolvent's `AlgebraicReal` must hide the refinement cache behind interior mutability so
comparison takes `&self` and the type can `impl Ord`. `&mut` comparison is not merely
awkward — it is unsortable, un-`BTreeMap`-able, and it cost this consumer a shared-mutable
wrapper and a self-comparison guard in its hot path.

### 4.2 Per-operation — carrier certification

`cadabra-check` mints a `WholeCarrierCertificate` per carrier per operand (two per
sphere×cylinder branch, e.g.
`carrier_cylinder_cylinder.rs:147-151` — `left_lift`, `right_lift`). Each mint runs:

1. Weierstrass rationalization `t = 2·atan u` of an expression tree over
   `{const, cos t, sin t, cos 2t, sin 2t, √r(t)}` into `(A(u) + B(u)√H(u))/(1+u²)^k` with
   `A,B,H ∈ ℚ[u]` (`cadabra-check/src/weierstrass.rs`, `carrier.rs:155-180`);
2. either an **identity proof** `A ≡ 0 ∧ B ≡ 0` in the rational kernel
   (`ProofStrength::AlgebraicallyExact`), or
3. an **exact Bernstein sup-norm enclosure with certified de Casteljau subdivision**
   (`ProofStrength::IntervalEnclosed`, `cadabra-check/src/enclosure.rs`, consuming
   `lazy_exact::Bernstein`).

The design note is explicit that the naive route does not work
(`docs/notes/design/whole_carrier_enclosure.md:29-46`): interval evaluation of a
multi-occurrence expression over-widens by the dependency problem, so a residual that is
*identically zero* returns a wide non-zero interval and "the checker rejects a **true**
fact"; and `√r(t)` blows up at a radicand root, "exactly where the geometry is most
delicate."

**This is a first-class resolvent requirement, and it is not obvious from the spec.** A
CAS that gives you `resultant` and `isolate_roots` but no *certified sup-norm enclosure of
a polynomial over a rational box* cannot serve this consumer's certification path at all.
Bernstein-basis range bounds with exact de Casteljau subdivision belong in resolvent L1/L2.

### 4.3 Per-operation — exact linear algebra over ℚ[λ] and ℚ

Three hand-rolled routines, all in `classification.rs`:

| Routine | Lines | What it is |
|---|---|---|
| `qpoly_determinant` | `:196-221` | Laplace expansion of an n×n matrix of `QPoly`s |
| `member_rank_at_root` | `:242-260` | rank of `A+λB` at an algebraic root, by testing every k×k minor's vanishing with `RealRoot::is_root_of` |
| `inertia` | `:292-338` | Sylvester inertia by exact congruence with zero-pivot rescue |

Plus a fourth in `matrix.rs:378-381`, a Leibniz 24-term determinant of a 4×4 matrix of
degree-≤1 λ-polynomials, with a `LambdaPoly` type that is a `[S; 5]` dense univariate
truncated at degree 4 (`matrix.rs:277-281, 337-338`).

`qpoly_determinant` is recursive Laplace — O(n!) — over `QPoly` with `dashu` rationals.
That is the 2.448 ms. A fraction-free Bareiss or a modular/CRT determinant would be
orders faster with no change to the result. cadabra2 does not ask for modular methods by
name; it asks for a fast exact determinant, and modular is how you give it one. That
distinction matters for resolvent's API: **primes must not appear in the signature.**

`member_rank_at_root` is the interesting one — it computes rank at an *algebraic* λ without
ever materializing λ, by asking `is_root_of` on each minor polynomial. That is a nice
technique and resolvent should support it directly (`AlgebraicReal::is_root_of(&Poly)`,
which `lazy-exact` already has at `roots.rs:480`). It is also exactly why the *rank* works
at irrational roots while the *inertia* does not (§3.1): rank is a vanishing question,
inertia needs arithmetic in ℚ(α).

### 4.4 Per-operation — separating witness between two algebraic numbers

Hand-rolled **twice**, in two different crates of the same consumer:

```rust
// cadabra-arrange/src/trim.rs:842-854
fn rational_between(low: &SharedRoot, high: &SharedRoot) -> KernelResult<Rational> { … }
```
```rust
// cadabra-geom/src/exact/harmonic.rs:793-818
fn sample_between_roots(roots: &[RealRoot], index: usize) -> KernelResult<ExactScalar> { … }
```

Both bisect with a hard 256-step budget and a typed refusal on exhaustion ("turns a
surprise … into a typed refusal instead of a hang"). Two independent implementations of
one primitive inside one consumer is the clearest possible signal that it belongs in the
library. Resolvent should ship `rational_strictly_between(&AlgebraicReal, &AlgebraicReal)
-> Option<Rational>` and `rational_sample_in_gap(&[AlgebraicReal], usize)`.

### 4.5 Batch — residual sup-norm bounds that are currently hand-derived

`intersection/rows/residual.rs` is the file the brief flagged, and the finding is a *soft*
one. It contains no exact algebra at all: it is dimensional-honesty plumbing that converts
a quadric implicit value (squared-length units) into a model-space distance. Where it can
do that exactly it does (`sphere_circle_distance`, `:59-73`, decomposes the centre offset
and reports the exact extremal radii). Where it cannot, it degrades to a **coefficient
one-norm** of a five-term trigonometric polynomial:

```rust
// residual.rs:201-230
fn trig_quadratic_coefficients(...) -> [f64; 5] { … }
fn coefficient_difference_bound(left, right) -> f64 { /* Σ|lᵢ − rᵢ| */ }
```

and `cone_circle_normalized` (`:145-176`) gives up on distance entirely, returning a
dimensionless ratio, with the honest comment "without pretending the raw squared-length
equation is a distance."

**What lifts it:** the same Bernstein sup-norm enclosure from §4.2. `sup_{t} |ρ(t)|` over a
rationalized trig polynomial is a range bound on a degree-≤4 polynomial over a box —
tighter than the coefficient 1-norm and *certified* rather than `f64`. Today the residual
evidence in this file is `f64`-computed and self-measured from the same `f64` that produced
the geometry, which `ROADMAP.md`'s watch list flags as "internally consistent, not
independently correct". Routing residual.rs through resolvent's enclosure would close that.

Latency: batch. It runs once per emitted curve component.

### 4.6 Summary table

| # | Operation | Layer | Evidence | Urgency | Latency |
|---|---|---|---|---|---|
| 1 | exact order of two real algebraic numbers, `&self` receiver, `impl Ord` | L3 | `arrangements/src/geoms/sine_radical.rs:83-87`; `cadabra-arrange/src/trim.rs:857-862` | blocked-now | inner-loop |
| 2 | sign of `α + β√h (+ γ√h')` at an algebraic abscissa | L3 | `sine_radical.rs:384,638,837`; `lazy-exact/src/roots.rs:622-649` | blocked-now | inner-loop |
| 3 | real root isolation over ℚ with exact multiplicities (Yun + Descartes/VCA) | L2 | `quadric/roots.rs:45-74`; `geom/src/exact/harmonic.rs:377` | blocked-now | per-operation |
| 4 | square-free part / decomposition, gcd, `is_root_of` | L2 | `quadric/roots.rs:64`; `classification.rs:253` | blocked-now | per-operation |
| 5 | certified Bernstein sup-norm enclosure + exact de Casteljau subdivision | L1/L2 | `cadabra-check/src/enclosure.rs`; `whole_carrier_enclosure.md:29-46` | blocked-now | per-operation |
| 6 | determinant of a matrix over ℚ[λ] (fast; Bareiss/modular) | L2 | `classification.rs:196-221`; `matrix.rs:378-381`; 2.448 ms | blocked-now | per-operation |
| 7 | rank of a polynomial matrix at an algebraic root | L2/L3 | `classification.rs:242-260` | blocked-now | per-operation |
| 8 | Sylvester inertia / congruence diagonalization over ℚ | L2 | `classification.rs:292-338` | blocked-now | per-operation |
| 9 | **the same over ℚ(α)** — number-field linear algebra | L0+L2 | `classification.rs:78-87` (`None` at irrational root), `:441-445` | blocked-now | per-operation |
| 10 | rational witness strictly between two algebraic numbers | L3 | `trim.rs:842-854`; `harmonic.rs:793-818` (two hand-rolls) | blocked-now | per-operation |
| 11 | one-radical extension `a + b√r` with total order across distinct radicands | L0 | `cadabra-core/src/exact/radical.rs`; `lazy-exact/src/sqrt_ext.rs` | blocked-now | per-operation |
| 12 | factorization of a quadratic form into linear factors over ℚ and ℚ(√d) | L1/L2 | `carrier_cylinder_cylinder.rs:216-222,561-596` | blocked-now | per-operation |
| 13 | degenerate-tower detection (is `q` a rational square? minimal polynomial) | L0 | `cadabra-check/src/biquad.rs:30-42` | next-milestone | per-operation |
| 14 | factorization of a degree-4 plane curve over ℚ (reducibility ⇒ conics/circles) | L2 | `rows/plane_torus.rs:9-27,374-378` | next-milestone | per-operation |
| 15 | resultant / subresultant PRS eliminating one variable from a bivariate system | L2 | `ssi-boolean-plan.md:154-166` (F-T2/F-T3, unbuilt); `arrangements/src/geoms/conics.rs:272-287` (hand-rolled closed form) | next-milestone | per-operation |
| 16 | topology of a real bivariate algebraic curve `G(a,b)=0` (critical points + fibres) | L2 | `ssi-boolean-plan.md:158-161` | next-milestone | batch |
| 17 | exact interval arithmetic without global rounding-mode state | L0 | `lazy-exact/src/interval.rs`; `cadabra-geom/src/certified/` | blocked-now | inner-loop |
| 18 | lazy filtered exact real (`interval` eager, exact lazy) with an expression DAG | L0 | `lazy-exact/src/real.rs`; used as `Real<Rational>` throughout the pencil path | blocked-now | per-operation |
| 19 | forward-mode dual over the exact rung (`Dual<Real>`) for exact ∂x/∂p | L0 | `scalar-seam`; `ROADMAP.md` §F Track D2 | eventual | per-operation |
| 20 | `Scalar`-generic arithmetic seam so one algorithm text runs on `f64`, interval, or exact | L0 | `cadabra-geom/src/nurbs/` (de Boor written once); `fastpath/filter.rs` `TierField` | blocked-now | inner-loop |

---

## 5. Layers stressed

- **L0 (coefficient rings): heavily, and in an unusual shape.** cadabra2 wants ℚ, an
  interval type, a lazy filtered real, a one-radical extension, and a multiquadratic tower
  — and it wants them behind *one generic seam* so a single algorithm text instantiates at
  every tier (`fastpath/filter.rs`'s `TierField`, `scalar-seam`'s `Scalar`). It does **not**
  want `GF(p)` or `Z/n` exposed. Algebraic extensions are the biggest L0 gap (§3.1, §3.2,
  §3.3).
- **L1 (polynomials): dense univariate only, today.** Everything cadabra2 touches is
  `QPoly` — a dense `Vec<Rational>` at degree ≤ 4-ish. The packed-exponent sparse
  multivariate representation the spec calls a one-way door is **not exercised by this
  consumer at all** today, and would only be exercised by the torus lane (§3.5) and the
  Steinmetz factorization (§3.2), both of which are degree ≤ 4 in ≤ 3 variables. cadabra2
  is therefore a **weak witness for the L1 representation decision** — treat it as a
  constraint that dense-small must be cheap, not as evidence about sparse-large.
- **L2 (the engine): the centre of gravity.** Root isolation, square-free decomposition,
  gcd, exact linear algebra, Bernstein enclosure — used now. Resultants/subresultants and
  factorization — the named lifts.
- **L3 (algebraic numbers): the load-bearing bridge, exactly as the spec predicted.**
  Both the inner loop and the event vocabulary are `RealRoot`. Multiplicity must be on the
  type: "a double root of the radicand is a sheet junction where the two sheets meet, not
  two separate crossings" (`cadabra-core/src/exact/algebraic.rs:106-108`).
- **L4 (expression DAG): stressed, but not in the shape the spec assumes.** See §8.

Not stressed: modular methods as a user-facing concept, Gröbner/F4 (zero occurrences of
"Gröbner", "Buchberger", "F4", "ideal membership" anywhere in cadabra2's crates), symbolic
calculus, simplification.

---

## 6. Anti-findings — what cadabra2 does NOT need, or solves better itself

Stated bluntly, because these are the places resolvent would waste effort or actively
damage the consumer.

1. **No Gröbner bases, no F4, no ideal membership.** Grepped the whole workspace: zero
   hits. The natural-quadric problem is a pencil problem with a complete classical theory
   (`ssi-boolean-plan.md:110-152`); the freeform problem is subdivision + interval-Newton
   (§1.4 of the same doc). Neither is an ideal-theoretic problem. Building F4 first would
   serve this consumer not at all.

2. **No multivariate polynomial factorization at scale, no van Hoeij.** The only
   factorizations wanted are degree ≤ 4 in ≤ 3 variables (§3.2, §3.4). Zassenhaus +
   lattice recombination is over-engineering *for this consumer*; a small-degree special
   case would satisfy it entirely.

3. **No numeric root polishing, and resolvent must not offer it as a convenience.** The
   whole point of `quadric/roots.rs` is "no numeric root polishing enters the decision
   path" (`:11-12`). A `f64` root-finder in resolvent's API is an attractive nuisance here.

4. **No Newton / corrector / marching.** cadabra2 owns this and owns it well:
   `intersection/freeform/corrector.rs` runs a "raw-`f64` inner loop inside a validated
   envelope" (`:31-41`) with interval post-conditions and `DirectionCone` certificates. The
   `REL_general` four-callback factoring is on its watch list
   (`dual-path-architecture.md:91-100`). Resolvent supplying a Newton solver would be
   rejected.

5. **No tolerance model, no epsilon, no "approximately equal".** cadabra2 has a role-typed
   `ToleranceContext` with a session-scale relative+absolute model and forbids global
   epsilons. Any resolvent API taking an `eps: f64` for an equality decision is unusable
   here.

6. **Resolvent certificates must NOT feed `cadabra-hints`.** The brief asked; the answer
   is a firm no, and the reason is instructive. `cadabra-hints` is defined by the invariant
   "**a hint is structurally incapable of deciding anything**"
   (`cadabra-hints/src/lib.rs:19-47`), rule H1 "a hint is never evidence — no type here
   implements any evidence, proof, certificate, sign, or certainty trait", and the
   consequence "**hint values need no unforgeability** … the trust boundary does not pass
   through this cache at all … the single most load-bearing simplification in the design."
   Feeding a certificate into the hint store would destroy that simplification. What
   *could* legitimately flow into hints is resolvent's non-decision metadata — which tier a
   previous call settled at, how many bisections a root needed, the precision reached —
   i.e. warm-start knobs. Resolvent should expose those as plain data with no proof type
   attached.

7. **No `String`/`Display`-based symbolic API.** cadabra2 bans display readouts in
   production paths (`STATUS.md`, cadabra-hints S8: "display readouts clippy-banned in
   production").

8. **No panics, anywhere, for any reason.** cadabra2's CORE RULE is "Our code must never
   panic and all functions that can fail must return a result type." `lazy_exact::SqrtExt::new`
   panics on a negative radicand (`sqrt_ext.rs:38-41`), and cadabra2 had to write a guard
   around it (`cadabra-core/src/exact/radical.rs:77-88`). Every panicking constructor in
   resolvent becomes a hand-written guard in every consumer.

9. **cadabra2 does not need resolvent to be fast on the generic path.** §2. The fast face
   already took the volume. Optimizing resolvent's generic-case throughput at the cost of
   API honesty would be optimizing the wrong thing *for this consumer*.

---

## 7. The fail-closed / evidence contract — extracted

This is the contract resolvent must satisfy to be adoptable here. Every clause is quoted
or paraphrased from cadabra2's own binding text.

**C1 — Refusal is typed, specific, and structural.** *"Anything that is not implemented
should always return a NotImplemented kernel error … Even if a fallback is implemented, it
should still fail if the main algorithms are not implemented."* (CLAUDE.md CORE RULES). The
refusal names the missing *capability* and the *subject* it was needed for
(`KernelError::NotImplemented { capability: Capability, subject: Subject }`,
`cadabra-core/src/error.rs:290-297`). Resolvent's errors must be an enum with the same
shape: what could not be done, to what. `CheckError`
(`cadabra-check/src/error.rs:17-90`) is the model — twelve variants, every one "a *refusal
to certify*, never a silent pass", several carrying the offending data (`NearCritical { u_lo,
u_hi }`).

**C2 — No silent approximation, no silent default.** *"Never silently fallback to data like
zeroed points or vectors. Never silently fallback to approximate algorithms."* A resolvent
function that returns a best-effort answer without saying so is unusable.

**C3 — Evidence grade travels with the value.** Three levels, and the type system enforces
the ladder: `…Record` (diagnostic) → `Verified…` (proof) → `Ready…` (strict gate)
(`ROADMAP.md` §A; `dual-path-architecture.md:38-41`). Resolvent's results must be
distinguishable as *decided-exactly* vs *bounded* vs *unknown*, at the type level.
`ProofStrength::{AlgebraicallyExact, IntervalEnclosed}`
(`cadabra-check/src/certificate.rs:66-79`) is the exact vocabulary.

**C4 — Certificates are unforgeable by construction.** *"all fields are private and the
only constructors are crate-private, so this can be *inspected* by downstream code but
never *minted* by it — a certificate exists iff this crate proved the claim"*
(`certificate.rs:189-199`). Resolvent's certificate types must have private fields and
crate-private constructors. A `pub struct Certificate { pub .. }` is worthless here.

**C5 — Certificates are *tethered* to the claim they attest.** This is the subtle one and
cadabra2 learned it the hard way (refactor-p1, `STATUS.md`). Each evidence variant carries
the very expression it proved (`IdentityEvidence::claim`, `BoundEvidence::claim`,
`BiquadIdentityEvidence::claim` — all private, read through an accessor), and
`certifies(&expr)` is structural equality against it (`certificate.rs:310-335`). The
purpose: *"a transplanted certificate fails the comparison instead of riding along"*
(`certificate.rs:41-42`). Resolvent must ship the same: a certificate carries its claim and
exposes `certifies(claim) -> bool`.

**C6 — The exactness boundary is three named exits, and only three.**
`cadabra-core/src/exact/mod.rs:40-46`:
> Exactly three exits exist, on every exact type: `demote_exact` (lossless or a typed
> error), `enclosure` (certified outward interval), `approx_lossy` (nearest double,
> diagnostic only).

This is enforced mechanically: `clippy.toml` bans `.raw()` and lossy exits outside declared
`L0 leaf:` modules, and all 45 migration opt-outs were deleted (`STATUS.md`). **Resolvent
should adopt this API contract verbatim.** It is the single most transplantable design
decision in the consumer, it is already proven in production, and getting it wrong forces
the consumer to wrap every one of resolvent's types.

**C7 — Sound beats complete; incomplete is a first-class outcome.** §3.3's biquad rule
loses completeness in degenerate towers and says so. `CheckError::NearCritical` is a
certified *"I could not decide"* carrying the box it could not decide over. Resolvent must
be able to return "unknown, and here is why" as a value, not as an error-that-means-bug.

**C8 — Refusal must be reachable without a hang.** Both hand-rolled witness searches
(§4.4) carry a hard step budget precisely so an unexpected non-separation becomes a typed
refusal rather than an infinite loop. Resolvent's refinement loops need caller-visible
budgets with the same discipline (`SubdivisionBudget` in `cadabra-check/src/budget.rs` is
the model).

**C9 — Determinism and byte-identity.** cadabra2 pins a 52-case golden corpus as
byte-identical across a ten-step refactor (`STATUS.md`). Resolvent must not have
iteration-order or hash-order nondeterminism in any output.

---

## 8. Answers or certified answers? — certified, unambiguously, and with a twist

cadabra2 wants **certified answers**, and resolvent's certificate-emitting design is aimed
correctly at it. But the twist matters:

**cadabra2 does not want resolvent's certificate to be the trust root.** Its TCB is
`cadabra-check`, a hook-protected path only a deliberate human session may modify
(`oracle/mod.rs:48-55`). The whole `cadabra-testkit/src/oracle/{exact,interval,internal}.rs`
tier — including a **from-scratch `BigInt`** (`oracle/exact.rs:27-38`) and a
**from-scratch directed-rounding `Interval`** (`oracle/interval.rs`) — is written
`std`-only and dependency-free specifically so it can be *lifted verbatim into the TCB*.
cadabra2 currently runs three independent bignum stacks: `dashu` via `lazy-exact`, this
hand-rolled `BigInt`, and `f64` expansions in `lazy-exact/src/expansion.rs`.

That is a deliberate cross-check, not an accident. `cadabra-check/tests/cross_check_lazy_exact.rs`
exists as *"a second dissimilar consumer pinning that kernel's behaviour"*
(`cadabra-check/Cargo.toml:9-14`). So the posture toward resolvent is:

- **use it for answers**, including certified ones;
- **but expect to be independently cross-checked**, and design so that being cross-checked
  is cheap: small, auditable, dependency-light core; no unsafe; deterministic; behaviour
  pinnable by a differential test.

There is a real tension here and resolvent should decide it consciously. cadabra2's TCB
comment says the oracle tiers are dependency-free *"so they can be lifted into
`cadabra-check`, the trusted computing base, which forbids third-party dependencies"*
(`oracle/exact.rs:12-15`) — yet `cadabra-check` *does* depend on `lazy-exact` and
transitively `dashu`, ratified explicitly with the tradeoff named
(`ssi-boolean-plan.md:515-521`). The comment is aspirational; the ratification won. The
lesson for resolvent: **a consumer with a TCB will admit you if your dependency surface is
small and your behaviour is pinnable.** `lazy-exact` got in on `dashu` + `smallvec` +
`thiserror` + a zero-dep seam crate. That is roughly the budget.

Second twist: the certificate is consumed **as an admission ticket, not as a proof to
read**. `WholeCarrierCertificate` gates topology publication; the divergence protocol
(`oracle/certified_divergence.rs:6-27`) uses its *presence* to classify a
Parasolid disagreement as `CertifiedDivergence` (a moat entry) rather than `MissionBug`.
Nobody re-verifies it. So the value of a resolvent certificate is: (a) it cannot be forged,
(b) it names what it attests, (c) it is cheap to carry. Elaborate proof objects that a
consumer must interpret are not wanted.

---

## 9. How resolvent's error model composes with cadabra2's

cadabra2 has a two-layer pattern already, and resolvent should slot into the lower layer.

- **Lower layer, library-local vocabulary.** `CheckError` (`cadabra-check/src/error.rs`)
  is deliberately its own small enum: *"the checker itself keeps its own small vocabulary
  so the trust boundary depends on nothing but the exact-arithmetic substrate"* (`:5-8`).
- **Upper layer, kernel vocabulary.** *"Producer code (the quadric SSI rows in
  `cadabra-algorithms`) maps these onto the kernel's `KernelError::NotImplemented`
  fail-closed sites"* (`:5-7`).

**Therefore: resolvent must NOT try to be `KernelError`-shaped, and must NOT be
`std::error::Error`-generic-and-vague.** It should ship a closed, small, `PartialEq`,
`Clone`, non-`Box<dyn>` enum with:

- one variant per *reason a decision could not be made*, distinguished from *the caller
  broke the contract*;
- offending data on the variant where the caller could act on it (cf. `NearCritical { u_lo,
  u_hi }`);
- no `String` payloads (they defeat `PartialEq` matching, which cadabra2 uses in tests:
  `assert!(matches!(…, Err(KernelError::InvalidGeometry { reason: InvalidDomain, .. })))`).

The adapter then writes one `From<ResolventError> for KernelError` — about 20 lines. That
composition already exists for `lazy-exact` and it is 6 lines
(`cadabra-core/src/exact/algebraic.rs:81-90`), which is the proof it works.

One concrete flaw to avoid: `lazy_exact::isolate_roots` returns
`Result<Vec<RealRoot>, RootError>` with exactly one variant, `ZeroPolynomial`. cadabra2
throws the information away — `Err(_) => Err(KernelError::invalid_geometry(...))`
(`algebraic.rs:84`) — and elsewhere writes `.unwrap_or_default()` after hand-checking for
the zero polynomial (`quadric/roots.rs:62-63`). A single-variant error that the caller
always pre-checks is a signature smell: prefer a total function whose domain excludes the
degenerate case (`isolate_roots(p: &NonZeroPoly)`), or make the variant carry something.

---

## 10. API pressure on resolvent — the specific shape constraints

Ranked by how much a violation would cost this consumer.

1. **`AlgebraicReal` comparison and sign query must take `&self`.** Refinement cache behind
   interior mutability. `impl Ord`. Evidence: §4.1's `Rc<RefCell<_>>` + `ptr_eq` guard.
2. **Three named exits from exact, and only three:** `demote_exact` (lossless or typed
   error) / `enclosure` (certified outward interval) / `approx_lossy` (diagnostic double).
   No `as f64`, no `Into<f64>`, no `Display` that rounds. (`cadabra-core/src/exact/mod.rs:40-46`.)
3. **Total functions. No panics. Ever.** Including `sqrt` of a negative, division by zero,
   and every "caller bug" case. (`sqrt_ext.rs:38-41` is the counter-example that cost a
   guard.)
4. **A `Scalar`-style seam trait so one algorithm text instantiates at `f64`, interval, and
   exact.** cadabra2 writes de Boor once and runs it at three tiers; the fastpath renders
   T0/T1/T2 from one `TierField` program. Resolvent's polynomial and linear-algebra
   routines should be generic over a ring trait, not hard-wired to ℚ.
   (`scalar-seam`; `fastpath/filter.rs`.)
5. **Certificates: private fields, crate-private mint, carry the claim, `certifies(claim)`.**
   (`cadabra-check/src/certificate.rs:189-335`.)
6. **Multiplicity on the root type**, not recomputed. Tangency detection depends on it.
   (`algebraic.rs:104-112`.)
7. **Budgets are caller-visible and exhaustion is a typed value, not a hang.**
   (`cadabra-check/src/budget.rs`; the two 256-step witness loops.)
8. **Exact ingress from `f64` must be lift-then-operate, never operate-then-lift**, and the
   API should make the wrong order hard to express. cadabra2 codified this as SEM-0
   ("Lift THEN subtract", `cadabra-geom/src/exact/algebra.rs:36-45`) and enforced it by
   making `QPoint3::vector_to` the *only* two-point subtraction on the exact face. Resolvent
   should not offer `Rational::from_f64(a - b)`-shaped conveniences.
9. **No `eps` parameter on any equality or sign decision.**
10. **Small dependency surface; no `unsafe`; deterministic output.** The TCB admission
    budget is roughly `dashu` + `smallvec` + `thiserror`. `#![forbid(unsafe_code)]` is
    already the norm in both `lazy-exact` and `cadabra-check`.

---

## 11. L4 — the one place the spec's shape is wrong for this consumer

The spec's L4 is a hash-consed DAG, `egg`-compatible, for simplification by e-graph.
cadabra2 has an expression type and it is *not* that:

```rust
// cadabra-check/src/carrier.rs:155-180
pub enum CarrierExpr {
    Const(Rational), Cos, Sin, Cos2, Sin2, Radical(Radicand),
    Neg(Box<CarrierExpr>), Add(Box<_>, Box<_>), Sub(Box<_>, Box<_>), Mul(Box<_>, Box<_>),
}
```

Four observations:

1. **The atom set is domain-specific and closed.** `cos t`, `sin t`, `cos 2t`, `sin 2t`,
   `√r(t)`. `Cos2` is carried as a first-class atom *rather than* being rewritten to
   `2cos²t − 1`, deliberately, "so a general-position carrier's second-harmonic terms
   rationalize directly" (`:162-165`). A canonicalizing e-graph would rewrite exactly the
   thing this consumer needs left alone.
2. **Structural equality is a feature, not a limitation.** `certifies` is plain structural
   equality against the stored claim (`certificate.rs:52-54`). Hash-consing would make it
   cheaper — a genuine win — but *canonicalization* would break the tether semantics
   unless the mint and the rebuild canonicalize identically.
3. **There is real sheet-canonicalization subtlety already.** The rebuild side normalizes
   `−√ρ` to the canonical `+√ρ` form so one certificate attests both sheets, justified by
   the componentwise proof rule (`certificate.rs:44-58`). That is a hand-placed, carefully
   argued normalization — precisely the kind of thing an automatic simplifier would get
   wrong.
4. **Boxed, not interned.** `BiquadIdentityEvidence::claim` is `Box`ed only because it is
   four expressions and would inflate the enum (`certificate.rs:163-167`).

**Conclusion for resolvent L4:** the demand is for a *hash-consed expression container with
consumer-defined opaque atoms and structural (not canonical) equality*, plus an optional
canonicalizer the consumer opts into. An `egg`-first design where simplification is the
point would be rejected here. This consumer is evidence for `L4 = cheap identity + open
atom set`, and evidence *against* `L4 = simplifier`.

---

## 12. Adapter sketch, and an honest line count

The adapter already exists. It is `cadabra-core/src/exact/` plus
`cadabra-arrange/src/lift.rs`. Measured:

| File | Total | Test block starts | Doc-comment lines | Est. code |
|---|---|---|---|---|
| `exact/scalar.rs` | 428 | 308 | 114 | ~150 |
| `exact/radical.rs` | 410 | 263 | 88 | ~130 |
| `exact/algebraic.rs` | 606 | 432 | 176 | ~180 |
| `exact/interval.rs` | 218 | 176 | 64 | ~80 |
| `exact/mint.rs` | 259 | 144 | 66 | ~60 |
| `cadabra-arrange/src/lift.rs` | 238 | — | ~90 | ~90 |

The **pure delegation** portion — newtype + method forwarding + error mapping for ℚ,
`a+b√r`, `AlgebraicReal`, and `Interval` — is roughly **250 lines**. The rest is cadabra2's
own semantic-typing discipline: `WeierstrassParam`/`WeierstrassSample`/`WeierstrassSpan`
(~180 lines, entirely cadabra2 domain vocabulary — the half-angle key, the seam state, the
branch), the `mint` module (~60 lines, the "decide exactly, demote once, validate" guard),
and `GeometryKind`-tagged error construction.

**Verdict on the <200-line adapter test: passes, with the caveat named.** A resolvent
adapter for cadabra2 lands at ~250 delegation lines today; the ~50-line overage is entirely
error mapping and panic guards that a total, small-enum-error resolvent would remove
(§10 items 2, 3, 9). The domain vocabulary on top (`WeierstrassParam` et al.) is not
adapter code — it is geometry, it belongs in cadabra2, and it would exist regardless of
which CAS sat underneath.

Sketch of the delegation core:

```rust
// cadabra2 crate: cadabra-core/src/exact/
pub struct ExactScalar(resolvent::Rational);          // + Sign, cmp, arith, 3 exits
pub struct ExactRadical(resolvent::SqrtExt<Rational>); // + total cross-radicand order
pub struct AlgebraicNumber(resolvent::AlgebraicReal);   // + multiplicity, is_root_of, cmp
pub struct IntervalScalar(resolvent::Interval);         // + Scalar impl

impl From<resolvent::Error> for KernelError { /* ~20 lines, one arm per variant */ }
```

Nothing in that sketch mentions CAD. Nothing in resolvent would need to mention cadabra2.
The one place a naming conflict could arise — `Sign` — cadabra2 already resolves by
re-exporting the substrate's (`cadabra-core/src/exact/scalar.rs:53`, `pub use
lazy_exact::Sign;`), which is the right answer and costs zero lines.

**Where the two sides would conflict, and who eats it.** cadabra2's `Radicand` /
`CarrierExpr` / `BiquadTower` are domain algebra built *on top of* the substrate, not
adapters to it. If resolvent shipped a general number-field type, `BiquadTower` would
become a thin instantiation of it — but the *componentwise proof rule* over the basis
`{1,α,β,αβ}` and its soundness argument (`biquad.rs:20-42`) stays in cadabra2, because it
is a proof strategy about carrier residuals, not about fields. **The adapter eats the
difference; resolvent takes the general shape (a number field with a known basis and
multiplication), and the proof rule stays in the consumer.**

---

## 13. What would settle the open questions

I did not run cadabra2's benches, so §2's numbers are cadabra2's own recorded figures, not
mine. Specifically unresolved:

- **How much of the 2.448 ms is the `Real<Rational>` DAG vs the ℚ arithmetic vs the
  recursive Laplace?** Profiling `classify_quadric_intersection_exact` would say whether a
  Bareiss determinant alone closes most of the gap, or whether the lazy-DAG overhead
  dominates. That decides whether resolvent's L2 linear algebra needs modular methods at
  all for this consumer.
- **What fraction of real Boolean workloads actually reaches the exact arm?** `p_straddle = 0`
  is measured on a *generic* corpus, which is the easy case by construction. The
  `degeneracies/` corpus and the `xt-corpus-loop` (`docs/notes/design/xt-corpus-loop.md`)
  would give the honest escalation rate on real parts. Until then §2's "resolvent may cost
  milliseconds" is an inference, not a measurement.
- **Does the torus lane actually decompose into univariate resultants, or does it need
  genuine bivariate CAD?** `ssi-boolean-plan.md:158-161` asserts the former for F-T2
  (generator-line reduction gives a quartic-in-`b` per generator) but the "2-D preimage
  topology `G(a,b)=0`" clause suggests the latter. Working one plane×torus generic case by
  hand would settle whether demand #16 is real or whether #15 suffices.
- **Would cadabra2 actually migrate off `lazy-exact`?** Nothing in this evaluation shows
  it would. The realistic path is resolvent *subsuming* `lazy-exact` — same author, same
  license, same design lineage — rather than competing with it. That is a decision for the
  ecosystem, not a finding about cadabra2, and the brief explicitly defers the
  resolvent↔arrangements refactor question.
