# R2 — What a real consumer actually calls

**Status:** research input for the resolvent founding plan. Not a commitment; a
requirements derivation.
**Method:** read the shipping source of `/home/dev/projects/arrangements`
(13,064 LOC in `crates/arrangements/src`, 3,602 LOC in `crates/lazy-exact/src`)
and counted what it calls, then read the prospective consumers in
`/home/dev/projects/IDEAS-crates.md` (#34 FEM, #12 SMT NRA, #27 medial axis,
#24 constraint solver) for pressure on the API shape.
**Constraint honored throughout:** resolvent does not depend on arrangements and
does not import its traits. Everything below is stated as *what resolvent must
expose*, never as *what resolvent must know about geometry*.

---

## 0. The five findings that change the design

1. **The entire algebraic surface a shipping 17k-LOC exact geometry engine
   consumes is about 25 functions.** All of `lazy-exact`'s algebra that
   `arrangements` touches is re-exported in six names
   (`crates/lazy-exact/src/lib.rs:82-83`): `QPoly`, `RealRoot`, `isolate_roots`,
   `sign_radical1`, `sign_radical2`, `SqrtExt`. resolvent's first useful release
   is those 25 functions *generalized*, not a CAS.

2. **The consumer never does algebraic-number arithmetic.** `RealRoot`
   (`crates/lazy-exact/src/roots.rs:317-612`) has no `add`, no `mul`, no `div` —
   and four curve families ship without them. Points are carried as *(isolated
   root ξ, a representation over ξ)*, never as a materialized element of ℚ(ξ).
   Field arithmetic in ℚ(α) is therefore **not** must-have for the geometry
   consumer. It *is* must-have for SMT NRA. That asymmetry is the single
   biggest scoping win available.

3. **What the consumer does instead is hand-assemble polynomial algebra.**
   Across `crates/arrangements/src/geoms/*.rs`: 110 calls to `QPoly::mul_poly`,
   33 `sub_poly`, 31 `scale`, 24 `add_poly`. Three separate hand-rolled
   resultants (`conics.rs:276-287`, `spherical_circle.rs:589-597`,
   `sine_radical.rs:614-621`) and roughly 150 lines per family of by-hand
   radical-ladder derivation (`conics.rs:110-121, 292-313, 317-324, 631-663`).
   resolvent's job is to replace ~200 by-hand polynomial-arithmetic calls with
   ~5 named operations.

4. **None of the "non-polynomial" curve families need transcendental algebra.**
   `sine_waves`, `sine_radical`, and `spherical_circle` all rationalize —
   Weierstrass `t = tan(u/2)` for the first two
   (`sine_waves.rs:1-21`, `sine_radical.rs:19-38`), the longitude key
   `W = tan(u/2) = (r−x)/y` plus monotone `acos` for the third
   (`spherical_circle.rs:16-40`). The transcendental coordinates are *never
   compared*; a rationalizing change of variable chosen by the **consumer**
   reduces every predicate to univariate algebra. resolvent can be purely
   algebraic and still serve every shipped family. (§5 surfaces where this
   stops being true.)

5. **Correctness here is easy and speed is the whole game — and the speed is
   decided by a representation choice made before any agent fans out.** The
   arrangements inner loop is `RealRoot::sign_of` (`roots.rs:497-524`), which
   per invocation runs `is_root_of` → a **monic Euclid gcd over ℚ**
   (`roots.rs:169-182`) and then loops `descartes_in` (`roots.rs:270-288`, two
   affine compositions + a reversal, O(n²) rational ops with growing
   denominators). At degree ≤ 4 with small coefficients this is fine. At the
   degree-20, 500-bit-coefficient resultants that arbitrary-degree curves
   produce it is fatal. `conics.rs:459-460` and `conics.rs:565-567` additionally
   recompute the resultant *and* re-isolate its roots on **every**
   `cmp_y_right_of` and **every** `intersect` call, with no cache.

---

## 1. The `Geometry` trait, method by method, reduced to algebra

Source: `/home/dev/projects/arrangements/crates/arrangements/src/geometry.rs`.
"Algebraic operation" is what a *general* (arbitrary-degree, curve given as
`f(x,y) = 0`) implementation would need — not what the degree-4 code does today.

### 1.1 `trait Geometry` (geometry.rs:22-107)

| Method | Line | Algebraic operation required |
|---|---|---|
| `cmp_x(p, q)` | 31 | Total order on real algebraic numbers, with **equality decided algebraically** (gcd of defining polys + sign-change certificate), never by "intervals got close". Today: `RealRoot::cmp_root`, roots.rs:549-596. |
| `cmp_xy(p, q)` | 34 | The above, then: given two points over the *same* abscissa α, sign of `y_p − y_q` where each `y` is an element of a real extension of ℚ(α). Today: `AlgPoint::cmp_y_same_x` (conics.rs:108-122) → `sign_radical2`. General: **sign of an element of a real extension tower over an algebraic point**. |
| `min_endpoint` / `max_endpoint` | 38, 41 | None (structural). |
| `is_vertical(c)` | 44 | None for these families — arcs are pre-split at vertical tangents (conics.rs:419-421). General: decide whether a branch is vertical ⇔ `∂f/∂y ≡ 0` on a component. |
| `cmp_y_at_x(p, c)` | 49 | Sign of `f_c(α, β)` where `α = p.x`, `β = p.y` is in an extension over α — **plus** branch disambiguation (which of the up-to-`deg_y` branches of `c` sits at α). Today: conics.rs:423-440, exploiting that degree 2 in y means exactly two branches. General: **fiber structure of a curve over an algebraic abscissa** (CGAL's status line). |
| `cmp_y_right_of(c1, c2, at)` | 54 | The interesting one. The shipping code **refuses derivative towers** and instead builds a *certified rational witness* strictly between `at.x` and the next event, then compares two one-root numbers there (conics.rs:442-476, witness loop at conics.rs:357-380). This requires: (a) a **superset of the next event abscissas** = roots of `Res_y(f,g)` and of `Res_y(f, ∂f/∂y)`; (b) **a rational strictly between two algebraic numbers**, obtained by refining both until separated; (c) cheap evaluation of a branch at a *rational* abscissa (a `SqrtExt` today — a degree-2 radical, not a general algebraic number). |
| `cmp_y_left_of` | 67 | Provided by default from the above. No new algebra. |
| `eq_points` | 101 | From `cmp_xy`. |
| `eq_curves(c1, c2)` | 106 | **Associate test on polynomials**: are two coefficient vectors projectively equal? Hand-rolled as all 2×2 minors over 6 coefficients (conics.rs:259-270, O(k²)). General: canonical associate normalization (content-free, positive leading coefficient) then equality — plus, for a *curve* rather than a support, an equal-component test = bivariate gcd. |

### 1.2 `trait SubdivideGeometry` (geometry.rs:142-166)

| Method | Line | Algebraic operation required |
|---|---|---|
| `make_x_monotone(c, out)` | 148 | Split at x-critical points. Today for a conic: real roots of the discriminant (conics.rs:494-495). General: real roots of `Res_y(f, ∂f/∂y)` (critical + singular x), **plus the fiber structure** (how many real branches above each open x-interval, and how branches match across a critical fiber). This is the largest single component that does not exist anywhere in `lazy-exact`. |
| `split(c, at)` | 156 | None (structural). |
| `intersect(c1, c2, out)` | 160 | (a) `Res_y(f, g)` → candidate abscissas; (b) real root isolation with **multiplicity**; (c) per candidate, decide which branch pairs actually coincide — sign of `g` at a point of `f` (conics.rs:582, `sign_at_point`); (d) **crossing parity** (`CrossingKind`, geometry.rs:110-118): today derived from the resultant root's multiplicity plus a gradient-cross-product sign (conics.rs:595-618, `tangency_sign` at conics.rs:631-663), degrading to `CrossingKind::Unknown` when two common points sit over one abscissa; (e) **common-component detection** — a vanishing resultant means a shared component, which today is sidestepped by the `same_support` pre-check (conics.rs:562-566, `sine_radical.rs:1158-1170`). General: bivariate gcd. |

### 1.3 `trait MergeGeometry` (geometry.rs:169-174) and `trait Topology`

`can_merge`/`merge`: the associate test again, nothing new.

`Topology::cmp_ends_at_contracted` (`topology.rs:125-137`) and
`cmp_end_vs_point_at_contracted` (`topology.rs:140-150`) are `unimplemented!()`
for planar topologies and are exercised only over `Segments` today. A general
implementation is **angular order of curve ends at a common algebraic point** —
which is exactly the derivative tower `cmp_y_right_of` was designed to avoid:
compare tangent slopes at an algebraic point, and on ties compare higher-order
contact (a jet / Puiseux-order comparison). Note this as the one place where
the witness trick does not apply, because there is no interval to place a
witness in. **Eventually**, not must-have.

---

## 2. The degree-4 ceiling in the flesh: `conics.rs`

### 2.1 The hand-rolled resultant

`Ellipse::resultant_x` (`conics.rs:276-287`) is the closed-form Sylvester
resultant of two quadratics in `y`:

```
Res = (p₂q₀ − q₂p₀)² − (p₂q₁ − q₂p₁)(p₁q₀ − q₁p₀)
```

with `pᵢ`, `qᵢ` the y-coefficients as polynomials in x. It is 11 lines because
degree 2 in `y` has a closed form. `spherical_circle.rs:589-597` is the *same
eleven lines* for the latitude quadratics. `sine_radical.rs:614-621` is a
different trick — double squaring to eliminate two radicals — producing a
degree-≤8 polynomial that **introduces spurious roots**, which must then be
filtered by an exact on-both-sheets test (`sheets_meet`,
`sine_radical.rs:623-638`), and whose root multiplicities carry no intersection
meaning.

Three copies, one of them lossy, and none of them generalizes past degree 2 in
the eliminated variable. **`Res_y(f, g)` for general bivariate `f, g`, with
correct multiplicity semantics, is the unlock.**

### 2.2 `AlgPoint` and `YRep`

```rust
// conics.rs:51-56
pub struct YRep { a: QPoly, b: QPoly, h: QPoly, d: QPoly }  // y = (a(ξ) + b(ξ)√h(ξ)) / d(ξ)

// conics.rs:71-74
pub struct AlgPoint { x: SharedRoot, y: YRep }              // SharedRoot = Rc<RefCell<RealRoot>>
```

`YRep` is a **normal form for an element of ℚ(ξ)[√h]** — the y-coordinate of a
point on a conic branch, kept symbolic over its own abscissa. Every predicate
reduces to a sign of such an element, via `sign_radical1`/`sign_radical2`
(`roots.rs:622-683`).

**What a general `AlgebraicReal` would have to provide to replace this.** Not
"be a number you can add" — the consumer never adds. It must provide:

- **R-a.** Exact sign of `P(α)` for `P ∈ ℚ[x]`, α an isolated real root
  (`RealRoot::sign_of`, roots.rs:497-524) — 20 call sites in `geoms/`.
- **R-b.** Exact sign of a **radical-tower element** over α:
  `a(α) + b(α)√h₁(α) + c(α)√h₂(α)`, and by extension arbitrary depth. This is
  what `sign_radical1/2` do at depth 1 and 2, by squaring. Depth ≥ 3 is not
  implemented anywhere and would be needed by any family with a
  three-radical carrier. **If resolvent ships only `sign_of(P)` for `P ∈ ℚ[x]`,
  every consumer re-derives these ladders by hand — which is precisely the ~150
  lines/family the code contains today.**
- **R-c.** The alternative to R-b: materialize the y-coordinate as a genuine
  `AlgebraicReal` over ℚ and compare. This is *simpler* for the consumer and
  *much slower*: computing the ℚ-minimal polynomial of an element of
  `ℚ(ξ)(√h)` costs a resultant of degree `deg(ξ) · 2`, per point, per predicate.
  Both must exist; R-b is the fast path and R-c the general fallback. Which one
  is the *default* is a decision (§7, D3).
- **R-d.** A **rational strictly between** two algebraic numbers, or between an
  algebraic number and a set of them (`Conics::witness_right_of`,
  conics.rs:357-380, refines a shared root against a vector of upper limits
  until separated). This is a named API, not an idiom to re-invent.
- **R-e.** Cheap **evaluation at a rational abscissa** producing a degree-2
  radical (`SqrtExt`) rather than an algebraic number
  (`Conics::branch_value_at`, conics.rs:382-395; `SqrtExt::cmp_cross`,
  `sqrt_ext.rs:187-222`, 31 call sites). This is the single hottest fast path
  in the consumer and it must not be routed through the general machinery.
- **R-f.** Isolation results as `(number, multiplicity)` **pairs** — see §7 D5.

---

## 3. Census of `lazy-exact`: what exists, what resolvent duplicates, what must differ

This is the section the deferred integration decision hinges on. Prior art by
the same author, solving adjacent problems, 3,602 LOC.

| File | LOC | What it is | resolvent's relation |
|---|---|---|---|
| `interval.rs` | 431 | f64 interval type, **no global rounding-mode state**; outward widening via `next_up`/`next_down` | **Do not duplicate, do not depend on, do not expose.** resolvent is exact-only; its bounds are `Rational` pairs. See §7 D1. |
| `uncertain.rs` | 120 | `Sign`, `Uncertain<T>`, `UBool`/`UOrd`/`USign` tri-state | resolvent needs its own `Sign`; trivially mapped in an adapter. A tri-state verdict type is needed only for the enclosure API (R14). |
| `exact/rational.rs` | 203 | `Rational` over dashu `RBig`; outward-correct `to_interval` by exact re-comparison (rational.rs:112-126) | resolvent needs its own ℚ. The **bignum choice is R1's lane**; note only that this codebase already settled on `dashu` and that the ecosystem survey (`docs/research/research-rust-ecosystem.md:74,81,135`) rules out `algebraeon` (GPL-3.0) and `algebraics` (LGPL) as dependencies. |
| `exact/mod.rs` | 72 | `RingOps` / `ExactRing` / `ExactField` — a *by-reference ops surface*, explicitly "not an algebraic claim" (mod.rs:11-15) since `Interval` implements it too | resolvent's L0 ring traits will look similar and **must not be the same traits**. Constraint #1: the consumer writes the adapter. |
| `real.rs` | 724 | `Real<E>`: lazy-exact DAG, eager interval + lazy exact, `Arc`+`Mutex`+atomic interval cache, **iterative** eval and drop (real.rs:1-15) | Orthogonal axis — filtering of *arithmetic*, not of *algebra*. resolvent does not duplicate it. **But** resolvent should steal its concurrency protocol for `AlgebraicReal` refinement (§7 D4). |
| `ladder.rs` | 134 | `certify(filter, exact)` (ladder.rs:11-16) + generic `det2`/`det3` | Consumer-side. Not resolvent's. |
| `eft.rs`, `expansion.rs`, `scalar.rs` | 275 | Error-free transforms, expansions, the `Scalar` seam | Not resolvent's. |
| `sqrt_ext.rs` | 322 | `SqrtExt<T> = a + b√r`; exact sign by squaring (sqrt_ext.rs:152-172); **total cross-root comparison** with no separation bounds (sqrt_ext.rs:187-222) | **resolvent must ship an equivalent and must not subsume it.** `circle_segments.rs` (931 LOC) uses `SqrtExt` exclusively and never touches `RealRoot`. Routing degree-2 radicals through defining-poly+interval machinery would be a large, silent regression. |
| `bernstein.rs` | 298 | Exact Bernstein coefficients over `[lo,hi]` + de Casteljau subdivision over ℚ; `sign_over() -> USign` returns **`Unknown`, never a guess** (bernstein.rs:135-152) | resolvent should ship this: certified range enclosure is needed by geometry's fail-closed rung and by FEM positivity checks. Direct duplication; low risk. |
| `roots.rs` | 927 | `QPoly` (dense univariate over ℚ), Euclid gcd, Yun, Descartes/VCA isolation, `RealRoot`, `sign_radical1/2` | **This is resolvent's L1+L2+L3 in miniature, built the way resolvent must not build it.** Detail below. |

### 3.1 `roots.rs` in detail — the duplication and the three differences

What it already does, correctly:

- `QPoly` dense univariate over ℚ, low-to-high, trimmed (roots.rs:43-54); Horner
  eval, derivative, add/sub/mul/scale, `divrem` (roots.rs:143-166).
- Monic Euclid gcd (roots.rs:169-182); square-free part; **Yun's square-free
  decomposition** returning `(multiplicity, factor)` pairs (roots.rs:199-229).
- Descartes sign-variation count on the open interval via the standard
  Möbius chain `(lo,hi) → (0,1) → (0,∞)` (roots.rs:270-288), used only in the
  directions where it is *proof* (roots.rs:14-16).
- Cauchy root bound (roots.rs:291-306); VCA bisection isolation with exact
  rational roots deflated out at midpoints (roots.rs:361-402).
- `RealRoot { poly, lo, hi, multiplicity }` (roots.rs:317-322) — literally the
  `AlgebraicReal` shape from `IDEAS-crates.md` #28 — with `refine`,
  `refine_to_width`, `is_root_of` (gcd + sign change, roots.rs:480-494),
  `sign_of`, `cmp_rational`, `cmp_root`, `to_interval` (roots.rs:600-611).
- `sign_radical1` / `sign_radical2` (roots.rs:622-683).

**Difference 1 — coefficient domain and representation.** Everything is dense
univariate over **ℚ**, with monic normalization in gcd (roots.rs:180-181) that
multiplies denominators at every Euclidean step. resolvent must work over **ℤ**
with primitive-part/content normalization and **modular** gcd, resultant, and
square-free decomposition. This is not an optimization; it is the difference
between a degree-20 resultant being computable and not. It is a **one-way door**
(founding constraint #5) and must be settled before fan-out.

**Difference 2 — univariate only, and the caller eliminates by hand.** There is
no bivariate type, no `Res_y`, no subresultant PRS, no curve analysis. Every
consumer family hand-eliminates (§2.1). resolvent must own elimination.

**Difference 3 — `sign_of` has no separation-bound machinery and no
termination budget.** The module header says so explicitly (roots.rs:8-11):
"no separation-bound machinery is included — comparisons terminate because
distinct algebraic numbers are eventually separated by bisection". That is true
and it is *correct*; it is also unbounded in the worst case, and `sign_of`
(roots.rs:509-523) runs a full `descartes_in` per bisection iteration.
resolvent needs, at minimum, a Mignotte/Davenport separation bound to convert
"terminates eventually" into "terminates in a computable number of steps", and
should prefer sign determination that does not re-run Descartes per step.

**What resolvent must add that has no counterpart here at all:** bivariate and
multivariate polynomials, resultants and subresultant PRS, factorization,
Gröbner, curve analysis / fiber structure, algebraic numbers over ℚ(α),
separation bounds, modular reconstruction.

### 3.2 The `Rc<RefCell<RealRoot>>` tax

`RealRoot::refine` takes `&mut self` (roots.rs:450). Every `Geometry` predicate
takes `&self` and `&Point` (geometry.rs:31-59). The four algebraic families each
independently define:

```rust
pub type SharedRoot = Rc<RefCell<RealRoot>>;
fn shared(r: RealRoot) -> SharedRoot { Rc::new(RefCell::new(r)) }
fn cmp_roots(p: &SharedRoot, q: &SharedRoot) -> Ordering {
    if Rc::ptr_eq(p, q) { return Ordering::Equal; }   // self-deadlock guard
    p.borrow_mut().cmp_root(&mut q.borrow_mut())
}
```

— `conics.rs:32-46`, `sine_waves.rs:31-42`, `sine_radical.rs:71-84`,
`spherical_circle.rs:70-88`. Four copies, including four copies of the
pointer-equality guard that exists to stop a point deadlocking against itself
under `RefCell`. This is a **hard API requirement on resolvent**, not a nit
(R7 below).

---

## 4. What the other geometry families demand

- **`segments.rs`** (307 LOC): rational coordinates only. Needs nothing from
  resolvent. Evidence that resolvent must be *avoidable* — the cheap case must
  not pay.
- **`polylines.rs`** (685 LOC): same.
- **`circle_segments.rs`** (931 LOC): rational circles + lines →
  `SqrtExt<Rational>` coordinates only (`circle_segments.rs:1-9`). **Never
  imports `RealRoot` or `QPoly`.** Supports stay rational forever; only
  endpoints carry roots. This is the load-bearing evidence for R6 (keep a
  first-class degree-2 radical type).
- **`conics.rs`** (722 LOC): §2.
- **`sine_waves.rs`** (591 LOC): Weierstrass-rationalized sinusoids. Points are
  `(branch, isolated t, num/den ∈ ℚ[t])`; predicates are plain
  `RealRoot::sign_of` — *no radicals at all* (`sine_waves.rs:9-13`). Carries a
  point at infinity as a distinct enum variant `TPos::Pi`
  (`sine_waves.rs:95-100`), where `v = c0 − c1` is exactly rational.
- **`sine_radical.rs`** (1,306 LOC): `v = base(u) ± √radicand(u)`, rationalizes
  to exactly the conics `YRep` shape `(a + b√h)/d` with `h` of degree ≤ 4
  (`sine_radical.rs:19-38`). Intersections need the degree-≤8 double-squaring
  elimination (§2.1). Two-sheeted, with sheet junctions where the radicand
  vanishes and `v' = ∞`.
- **`spherical_circle.rs`** (1,408 LOC): the exact structural twin of `conics`
  with `W = tan(u/2)` for `x` and `z = cos v` for `y`
  (`spherical_circle.rs:41-43`). Fails closed with `SphError::Unsupported` on
  two configurations rather than approximating (`spherical_circle.rs:47-60`) —
  the seam and the pole-crossing plane, both detected by **exact algebraic
  conditions**.

**The transcendental question, stated precisely.** The three "non-polynomial"
families are non-polynomial in their *chart coordinates* and fully algebraic in
a *rationalizing parameter the consumer chose*. Nothing in the shipped code
needs resolvent to know what `sin` is. What the consumer does need, and does
itself today, is the bookkeeping around the substitution: branch index, the
image of `u = π` at `t = ±∞`, and the reciprocal transform. Two consequences:

- resolvent should expose the **reciprocal / coefficient-reversal** transform
  `x^n · p(1/x)` as public API — it exists but is private (`QPoly::reverse`,
  roots.rs:242-246). Consumers use it to move ∞ to 0.
- resolvent should **not** own the branch/∞ enum. That is chart bookkeeping and
  it belongs to whoever owns the chart. Keeping it out is also what keeps the
  deferred-integration adapter thin.

§5 covers where transcendentals genuinely appear.

---

## 5. Pressure from the outward consumers

Read as pressure on API shape, not as commitments.

### 5.1 FEM form compiler (#34, `IDEAS-crates.md:588-651`)

- **Symbolic differentiation of weak forms** and "Newton with symbolically
  derived Jacobian" (`IDEAS-crates.md:646-651`). Needs L4: a hash-consed
  expression DAG with differentiation. **Not** a decision procedure.
- **Method of manufactured solutions, fully automated** (`IDEAS-crates.md:634-641`):
  pick `u_exact` symbolically, apply the differential operator symbolically, get
  the forcing term `f` exactly. MMS solutions are conventionally
  `sin(πx)sin(πy)`, `exp(x)`, etc.

  **This is the first genuinely transcendental requirement on resolvent, and it
  arrives from a non-geometry consumer — but it needs differentiation and
  code-emission only, never zero-testing.** Differentiating `sin`/`exp`/`log` is
  a rewrite rule. Deciding whether a transcendental expression is zero is
  Richardson/Schanuel territory and is undecidable in general.

  **This resolves the scope question cleanly and it should be the recommended
  decision: L4 may carry transcendental function symbols with differentiation
  rules; L0–L3 never see them, and no zero-test is offered for them.**

- **Exact symbolic integration of polynomials over reference simplices**
  (`IDEAS-crates.md:645`: "Element tensor vs symbolically-integrated exact
  tensor"). Cheap, exact ℚ, must-have for #34's M0.
- **CSE, sum factorization, contraction reordering** (`IDEAS-crates.md:620-623`)
  and egg-compatible e-graph simplification. Forces L4 to be a real DAG with
  stable node identity, not a tree, and forces polynomial → straight-line-code
  lowering to be a supported output.
- Build-time compilation via `build.rs`/proc-macro (`IDEAS-crates.md:610-613`)
  means **L4 and the codegen path must be usable from a build script** —
  compile-time cost matters, runtime cost mostly does not. Different performance
  regime from geometry entirely.

### 5.2 SMT NRA theory (#12, `IDEAS-crates.md:168-229`, M5 "NRA via MCSAT")

The heaviest and least compatible demands:

- **CAD projection** needs the full **subresultant PRS** and **principal
  subresultant coefficients**, plus discriminants and leading coefficients — not
  just the top resultant. Geometry needs `Res_y`; CAD needs the whole chain.
- **Algebraic sample points at multiple levels**: isolate the real roots of a
  univariate polynomial whose **coefficients live in ℚ(α₁,…,α_k)**, not ℚ. This
  is materially stronger than anything geometry needs and it is the requirement
  that decides the `AlgebraicReal` type parameterization (§7 D2).
- **Factorization is not optional** for NLSAT: projection sets blow up without
  it.
- **Proof production from day one** (`IDEAS-crates.md:196-199`, called out as
  non-retrofittable). For resolvent this means operations should optionally
  return **cofactors / certificates**: `gcd` returning Bézout coefficients,
  resultant returning the `Res = uf + vg` combination, Gröbner storing
  `f = Σ hᵢgᵢ`. This aligns exactly with the founding self-certification thesis
  — the *same* data that grades resolvent internally is the data #12 needs to
  emit externally. Design cofactor return in from the start; it is cheap when
  planned and a rewrite when not.

### 5.3 Exact medial axis (#27, `IDEAS-crates.md:448-478`)

- Medial axis of a polyhedron: pieces of planes and quadrics, sheets meeting
  along algebraic curves in 3-space. Needs **trivariate** elimination — Gröbner
  or resultant-based, with **ideal saturation** to strip degenerate components.
- "Maximal ball property, checked exactly" (`IDEAS-crates.md:470-472`) is a
  quantified statement over the boundary; the practical reduction is per-face
  critical-point systems → **0-dimensional multivariate solving with exact real
  solutions** (RUR, or Gröbner + eigenvalue, or triangular decomposition).
- This is the consumer that actually requires L2's Gröbner. Geometry does not.

### 5.4 Geometric constraint solver (#24, `IDEAS-crates.md:337-383`)

Mentions "exact predicate evaluation (your kernel)" — that is `lazy-exact`, not
resolvent. The only resolvent-shaped demand is branch tracking under continuous
drag (`IDEAS-crates.md:363-366`), which would want algebraic-number continuation.
Low pressure; listed for completeness.

---

## 6. Prioritized requirements

**Tier A = must-have for the first useful release.** Definition of "useful":
`arrangements` can implement a `Geometry` for curves `f(x,y) = 0` of arbitrary
degree over ℚ, replacing `conics.rs`, without hand-rolling elimination — behind
an adapter *it* writes.

Verdict column: **cert** = self-certifying or property-checkable, converges fast
under agent fan-out. **num** = success is a number to optimize; converges slowly;
must be sequenced with that in mind (founding constraint #3).

### Tier A — must-have for the first useful release

| # | Operation | Who needs it | Exact about | Verdict | Milestone |
|---|---|---|---|---|---|
| **R1** | `UPoly<R>`: dense univariate over a ring, with `R = ℤ, ℚ, GF(p)`. Content/primitive part, canonical associate normalization, Horner eval, derivative, `divrem`, reciprocal transform (public — cf. private `roots.rs:242`) | all | exact ring arithmetic; canonical form is *canonical* (associate test must be an `==`) | cert | M0 |
| **R2** | Univariate gcd and square-free decomposition, **modular** (mod-p + CRT/rational reconstruction + verify) | arrangements (`eq_curves`, `is_root_of`, `cmp_root`), SMT | divides both ways; Yun factors pairwise coprime and reconstruct the input | cert (correctness) / **num** (coefficient growth) | M1 |
| **R3** | Real root isolation over ℤ/ℚ: Descartes/VCA with integer Taylor shifts, returning `(AlgebraicReal, multiplicity)` pairs | arrangements (`make_x_monotone`, `intersect`), SMT | count is exact; each interval provably contains exactly one root; Σ multiplicities = degree of the square-free-corrected input | cert | M1 |
| **R4** | `AlgebraicReal`: `{ squarefree defining poly over ℤ, isolating (lo, hi) ∈ ℚ² }` with `refine`, total `cmp` (equality via gcd + sign-change certificate), `cmp_rational`, exact `sign_of(P)` for `P ∈ ℚ[x]`, outward-correct `(f64, f64)` enclosure | arrangements (**everything**), SMT, medial axis | trichotomy and transitivity as property tests; equality decided algebraically, never by interval width | cert (see note) | M1 |
| **R5** | **Separation bound** (Mignotte/Davenport class) so comparison and `sign_of` have a computable step budget rather than "terminates eventually" (cf. `roots.rs:8-11`) | arrangements, SMT | the bound is *valid* (never terminates early with a wrong verdict) | cert | M1 |
| **R6** | `SqrtExt`-equivalent: `a + b√r` over a field, exact sign by squaring, **total cross-root comparison** (`sqrt_ext.rs:187-222`), no separation bounds | arrangements — `circle_segments.rs` (931 LOC) uses *only* this; also the `cmp_y_right_of` witness fast path (conics.rs:382-395) | sign and order exact; cross-root comparison total | cert | M1 |
| **R7** | `AlgebraicReal` is `Clone` + **refines through `&self`** + `Send + Sync`, with cheap shared refinement progress and a self-comparison guard | arrangements — removes 4× duplicated `Rc<RefCell<RealRoot>>` boilerplate (§3.2) | refinement is monotone; a torn read is still a valid enclosure (cf. `real.rs:24-58`) | cert | M1 |
| **R8** | Radical-tower sign: exact sign of `Σ cᵢ(α)·√hᵢ(α)` at arbitrary depth, generalizing `sign_radical1/2` (`roots.rs:622-683`) | arrangements — the reason each family carries ~150 lines of by-hand ladder | sign exact; no tolerance anywhere | cert | M1 |
| **R9** | `rational_between(&AlgebraicReal, &[AlgebraicReal]) -> Rational` — a certified rational witness strictly between one algebraic number and a set of larger ones (cf. `Conics::witness_right_of`, conics.rs:357-380) | arrangements (`cmp_y_right_of` — the method that avoids derivative towers) | the returned rational is *strictly* between; termination guaranteed by distinctness | cert | M1 |
| **R10** | `MPoly<R, Order>`: sparse distributed multivariate, **packed exponent vectors**, monomial comparison as one integer compare | Gröbner, elimination, FEM | representation only; but see founding constraint #5 — one-way door | n/a (structural) | M0 |
| **R11** | **Resultant `Res_y(f, g)` for general bivariate `f, g`**, with correct multiplicity semantics and no spurious roots (contrast `sine_radical.rs:614-621`) | arrangements (`intersect`, `make_x_monotone`) — **the unlock** | `Res = 0` ⇔ common root (checkable via R3); multiplicity of a resultant root = intersection multiplicity in the generic case | cert (correctness) / **num** (degree-20 coefficient blowup) | M2 |
| **R12** | Bivariate gcd / common-component detection (a vanishing resultant means a shared component; today sidestepped by `same_support`, conics.rs:562-566) | arrangements (`intersect`, `eq_curves`) | divides both ways | cert | M2 |
| **R13** | **Curve analysis**: real roots of `Res_y(f, ∂f/∂y)`; number of real branches above each open x-interval; branch matching across critical fibers; branch value ordering at an algebraic abscissa. Returned as **purely algebraic data**, no geometric types in the signature | arrangements (`make_x_monotone`, `cmp_y_at_x` at arbitrary degree); medial axis | branch counts satisfy Bézout-style bounds; fiber counts consistent across adjacent intervals | cert (Euler/Bézout checks) / **num** | M2 |
| **R14** | Certified range enclosure over a rational interval (Bernstein + de Casteljau, `bernstein.rs`), verdict `Certain(Sign) | Unknown`, **never a guess** | arrangements' fail-closed rung; FEM positivity | endpoint coefficients equal endpoint values exactly; hull contains the true range | cert | M1 |
| **R15** | Results of R3/R11/R13 are **cheaply shareable** — `Arc`-backed, or an explicit analysis handle whose construction is the expensive part. Evidence: `conics.rs:459-460` and `:565-567` recompute the resultant *and* re-isolate on every predicate call | arrangements | n/a | **num** | M2 |

*Note on R4's verdict:* correctness is `cert` (trichotomy/transitivity property
tests are exactly the canary the founding doc names), but the **useful-release
gate is a number**: whether a degree-20 resultant with large coefficients can be
isolated and compared in bounded time. Do not schedule R4 as if a green property
suite means done.

### Tier B — eventually

| # | Operation | Who | Milestone |
|---|---|---|---|
| **R16** | Univariate factorization over ℤ: Zassenhaus + van Hoeij lattice recombination | arrangements (intersection multiplicity beyond the multiplicity-parity heuristic, conics.rs:600-618); SMT (projection-set control) | M3 |
| **R17** | Arithmetic in ℚ(α) — genuine field ops on algebraic numbers, and `AlgebraicReal` with coefficients in ℚ(α₁..α_k) | **SMT only** (multi-level sample points). Arrangements never adds algebraic numbers (finding #2) | M3 |
| **R18** | Subresultant PRS + principal subresultant coefficients as a first-class chain | SMT (CAD projection); arrangements would use `sres` for better multiplicity data | M3 |
| **R19** | L4 hash-consed expression DAG: differentiation, transcendental function *symbols* (no zero-test), CSE, egg-compatible, straight-line-code lowering | FEM #34 | M4 (**parallel lane** — near-independent of M1–M3) |
| **R20** | Exact symbolic integration of polynomials over reference simplices/hypercubes | FEM #34 M0 | M4 |
| **R21** | F4 Gröbner over GF(p) with modular reconstruction; ideal saturation | medial axis #27; **not** arrangements | M5 |
| **R22** | 0-dimensional multivariate real solving (RUR / triangular decomposition) | medial axis #27 | M5 |
| **R23** | Cofactor / certificate return on gcd, resultant, Gröbner (`f = Σ hᵢgᵢ`) | SMT proof production (non-retrofittable, `IDEAS-crates.md:196-199`); also the internal self-certification story | design from M1, ship by M3 |
| **R24** | Full CAD: projection operator, cell construction, single-cell explanations | SMT #12 M5 | M6 |
| **R25** | Tangent/jet comparison at an algebraic point (angular order at a common point, `topology.rs:125-150`) | arrangements' contracted-vertex topologies at arbitrary degree | M3 |
| **R26** | Algebraic-number continuation / branch tracking | constraint solver #24 | unscheduled |

### Explicitly out of scope (recommend refusing)

- **Zero-testing of transcendental expressions.** Undecidable in general. No
  consumer needs it: FEM needs differentiation and evaluation only (§5.1);
  geometry rationalizes before it asks (§4).
- **Any API taking a tolerance parameter.** `arrangements` refuses to silently
  perturb or merge (its DESIGN.md lists snap rounding and automatic tolerance
  modes as *permanently* out) and its exact families declare
  `type Error = Infallible` precisely because no configuration escapes the
  ladder (`sine_radical.rs:53-63`). A tolerance argument anywhere in resolvent
  would be unusable by its first consumer.
- **A `simplify()` that tries to be clever** — the founding doc's own named risk.

---

## 7. Decisions this research forces, and where consumers want incompatible shapes

**D1 — resolvent must not expose a float interval type.**
`lazy-exact` has a carefully-built `Interval` (431 LOC, no global rounding-mode
state). If resolvent exposes its own, the adapter has to convert at every
boundary and the two enclosure semantics can silently disagree. Expose bounds as
`(Rational, Rational)` plus an outward-correct `(f64, f64)` convenience, and let
the consumer build its own interval. This is the cheapest single thing that
keeps the deferred integration decision cheap.

**D2 — `AlgebraicReal`'s coefficient domain: ℤ-only vs tower-generic.**
*Incompatible shapes, and this is the sharp one.* Arrangements only ever needs
α defined over ℚ (finding #2, confirmed: `RealRoot` has no arithmetic and four
families ship). SMT NRA *requires* isolating roots of polynomials with
coefficients in ℚ(α₁,…,α_k). Options: (i) `AlgebraicReal { poly: UPoly<ℤ>, iv }`
— simple and fast, but representing an element of ℚ(α,β) over ℚ costs a
resultant of degree `deg α · deg β`; (ii) tower-generic — general but 5–10×
slower on the geometry path, which is 100% of the near-term usage.
**Recommendation:** ship (i), but make `UPoly<R>` generic over the coefficient
ring from day zero so that `UPoly<NumberField>` is an *added instantiation*, not
a rewrite. Settle before fan-out; this is a founding-constraint-#5 one-way door.

**D3 — radical-tower ladders (R8) vs materialized algebraic numbers (R-c).**
The consumer's y-coordinates are elements of `ℚ(ξ)[√h]` kept in normal form
`(a + b√h)/d` and only ever signed. Shipping only `sign_of(P), P ∈ ℚ[x]` forces
every consumer to re-derive squaring ladders by hand (the ~150 lines/family that
exist today). Shipping only materialized algebraic numbers is correct but pays a
resultant per point per predicate. **Recommendation:** ship both, make R8 the
documented fast path and materialization the general fallback, and say so in the
API docs so consumers do not silently pick the slow one.

**D4 — refinement mutability and `Send`/`Sync`.**
`&mut self` refinement forces `Rc<RefCell<_>>` on the consumer, four times over
(§3.2), including four copies of a self-deadlock guard. `lazy-exact::Real`
already solved the same problem the other way — `Arc` + per-node lock + a
*monotone* atomic interval cache where even a torn read is a valid enclosure
(`real.rs:1-15, 24-58`). **Recommendation:** `AlgebraicReal = Arc<Inner>`,
`&self` methods, `Send + Sync`, monotone refinement. Cost: `std` (or a
portable-atomic shim); `no_std` becomes feature-gated and probably loses the
shared-refinement ergonomics.

**D5 — multiplicity is not a property of a number.**
`RealRoot` stores `multiplicity: u32` (`roots.rs:321`) and `conics.rs:569` reads
it off a *resultant* root to infer crossing parity. But √2 has no multiplicity;
the field is a property of the isolation call. **Recommendation:**
`isolate_roots` returns `Vec<(AlgebraicReal, u32)>`. Small, but it is an API
shape and it is wrong in the prior art.

**D6 — two polynomial types, one algebra.**
`MPoly` with packed exponents is right for Gröbner and wrong for the geometry
path: a dense degree-8 univariate stored as `Vec<(packed_exp, coeff)>` costs
~2× memory and loses the O(1) indexed access that Horner, Taylor shift, and
Descartes all want — and 100% of arrangements' usage is dense univariate of
degree ≤ 8 today, ≤ ~40 at arbitrary degree. **Recommendation:** ship `UPoly<R>`
and `MPoly<R, Order>` as distinct types with explicit conversions and a
`Res_y: MPoly × MPoly → UPoly` bridge. Do not unify.

**D7 — who owns curve analysis (R13).**
Arrangements needs it; medial axis needs a 3D analogue; it is the largest
component with no counterpart in the prior art. If resolvent ships it, resolvent
starts owning geometry-shaped concepts, which makes founding constraint #1's
"thin adapter the consumer writes" harder. If resolvent does not, every consumer
rebuilds the hardest thing. **Recommendation:** ship it, but with a signature
that mentions no geometric type — inputs are `MPoly`, outputs are critical
abscissas, per-interval branch counts, and branch-index→root maps. Then it is
algebra that happens to be useful to geometry.

**D8 — transcendentals live in L4 only.**
FEM needs `sin`/`exp` symbols with differentiation rules and no zero-test;
geometry needs a zero-test and no transcendentals (§4, §5.1). These are
separable and should be separated by layer, permanently. **Recommendation:** L4
may carry opaque transcendental function symbols with differentiation and
evaluation; L0–L3 never see them; resolvent offers no transcendental zero-test
at any layer.

**D9 — sequencing: `cert` lanes and `num` lanes must not be scheduled alike.**
R1–R9, R12, R14 are certificate lanes and can be fanned out wide. R2, R11, R13,
R15 have a correctness certificate *and* a performance gate, and the performance
gate is the one that decides whether the release is useful. Sequence them so the
modular-methods substrate (R2) lands before the things that stress it (R11,
R13), because retrofitting modular arithmetic under a working ℚ implementation
is a rewrite, not an optimization.

---

## 8. Open questions — what would settle each

1. **What degree and coefficient size does arbitrary-degree arrangements
   actually hit?** Everything in §6's `num` column hinges on this and I do not
   know it. *Settled by:* generating a corpus of curve pairs of degree 3–8 with
   rational coefficients of realistic bit-size, computing `Res_y` with the
   existing `QPoly` (which will be slow but correct), and recording the degree
   and coefficient bit-length of the resultants and the wall time of
   `isolate_roots` + a `sign_of` sweep. That gives the actual target, not a
   guess. Cheap to run against the existing crate.

2. **Does `AlgebraicReal`-materialized comparison (R-c) actually cost enough to
   justify shipping R8's ladders?** *Settled by:* a microbenchmark comparing
   `sign_radical2` at a degree-4 ξ against materializing the same value's
   ℚ-minimal polynomial (degree 8) and signing it.

3. **Is `dashu` the right bignum, and is it permissive?** Deliberately not
   answered here — it is lane R1's. I note only that the same author already
   chose it (`exact/rational.rs:7-8`) and that the ecosystem survey rules out
   the GPL/LGPL alternatives as dependencies
   (`docs/research/research-rust-ecosystem.md:74, 135`).

4. **How much does modular gcd actually buy at these sizes?** Modular methods
   are "the structural decision" per the founding doc, but at degree 8 with
   64-bit coefficients a well-implemented ℚ subresultant PRS may win. *Settled
   by:* the same corpus as (1), with both implementations. This matters because
   if the crossover is above the real workload, resolvent is paying complexity
   for nothing at the point where its first consumer unblocks — while still
   needing it for #12 and #27.

5. **Should intersection multiplicity come from resultant-root multiplicity
   (today's heuristic, degrading to `CrossingKind::Unknown`, conics.rs:600-618)
   or from factoring the resultant (#28's stated pipeline)?** *Settled by:*
   constructing the ambiguous case the code already documents — two distinct
   common points over one abscissa with parallel gradients — and checking
   whether factorization actually resolves it or whether local intersection
   multiplicity via subresultants is required.

6. **Does the `CrossingKind::Unknown` escape hatch survive at arbitrary
   degree?** The consumer's contract allows it ("consumers re-derive locally",
   geometry.rs:116-117). If arbitrary-degree tangency makes `Unknown` the common
   case rather than the rare one, resolvent's multiplicity story becomes
   must-have rather than Tier B. *Settled by:* the corpus in (1), counting the
   `Unknown` rate.

7. **Is a `no_std` core worth preserving?** D4's recommendation costs it.
   *Settled by:* asking whether any prospective consumer is embedded. None of
   #12, #24, #27, #28, #34 obviously is.

8. **Where does the fail-closed "certified enclosure" verdict (R14) belong in
   the type system?** Arrangements' exact families declare
   `type Error = Infallible` and treat any `Unknown` as a design failure;
   `spherical_circle` instead fails closed with `SphError::Unsupported`. If
   resolvent returns `Uncertain<Sign>` from R14 but `Sign` from R4/R8, consumers
   get two verdict vocabularies. *Settled by:* a decision doc, not an
   experiment — but it should be made before the API is written, not after.
