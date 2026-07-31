# E3 — solverang as a resolvent consumer

**Status:** research input for the resolvent founding plan. Not a commitment.
**Verdict:** **would-benefit** — not blocked, not a priority driver, but with one
quantified, non-speculative win available from primitives resolvent needs internally
anyway.
**Method:** read `/home/dev/projects/solverang` (71,979 LOC under `crates/`, of which
~40k is `crates/solverang/src`), specifically the diagnosis, decomposition, branch,
and constraint-residual layers named in the brief. Every claim below cites a file and
line. Two microbenchmarks were run to avoid guessing at cost; both are labelled as
proxies with their limitations stated.

**The hypothesis under test** (from the brief): *exact algebra would be a new
capability for solverang rather than an unmet existing need.* I tried to falsify it in
both directions. **It survived, with one correction**: there is exactly one place where
exact algebra is not a new capability but a strictly better implementation of something
solverang already ships — generic rank of the constraint Jacobian — and it is both
faster and more correct than the incumbent. Everything else on the candidate list is
either a genuinely new capability (gated behind combinatorial work solverang has not
started), or belongs to interval arithmetic and numerical continuation rather than to a
CAS.

---

## 0. The seven findings, in order of how much they should move the roadmap

1. **The constraint model is already polynomial, by construction, and nobody did it on
   purpose for algebra's sake.** `sketch2d/constraints.rs:3-5` states the design rule:
   "**squared formulations** are used (e.g. `dx^2+dy^2 - d^2` instead of
   `sqrt(dx^2+dy^2) - d`) to eliminate the singularity at zero distance and produce
   smooth Jacobians". Rigid bodies carry a unit quaternion
   (`assembly/entities.rs:20-26`, `:127-134`), and the rotation matrix is quadratic in
   its components (`assembly/entities.rs:210`). A quaternion *is* the rational
   parametrization of SO(3) — the higher-dimensional analogue of `t = tan(θ/2)`. The
   brief asked whether the tangent-half-angle substitution is viable and what it costs.
   **It is already done, and it cost nothing, because it was adopted for Jacobian
   smoothness rather than for polynomiality.** 28 of the 31 concrete `impl Constraint`
   blocks are polynomial or rational-with-clearable-denominator. Exactly three are not
   (§4.3). This removes the single largest risk that would have made solverang
   *not* a CAS consumer.

2. **The one real demand is generic rank over `GF(p)`, and it is a *speedup*, not a new
   feature.** `graph/redundancy.rs:257-297` (`identify_dependent_blocks`) runs one SVD
   per constraint block on a monotonically growing row set — *k* SVDs of up to
   *m*×*n* — plus two more full SVDs (`:232`, `:319`). One row-echelon over `GF(p)` at
   a random evaluation point produces the same rank, the same dependent-row set, and
   additionally the dependency *certificate*, in a single pass. Measured proxies (§3.3):
   **18× faster at 200×200, 40× faster at 800×800**, against LAPACK — solverang uses
   nalgebra's own SVD (`Cargo.toml:27`), which is slower still, so the real gap is wider.

3. **`DiagnosticIssue::RedundantConstraint { implied_by: vec![] }` — `system.rs:803`.**
   solverang can say a constraint is redundant but never says *by what*. The field
   exists and is unconditionally empty. That vector is exactly the row-reduction
   certificate: the dependent Jacobian row expressed as a combination of pivot rows.
   It falls out of the same echelon as finding 2, for free, and it is the "diagnosis, not
   solving" differentiator that `IDEAS-crates.md` #24 says is the whole point of the
   project.

4. **The numerical-rank tolerance is absolute, not relative — but the fix is one line,
   not a CAS.** `graph/redundancy.rs:228-237` and `graph/dof.rs:210-219` both filter
   `s > tolerance` against a raw singular value; the tolerance is hard-coded to `1e-10`
   at both call sites (`system.rs:777`, `graph/dof.rs:103`). Roundoff noise in a
   computed singular value is ≈ `ε·σ_max ≈ 2.2e-16·σ_max`, so a genuinely rank-deficient
   Jacobian reads as full rank once `‖J‖ ≳ 4.5e5`. For squared-distance constraints the
   Jacobian entries are `2·Δcoordinate`, so this is reached at coordinates ~2.5e5 — a
   half-metre part expressed in micrometres. **I must be honest that this is not
   evidence for a CAS.** Replacing `s > tolerance` with
   `s > max(m,n)·ε·σ_max` fixes it in one line and captures most of the value.
   What no tolerance fixes is the *near*-degenerate configuration, where the true σ_min
   is genuinely small but nonzero — and that is the case where the question being asked
   ("is this constraint implied?") is a question about the *generic* system, not about
   the current floats. See §3.1.

5. **Structural rigidity is mostly not algebra — and where it is, it is exactly finding
   2.** In 2D, bar-joint minimal rigidity is Laman's condition, decided by a pebble game
   in O(V·E) with no arithmetic at all. In 3D, Laman fails (the double banana) and there
   is no known combinatorial characterization; the practical substitute *is* rank of the
   rigidity matrix at a random point — i.e. exact linear algebra over a finite field.
   Furthermore solverang's constraint vocabulary is not bar-joint: it includes angle,
   tangency, parallel, perpendicular, symmetric-about-line (`sketch2d/constraints.rs:770,
   312, 569, 671, 1545`), for which the pebble game does not apply at all and generic
   corank of the Jacobian is the only correct generic notion. **So the boundary is:
   2D bar-joint rigidity needs no algebra; everything else in solverang's actual
   vocabulary needs randomized exact rank, which is one echelon over `GF(p)`.**

6. **Every algebra demand beyond rank is gated behind combinatorial work solverang has
   not started.** `decomposition.rs:1-11` and `:153-248` are union-find connected
   components over the constraint-variable bipartite graph — nothing more.
   `graph/mod.rs:1-9` confirms the previous clustering layer was deleted. A typical CAD
   sketch is a *single* connected component, so a "cluster" is the whole sketch: hundreds
   of variables. Gröbner or resultant solving of a 200-variable quadratic system is not
   tractable by anyone's engine. Gröbner becomes tractable exactly when clusters are
   4-12 variables of degree ≤ 2 — which requires the Laman/DR-planner decomposition that
   `IDEAS-crates.md` #24 calls the real differentiator and that
   **solverang does not have**. The algebra demand is downstream of pure combinatorics.
   Scheduling consequence: resolvent must not build F4 *for solverang*.

7. **Branch tracking today is "pick the nearest root", the exact thing #24 forbids — and
   the fix is not algebra.** `solve/branch.rs:126-150` generates perturbed starting
   points from a deterministic `sin`-based scheme; `:62-89` picks the converged solution
   with smallest L2 distance to the previous configuration. `TODO.md:189` records the gap
   honestly ("Continuation/homotopy between the previous solution and a dragged target;
   branch continuity so sketches don't flip"). **Predictor-corrector path tracking on the
   Davidenko ODE is entirely numerical.** It needs the residual and the Jacobian, both of
   which solverang already has. A CAS contributes nothing to fixing branch flipping.
   Certifying the path (α-theory / alphaCertified) does need exact rational evaluation —
   but that is a research-grade addition on top of a feature that does not exist yet.

---

## 1. What solverang is, algebraically

| Layer | File | What it actually does |
|---|---|---|
| Constraint seam | `constraint/mod.rs:34-73` | `residuals(&ParamStore) -> Vec<f64>` and `jacobian(&ParamStore) -> Vec<(usize, ParamId, f64)>`. **Numeric only. There is no symbolic form anywhere on this trait.** |
| Decomposition | `decomposition.rs:153-248` | Union-find connected components. Not Laman, not DR-planning. |
| DOF | `graph/dof.rs:96-177` | `total_dof = ncols − rank(J)` via SVD; per-entity DOF from the column-restricted sub-Jacobian. |
| Redundancy/conflict | `graph/redundancy.rs:95-192` | SVD rank + incremental per-block rank test + left-null-space residual projection to split redundant from conflicting. |
| Pattern → closed form | `graph/pattern.rs:60-80`, `solve/closed_form.rs:135-290` | Matches on the constraint's **name string**, then reconstructs geometry numerically. |
| Reduction | `reduce/eliminate.rs:1-14` | Called "symbolic" in the module docs; it is one Newton step, `x = current − residual/J`. |
| Solvers | `solver/` | Newton–Raphson, Levenberg–Marquardt, trust region, BFGS/L-BFGS, ALM, sparse (`faer`), parallel (`rayon`), JIT (re-homed to `sinbad-anvil`, `jit.rs:1-19`). |
| Differentiation | `crates/macros/src/expr.rs:11-58, 86+` | A real symbolic-differentiation expression DAG — over `f64`, at proc-macro expansion time. |

### 1.1 The seam has no symbolic form, and it shows

The `Constraint` trait's design notes (`constraint/mod.rs:26-32`) enumerate what is
deliberately *not* on it: no dimension parameter, no geometry types, no column indices.
The omission that matters here is unstated: **no symbolic residual**. The consequence is
visible in the closed-form path.

`solve/closed_form.rs:149-247` must solve a circle-circle intersection, but the pattern
matcher (`graph/pattern.rs:66-80`) identified the constraints only by *substring match on
`Constraint::name()`* — `"distance"`, `"horizontal"`, `"angle"`. So the solver has a
constraint object that it knows is "a distance" and nothing else, and it recovers the
circle centre and radius by **numerically probing the residual function**:

```rust
// solve/closed_form.rs:216-238
let mut snap = store.snapshot();
snap.set(px, cur_x + 1.0);
let probe_residual = c.residuals(&snap)[0];
let delta = probe_residual - residual;
let denom = 2.0 * (delta - ux);
let d_actual = (1.0 - delta * delta) / denom;
```

Worse, the recovery algebra assumes an **unsquared** distance residual
(`closed_form.rs:200-201`: "For a standard distance constraint
`f = sqrt((x-cx)^2 + (y-cy)^2) - r`"), while the shipping `DistancePtPt` is squared
(`sketch2d/constraints.rs:16-20, 55`). The two disagree in both the gradient normalization
and the residual-to-radius conversion.

This is the single most vivid piece of evidence that solverang's constraints want a
symbolic form. **But it is not evidence for resolvent.** The fix is for `DistancePtPt` to
expose its own centre and radius, or for the pattern matcher to downcast to concrete types
instead of matching name substrings. That is a solverang refactor with zero external
dependencies. Recorded here as an anti-finding so the roadmap does not misread it.

---

## 2. Are the residuals polynomial? Yes, almost entirely.

This is the question that decides whether polynomial machinery applies at all. I read
every `impl Constraint` block: 18 in `sketch2d/constraints.rs`, 8 in
`sketch3d/constraints.rs`, 4 in `assembly/constraints.rs`, 1 in `assembly/entities.rs`.

### 2.1 Polynomial as written (no transformation needed)

| Constraint | Line | Residual | Degree |
|---|---|---|---|
| `DistancePtPt` | `sketch2d:18` | `(x2−x1)² + (y2−y1)² − d²` | 2 |
| `Coincident` | `sketch2d:239` | `[x2−x1, y2−y1]` | 1 |
| `Parallel` | `sketch2d:569` | `(x2−x1)(y4−y3) − (y2−y1)(x4−x3)` | 2 |
| `Perpendicular` | `sketch2d:671` | `(x2−x1)(x4−x3) + (y2−y1)(y4−y3)` | 2 |
| `Angle` | `sketch2d:770` | `(y2−y1)·cos a − (x2−x1)·sin a` | 1 |
| `Horizontal` / `Vertical` | `sketch2d:853, 908` | `y2−y1` / `x2−x1` | 1 |
| `Fixed` / `Midpoint` / `Symmetric` | `sketch2d:963, 1029, 1112` | affine | 1 |
| `EqualLength` | `sketch2d:1197` | `(x2−x1)²+(y2−y1)² − (x4−x3)²−(y4−y3)²` | 2 |
| `PointOnCircle` | `sketch2d:1295` | `(px−cx)² + (py−cy)² − r²` | 2 |
| `EqualRadius` | `sketch2d:1377` | `r1 − r2` | 1 |
| `TangentCircleCircle` | `sketch2d:440-441` | `d² − (r1±r2)²` | 2 |
| `Distance3D` | `sketch3d:23` | `Σ(Δ)² − d²` | 2 |
| `PointOnPlane` | `sketch3d:279` | `n·(p − p0)` | 1 |
| `Parallel3D` | `sketch3d:548-552` | two cross-product components | 2 |
| `Perpendicular3D` | `sketch3d:705` | `d1·d2` | 2 |
| `UnitQuaternion` | `assembly/entities.rs:127-134` | `qw²+qx²+qy²+qz² − 1` | 2 |
| `Mate` | `assembly:93-98` | `R(q₁)·v₁ + t₁ − R(q₂)·v₂ − t₂` | 3 |
| `CoaxialAssembly` | `assembly:228-234` | cross products of rotated directions | 4 |

The rotation-matrix entries are quadratic in the quaternion components
(`assembly/entities.rs:210-226`), so a rotated local point is degree 2 and a cross
product of two rotated directions is degree 4. **All of assembly is polynomial of degree
≤ 4 in the 7 parameters per body**, which is precisely the formulation the algebraic-
kinematics literature uses.

The `Angle` case deserves a note. `sketch2d/constraints.rs:782-783, 807-808` bakes
`sin(a)` and `cos(a)` into `f64` fields **at construction time**; the angle is constraint
*data*, not a solver variable. So no half-angle substitution is needed — but the
coefficients are then transcendental reals rounded to `f64`. Every `f64` is exactly a
dyadic rational, so the polynomial exists over ℚ; it is just the polynomial for an angle
infinitesimally off 30°. For rank and genericity that is harmless. For "is this system
*exactly* satisfiable" it silently answers a different question. §5 records this as a
conflict and who eats it.

### 2.2 Rational, polynomial after clearing denominators

| Constraint | Line | Residual | After clearing |
|---|---|---|---|
| `DistancePtLine` | `sketch2d:106, 185` | `cross²/len_sq − d²` | `cross² − d²·len_sq` (deg 4) |
| `TangentLineCircle` | `sketch2d:314` | `(signed_dist)²  − r²` | same shape (deg 4) |

Clearing `len_sq` introduces the spurious component `len_sq = 0` (a degenerate zero-length
line). Any exact treatment must saturate it out or carry it as a known extraneous factor.
This is standard and cheap for a *rank* computation (a random evaluation point almost
surely avoids it), and expensive for a *solving* computation.

### 2.3 Genuinely not polynomial — exactly three sites

- **`Insert::residuals`, `assembly/constraints.rs:520-522`**:
  `len = len_sq.sqrt().max(1e-15); axial = dot/len − offset`. A square root of a
  quaternion-derived quantity. Squaring clears it at the cost of losing the sign, and the
  `.max(1e-15)` clamp is not an algebraic operation at all.
- **`Gear`, `assembly/constraints.rs:545-560, 633-635`**: `θ = 2·atan2(sin_half, cos_half)`
  and the residual is `θ₁·ratio − θ₂`. This is genuinely transcendental *and* multivalued
  (`atan2` has a branch cut). It cannot be polynomialized: the constraint's meaning depends
  on the winding number, which is not an algebraic function. Note that `Gear` and `Insert`
  are also the two constraints whose Jacobians are **finite-differenced**
  (`assembly/constraints.rs:370, 534, 747`) rather than analytic.
- **`Spline2D`, `sketch2d/entities.rs:510`**: piecewise. Piecewise is not one polynomial;
  any exact treatment is per-segment with combinatorial case analysis.

**Verdict on §2:** ~90% of the constraint vocabulary is polynomial of degree ≤ 4 with
dyadic-rational coefficients. Polynomial-system machinery *applies*. That is a necessary
condition, not a sufficient one.

---

## 3. The one demand I would actually accept: generic rank over `GF(p)`

### 3.1 Why "exact rank" as normally imagined is a trap

The obvious proposal — "convert the Jacobian to exact rationals and compute exact rank" —
**answers the wrong question**, and I want that stated plainly before any of this reaches
a roadmap.

`build_dense_jacobian` (`graph/redundancy.rs:199-223`) evaluates
`Constraint::jacobian(store)` at the *current floating-point configuration*. Every entry
is an `f64`, hence exactly a dyadic rational. The exact rank of such a matrix is, with
probability essentially 1, **full** — floating-point roundoff has already destroyed the
algebraic relation that made it rank-deficient. Exact linear algebra on a float matrix
converts a *useful approximation* into a *useless certainty*.

The question the CAD user is asking is not "is this matrix singular at these particular
floats". It is one of two other questions:

- **(Generic)** "Is this constraint implied by the others *no matter where the geometry
  sits*?" — i.e. rank of the Jacobian as a matrix over the function field
  ℚ(x₁…xₙ). This is the correct definition of a redundant constraint in CAD, and it is
  what `RedundancyAnalysis` is morally trying to compute.
- **(Configurational)** "Are my three points *exactly* collinear as I authored them?" —
  meaningful at user-authored coordinates (which really are exact decimals: `10.0`,
  `0.0`, `45.0`), meaningless at post-Newton coordinates.

**Their disagreement is itself the diagnostic** that `IDEAS-crates.md` #24 verification
item 1 asks for: "DOF counting: structural prediction vs numerical rank of the Jacobian.
Disagreement means a degenerate (non-generic) configuration — which is itself a valuable
diagnostic output, not a bug." Generic rank supplies the structural half, exactly.

### 3.2 How generic rank is computed, and why it is cheap

Generic rank of a polynomial matrix is computed by evaluating at a random point and
taking the rank there — the Schwartz–Zippel argument. Concretely:

1. Pick a word-size prime `p` (≈ 2³¹).
2. Pick a uniform random point `α ∈ GF(p)ⁿ`.
3. For each constraint, reduce its residual polynomial's coefficients mod `p`, take
   `∂f/∂xⱼ` symbolically, evaluate at `α`.
4. Row-reduce the resulting `m × n` matrix over `GF(p)`.

Failure probability: the non-vanishing `r × r` minors have degree ≤ `r·d` with `d ≤ 4`
(§2), so a wrong (too-low) answer has probability ≤ `r·d/p`, about `1e-6` for `r = 500`.
Repeating with a second prime squares it. **No bignums appear anywhere.** All arithmetic
is single-word modular.

This is the "modular methods everywhere" decision of `IDEAS-crates.md` #4, but it lands
here as a *public API requirement*, not an internal strategy: solverang needs the modular
layer callable directly, not hidden behind CRT and rational reconstruction. It never wants
the rational answer.

### 3.3 Cost, measured

`identify_dependent_blocks` (`graph/redundancy.rs:257-297`) runs one `compute_rank` — a
full SVD — per constraint block, on an accumulated row set that grows to the full matrix.
`analyze_redundancy` then runs a full SVD for the global rank (`:134`) and a third for the
null-residual projection (`:319`).

I measured two proxies. **Both are proxies and I state their limits.**

*Proxy A — the incremental-SVD loop*, NumPy/LAPACK `dgesdd`, multithreaded, on dense
random matrices, k blocks:

| m = n | k | incremental loop | two full SVDs |
|---|---|---|---|
| 40 | 30 | 6.4 ms | 1.1 ms |
| 100 | 80 | 44.9 ms | 55.2 ms |
| 200 | 150 | 522 ms | 130 ms |
| 400 | 300 | 5.53 s | 333 ms |
| 800 | 600 | 74.5 s | 639 ms |

*Proxy B — one `GF(p)` row echelon reporting rank **and** the dependent-row set*, naive
Rust, `p = 2³¹−1`, `%` after every operation, no delayed reduction, no SIMD, single-
threaded, dense random:

| m = n | echelon |
|---|---|
| 40 | 0.2 ms |
| 100 | 3.7 ms |
| 200 | 29.3 ms |
| 400 | 228 ms |
| 800 | 1.86 s |
| 2000 | 28.7 s |

**Limits of the comparison, honestly:** proxy A uses LAPACK, which is faster than
nalgebra's Golub–Reinsch SVD by a typical factor of 3-10× — so solverang's real timings
are *worse* than proxy A. Proxy B is deliberately unoptimized; Montgomery or delayed
reduction typically buys 5-10×. Both use dense random matrices, while constraint Jacobians
are extremely sparse (4-12 nonzeros per row); sparsity helps `GF(p)` elimination more than
it helps SVD, but fill-in erodes the advantage. **Net: the 18×-at-200 / 40×-at-800 figure
is a conservative lower bound on the speedup, and it comes with exactness and a
dependency certificate attached.**

What would settle it properly: port `identify_dependent_blocks` to a `GF(p)` echelon
behind a feature flag and run the existing `sketch2d_property_tests.rs` (1,945 lines) and
`solver_megatest.rs` (1,639 lines) corpora with both paths, comparing verdicts and wall
time on the actual sparsity patterns.

### 3.4 What this buys that a tolerance fix does not

- **Exactness at near-degenerate configurations.** No σ threshold separates "genuinely
  dependent" from "nearly dependent"; generic rank does not have the question.
- **`implied_by`.** `system.rs:803` ships an unconditionally empty vector. The echelon's
  transform gives the dependent row as `Σ cⱼ · pivot_rowⱼ`, naming the implicating
  constraints. This is the diagnosis that `IDEAS-crates.md` #24 says is the entire UX gap.
- **A single pass instead of k passes.** Structural, not constant-factor.
- **3D rigidity classification** (`TODO.md:215-221`, currently blocked on nothing but
  effort): "scalable" means the uniform-dilation generator lies in the Jacobian's null
  space. Over `GF(p)`, that is one membership test against the computed null space, exact.

---

## 4. The candidate demands, accepted and rejected

| # | Candidate (from the brief) | Verdict | Reason |
|---|---|---|---|
| 1 | Exact rank / rigidity | **Accept** | §3. Replaces a working feature with a faster, exact one that also emits certificates. Not blocking. |
| 2 | Root counting via **Bézout** | **Reject from core** | It is `Π deg(fᵢ)`. Falls out of `MPoly::total_degree`; needs no resolvent feature. Also uselessly loose: 200 quadratics ⇒ 2²⁰⁰. |
| 3 | Root counting via **BKK / mixed volume** | **Reject from core** | Mixed volume of Newton polytopes is *convex geometry*, not algebra. It belongs in a polytope crate. Putting it in a CAS is scope creep with no second consumer. |
| 4 | Gröbner / resultant solving of irreducible clusters | **Reject as a driver** | Gated behind a decomposition solverang does not have (§0.6). And there is **no evidence in the repo of Newton failing to start**: `TODO.md` records no such failure mode, and `solver/auto.rs:1-542` already ladders NR → LM → trust region. A capability with no observed need. |
| 5 | Homotopy start-system construction | **Reject** | Total-degree start systems need only degrees; polyhedral ones need mixed volume (see #3). The tracking itself is a Davidenko-ODE predictor-corrector — pure `f64` numerics, needing only the residual and Jacobian solverang already has. |
| 6 | Exact verification of a converged solution via exact predicate evaluation | **Reject** | The converged solution is a float; its residual is essentially never exactly zero, so an exact predicate evaluation just reports a small nonzero number more slowly. The correct tool is an **interval Newton / Krawczyk** existence-and-uniqueness test in a box around x̂, which is directed-rounding `f64` arithmetic and not a CAS. `certify_final_residuals` (`system.rs:664-730`) is a tolerance check and should become a Krawczyk test, not an exact one. |
| 7 | Certified branch tracking | **Speculative** | alphaCertified-style certification does use exact rational arithmetic on `f`, `Df`, and higher-derivative bounds — the same primitives as #1 plus `MPoly` evaluation over ℚ. But it certifies a feature (`TODO.md:189`) that does not exist yet. Urgency: speculative. |
| 8 | *(new)* Dependency certificate for `implied_by` | **Accept** | §3.4. Same echelon, no extra cost, fills a shipped-but-empty field. |
| 9 | *(new)* Ideal membership as the exact definition of "redundant" | **Eventual** | `g ∈ ⟨f₁…f_k⟩` is the *correct* statement of constraint redundancy; the Jacobian-rank test is its linearization. Needs Gröbner, hence small clusters, hence #4's gate. |
| 10 | *(new)* Nullstellensatz certificate of inconsistency | **Eventual, and the most attractive new capability** | `differential_oracle.rs:313` pins a known divergence: solverang reports `Solved` for a system D-Cubed calls `NOT_SATISFIED`, because the least-squares solver converges to a nonzero minimum. `1 ∈ ⟨f₁…f_k⟩` is a *proof* of unsatisfiability with a checkable witness — stronger diagnosis than the licensed reference oracle produces. This is precisely a **new capability, not an unmet need**, which is the brief's hypothesis confirmed. |

### 4.1 The falsification attempts that failed

I looked for something solverang is genuinely *blocked* on that only algebra unblocks.

- **`TODO.md:215-221`, rigidity classification (RIGID/SCALABLE/UNI_SCALABLE/FLEXIBLE).**
  Not blocked: a scalable body-set has the uniform-dilation vector in the Jacobian null
  space; one matrix-vector product answers it. Algebra makes it exact, not possible.
- **`TODO.md:206-214` / `differential_oracle.rs:313`, `solve().status` over-reporting.**
  Not blocked: the fix is a status-classification change (`Solved` should mean "all
  residuals below tolerance"), stated in the TODO itself.
- **`TODO.md:189`, branch flipping under drag.** Not blocked by algebra: continuation is
  numerical (§0.7).
- **`TODO.md:193`, "minimal conflicting constraint sets (not just broad redundancy
  groups)".** `build_conflict_groups` (`graph/redundancy.rs:343-405`) currently unions
  conflicting constraints that merely *share a parameter* — which is a connectivity
  heuristic, not a minimal unsatisfiable core. Minimal cores are an *SMT/MaxSAT* problem
  (deletion-based MUS extraction over the rank oracle), not a CAS problem. The rank oracle
  it calls repeatedly would be resolvent's — which strengthens finding 2 (a MUS loop calls
  rank O(k) times, making the 18-40× compound) but adds no new algebraic demand.

None of these produced a blocking algebra need. **Hypothesis holds.**

### 4.2 Does the analytic differentiation overlap resolvent L4?

Partially, and the overlap is not worth acting on. `crates/macros/src/expr.rs:11-58`
defines a genuine expression AST with `differentiate` (`:86+`) and `simplify` (`:255+`).
It is a *different mechanism* from resolvent L4 in three ways that all matter:

1. It runs **at proc-macro expansion time**, producing Rust tokens. Resolvent L4 is a
   runtime hash-consed DAG.
2. Its leaves are `f64` and `RuntimeConst(String)` — it has no coefficient ring.
3. It is closed over transcendentals (`Sin`, `Cos`, `Tan`, `Atan2`, `Ln`, `Exp`, `Asin`,
   `Acos`, `Sinh`, `Cosh`, `Tanh`, `Abs`, `Sqrt`), which a polynomial-ring L1 by
   definition is not.

Its `simplify` is a single bottom-up constant-folding pass with the obvious identities
(`x+0`, `x·1`, `x·0`, `x^0`, `x^1`) and no cross-term collection, no common-subexpression
elimination, no factoring. An e-graph would beat it. But the consumer of that output is a
Cranelift JIT (`jit.rs:1-19`, re-homed to `sinbad-anvil`) which does its own CSE and
instruction selection, so the marginal value of better expression simplification is
whatever the JIT does not already recover — unmeasured, and plausibly near zero.

**Anti-finding: resolvent should not pitch L4 to solverang.** The expression layer
solverang has is compile-time, transcendental, and downstream of a real compiler. Replacing
it would be a large refactor for an unquantified win.

### 4.3 Would exact algebra even be tractable at realistic sketch sizes?

For **rank**: yes, comfortably — §3.3 shows `GF(p)` echelon beating the incumbent at every
size measured up to 800, and still running (28.7 s, unoptimized, dense) at 2000×2000,
which is far past any interactive sketch.

For **solving**: no. Take the whole-sketch cluster that solverang's connected-component
decomposition actually produces — 100 entities is ~250 parameters, ~250 equations of degree
2. Gröbner basis computation over ℚ or `GF(p)` for a generic 250-variable quadratic system
is not within reach of any engine, GPL or otherwise. Even msolve, the fastest thing in
existence for this, targets tens of variables on structured systems.

For **solving after a proper decomposition**: yes, and easily. A ruler-and-compass
irreducible cluster is 2-4 entities, 4-12 variables, degree ≤ 2. Bézout ≤ 2¹² = 4096, BKK
much smaller, and F4 over `GF(p)` on such a system is sub-millisecond. **The tractability
boundary is exactly the decomposition boundary**, which is why finding 6 is a scheduling
finding rather than a technical one.

---

## 5. Where solverang conflicts with resolvent's natural shape, and who eats it

| Conflict | Resolvent's general shape | Who absorbs it |
|---|---|---|
| Coefficients arrive as `f64` (`Angle::sin_a`, every `target_sq`, `Gear::ratio`) | Coefficients are exact ℚ | **Adapter.** Every `f64` is exactly a dyadic rational; `Rational::from_f64_exact` is lossless. Resolvent must expose it but must not offer a "round to a nice rational" heuristic — that would be a lie about which system is being analyzed. The adapter documents that `sin(30°)` becomes the dyadic rational nearest it. |
| Variable counts run to hundreds-to-thousands per sketch; packed-exponent monomial representations optimize for few variables and high degree | L1 packs exponents for Gröbner speed | **Adapter.** No single residual touches more than ~12 parameters (`Parallel3D` has 12, `assembly::Insert` has 14). The adapter creates a small polynomial ring per constraint kind and maps local variable indices back to `ParamId`s. **But this imposes a hard API requirement**: the variable count must be a runtime property of a ring/context value, not a const-generic burned into the polynomial type, or the adapter cannot instantiate rings from data. See §7. |
| Interactive latency budget (a diagnosis runs per edit, `system.rs:765`); bignum rational arithmetic is unaffordable | Modular methods are an internal strategy behind Las Vegas rational reconstruction | **Resolvent.** The `GF(p)` layer must be public and directly callable, returning a `GF(p)` answer, with no forced lift to ℚ. Resolvent takes this cost because it is the general shape anyway (`IDEAS-crates.md` #4 already calls modular methods "*the* structural decision") and because any consumer doing randomized generic-rank or Schwartz-Zippel testing needs the same thing. |
| Three transcendental sites (`Gear` `atan2`, `Insert` `sqrt`, `Spline2D` piecewise) | Resolvent is purely algebraic | **Adapter, by exclusion.** The adapter returns `None` for these three and the caller degrades to the numeric path for any cluster containing one. Resolvent must not grow an `Atan2` node. |
| Denominators (`DistancePtLine` `cross²/len_sq`) | Polynomials, not rational functions | **Adapter.** It clears denominators and records the extraneous factor. Resolvent needs no `RatFunc` type for this consumer. |

---

## 6. The adapter sketch, and whether it passes the 200-line test

The adapter lives in solverang (e.g. `crates/solverang/src/exact/`), behind an optional
feature, and it splits cleanly into two halves that must be judged separately.

### 6.1 Half one — the seam (resolvent-facing). ~35 lines.

```rust
use resolvent::{Fp, FpElem, MPoly, Q, Ring};
use crate::{constraint::Constraint, id::ParamId, param::SolverMapping};

/// Consumer-side trait. Lives in solverang. Resolvent knows nothing about it.
pub trait AlgebraicConstraint: Constraint {
    /// Parameters this constraint's polynomial is expressed over, in ring-variable order.
    fn poly_vars(&self) -> &[ParamId];
    /// Residuals as polynomials over `poly_vars()`. `None` = not algebraic
    /// (Gear, Insert, Spline2D).
    fn residual_polys(&self) -> Option<Vec<MPoly<Q>>>;
}

/// Generic rank + dependent rows + the `implied_by` certificate.
pub fn generic_rank(
    cs: &[(usize, &dyn AlgebraicConstraint)],
    mapping: &SolverMapping,
    seed: u64,
) -> Option<GenericRank> {
    let fp = Fp::new(2_147_483_647);                     // R1
    let n = mapping.len();
    let point: Vec<FpElem> = fp.random_point(n, seed);   // R2

    let mut rows = Vec::new();
    let mut row_owner = Vec::new();
    for (idx, c) in cs {
        let polys = c.residual_polys()?;                 // bail on transcendental
        let cols: Vec<usize> = c.poly_vars().iter()
            .map(|p| mapping.param_to_col[p]).collect();
        let local: Vec<FpElem> = cols.iter().map(|&j| point[j]).collect();
        for f in &polys {
            let f_p = f.map_coefficients(|q| fp.reduce(q))?;          // R3
            let mut row = vec![fp.zero(); n];
            for (k, &j) in cols.iter().enumerate() {
                row[j] = f_p.derivative(k).evaluate(&local);          // R4, R5
            }
            rows.push(row);
            row_owner.push(*idx);
        }
    }
    let ech = resolvent::linalg::row_echelon(&fp, rows);              // R6
    Some(GenericRank {
        rank: ech.rank,
        dependent: ech.dependent_rows.iter().map(|&r| row_owner[r]).collect(),
        implied_by: ech.dependency_certificates(&row_owner),          // R7
    })
}
```

**35 lines, and it touches seven resolvent items (R1-R7).** That is the seam, and it
passes the test with room to spare.

### 6.2 Half two — transcription. ~250-450 lines, and it is not resolvent's problem.

`residual_polys` must be implemented for 31 concrete constraint types. Each is 5-12 lines
of `MPoly` construction using nothing but `var(i)`, `constant(q)`, and `+ - *`:

```rust
impl AlgebraicConstraint for DistancePtPt {
    fn poly_vars(&self) -> &[ParamId] { &self.params }        // [x1, y1, x2, y2]
    fn residual_polys(&self) -> Option<Vec<MPoly<Q>>> {
        let (x1, y1, x2, y2) = (v(0), v(1), v(2), v(3));
        let dx = &x2 - &x1;  let dy = &y2 - &y1;
        Some(vec![&dx * &dx + &dy * &dy - MPoly::constant(Q::from_f64_exact(self.target_sq)?)])
    }
}
```

**Judgement on rule 4.** The acceptance criterion is "the adapter can be written in
roughly under 200 lines, with zero changes to resolvent". The *resolvent-facing seam* is
35 lines and needs zero resolvent changes beyond R1-R7 (all of which §7 argues are general
CAS primitives, not solverang accommodations). The *total* adapter is 300-500 lines,
dominated entirely by mechanical per-constraint transcription that scales with solverang's
constraint vocabulary and is independent of resolvent's API shape. I record both numbers
rather than picking the flattering one. **The test as intended — "does the consumer force
resolvent to expose something bespoke" — passes.**

### 6.3 The two solverang-side prerequisites, so the roadmap is not surprised

1. `AlgebraicConstraint` must be implemented 31 times. That is solverang's work.
2. `mapping.param_to_col` is currently a `HashMap<ParamId, usize>` field
   (`graph/redundancy.rs:213`); indexing it per Jacobian entry is fine but the adapter
   should hoist it. Cosmetic.

Neither touches resolvent.

---

## 7. API pressure on resolvent

Each item is stated as a general primitive, with the argument for why it earns core status
independent of solverang. Items that only solverang wants are marked as such and
recommended **out** of core.

| Ref | Layer | Required surface | General-primitive justification |
|---|---|---|---|
| R1 | L0 | Prime field with a **runtime** modulus: `Fp::new(p) -> Fp`, `Fp::zero/one/reduce/inv`, elements `Copy` and word-sized | Every modular method needs it; F4 needs it; multi-prime CRT needs the modulus to be data, not a type parameter. Non-negotiable and already implied by `IDEAS-crates.md` #4. |
| R2 | L0 | Uniform random element / random point generation over `GF(p)` from a caller-supplied seed | Schwartz-Zippel testing, sparse interpolation, and modular GCD all need it. Must be seedable so consumers get determinism (`solverang` has a hard determinism requirement, `TODO.md:236-240`). |
| R3 | L0/L1 | Coefficient homomorphism `MPoly<Q> -> MPoly<GF(p)>`, fallible when `p` divides a denominator | The core of every modular algorithm. Fail-closed on bad primes, not silent. |
| R4 | L1 | `MPoly::derivative(var) -> MPoly` | Standard. Also lets the adapter transcribe *only the residual* and let resolvent produce the Jacobian — halving the transcription in §6.2 and removing a class of hand-differentiation bugs. |
| R5 | L1 | `MPoly<R>::evaluate(&[S]) -> S` where `S: Ring` and there is a hom `R -> S` | Needed for modular evaluation, for interpolation-based GCD, for sign evaluation at algebraic points. |
| R6 | L2 | Matrix over a field with `row_echelon()` returning **rank, pivot rows, and dependent rows** — not just a rank integer | F4 *is* row reduction over `GF(p)`; the Sylvester-matrix route to resultants is a determinant/echelon; solving linear systems over ℚ by modular methods is an echelon. This is already load-bearing internally; the ask is that it be **public**. |
| R7 | L2 | The echelon's transform, so a dependent row can be reported as `Σ cⱼ · pivot_rowⱼ` | Same object as a cofactor representation `f = Σ hᵢgᵢ`, which `IDEAS-crates.md` #4's verification section already requires for Gröbner self-certification. The certificate is not an extra feature; it is the same discipline applied one layer down. |
| R8 | L0 | `Rational::from_f64_exact(f64) -> Option<Rational>` (exact dyadic; `None` on NaN/∞) and the inverse `to_f64` with a documented rounding mode | Any CAS with a numeric boundary needs it. Deliberately *not* a "nice rational" heuristic — that would silently change the problem. |
| R9 | L1 | Ring/variable count as a **runtime context value**, not a const generic | §5. An adapter that builds rings from data cannot use `MPoly<Q, 4>`. This is a real one-way-door constraint on the L1 representation and should be settled before fan-out, as the brief requires. |
| R10 | L1 | `MPoly::total_degree()` | Trivial, and it is all that Bézout counting needs — which is why Bézout does not deserve its own API (§4, #2). |

**Explicitly recommended OUT of core, on solverang's evidence:**

- Mixed volume / Newton polytopes / BKK — convex geometry, no second consumer, and the
  brief's rule 2 is not met.
- Interval arithmetic and Krawczyk/interval-Newton — this is what §4 #6 actually wants,
  and it is a numerics library, not a CAS. If resolvent ships it, it should be because
  root isolation needs it internally, not because a consumer asked.
- Any `Atan2`/`Sqrt`/piecewise expression node — §2.3 and §4.2.
- An expression-DAG pitch to this consumer at all — §4.2.

---

## 8. Verdict

**would-benefit.**

Reading it against the enum, deliberately strictly:

- Not **blocked-today**: every diagnosis solverang ships works, and the S5 divergence,
  the rigidity gap, and the branch-flipping gap all have non-algebraic fixes (§4.1).
- Not **strong-consumer**: exactly one accepted demand, and it improves an existing
  feature rather than enabling a new product capability. Two of the seven candidate
  demands in the brief were rejected outright as belonging to convex geometry and
  interval arithmetic respectively.
- Not **weak-consumer**: the accepted demand is real, quantified at 18-40× with
  exactness and certificates attached, and it lands on `implied_by` — the field that
  `IDEAS-crates.md` #24 identifies as the entire differentiator, currently shipped empty.
- Not **not-a-consumer**: §2 establishes that ~90% of the constraint vocabulary is
  polynomial of degree ≤ 4, so the machinery genuinely applies.

**The brief's hypothesis survives.** Exact algebra is predominantly a *new capability*
for solverang — Nullstellensatz certificates of inconsistency (§4 #10) being the most
attractive one, since it would beat the licensed D-Cubed reference on the exact
observable where solverang currently diverges from it. That makes solverang a legitimate
source of API pressure on R1-R10 and **not** a priority driver for resolvent's roadmap.

**The scheduling consequence, stated so it cannot be misread:** the demands solverang
would place on resolvent's *interesting* layers — F4 (L2), algebraic numbers (L3),
expression DAG (L4) — are all either gated behind a Laman/DR-planner decomposition
solverang has not begun (§0.6, §4.3), or actively rejected (§4.2, §4 #6). What solverang
wants is L0 and a public, callable `GF(p)` linear-algebra primitive. **Do not build F4 for
solverang.** If R1-R7 exist because another consumer or resolvent's own internals needed
them, solverang's adapter is 35 lines of seam over an afternoon of transcription. If they
do not, solverang alone does not justify them — which is rule 2 working correctly.

---

## 9. What would change this verdict

Concretely, so it is checkable rather than rhetorical:

1. **solverang ships the pebble-game decomposition** (`IDEAS-crates.md` #24 M0). Clusters
   drop from ~250 variables to 4-12, and F4/resultant solving of irreducible clusters
   moves from intractable to sub-millisecond. That single change would move solverang from
   *would-benefit* to *strong-consumer*, and it is entirely within solverang's control.
2. **A measured case where Newton demonstrably cannot start** on an irreducible cluster
   that a homotopy or Gröbner solve reaches. `TODO.md` records no such case today. One
   reproducible instance would upgrade §4 #4 from rejected to real.
3. **A user-visible incident traced to the absolute rank tolerance** at
   `graph/redundancy.rs:236`. Fix the tolerance first (one line); if misdiagnosis persists
   at near-degenerate configurations, that is direct evidence for §3 and worth a
   changelog entry.
4. **The `GF(p)` port benchmarked on real sparsity** (§3.3, "what would settle it"). If
   the sparse structure makes nalgebra's SVD competitive, the 18-40× collapses and the
   accepted demand weakens to "exactness only".
