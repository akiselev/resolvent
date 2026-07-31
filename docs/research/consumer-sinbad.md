# E1 — sinbad as a resolvent consumer

**Status:** consumer evaluation, research input for the resolvent founding plan. Not a
commitment.
**Subject:** `/home/dev/sinbad` @ `d5726c8`, 61,419 lines of Rust across 42 crates
(`tokei crates`), of which 33 are workspace members (`/home/dev/sinbad/Cargo.toml:5-45`).
**Method:** read shipped source first, plans second. Every claim below cites a file and
line in sinbad. Where I cite a plan I say so, and I discount it.
**Verdict:** **would-benefit.** Real, evidenced demands exist. None of them is blocking a
shipped code path today, and every one of them lands in resolvent's L4 — the layer
`IDEAS-crates.md:114-115` itself calls "a thin layer on top, not the point." Sinbad does
not use L1 multivariate polynomials, L2 Gröbner/factorization, or L3 algebraic numbers
**at all**. Treat sinbad as a *shape constraint* on the expression layer and a
*determinism constraint* on the whole library — not as a demand driver for the engine.

---

## 0. The five findings that matter

1. **The highest-value lead is real but smaller than advertised.** MMS forcing terms are
   hand-authored closures (`crates/sinbad-testkit/src/mms.rs:124-207`) and the module
   docs name the replacement explicitly: "have **SymPy, offline, codegen `u*` and its
   forcing `f` as committed, content-addressed Rust closures**" (`mms.rs:31-36`).
   `SINBAD-ACCELERATION-SWEEP.md:177` is blunter and better-scoped: "A tiny in-Rust
   polynomial/trig symbolic-diff covers the common cases with no Python at CI time."
   That is the demand. It is symbolic differentiation of elementary functions over a
   small DAG, plus a code emitter. It is not a computer algebra system.

2. **Every shipped manufactured solution is transcendental, not polynomial.**
   `harmonic_trig` is `sin(πx)·sinh(πy)` (`mms.rs:158-168`), `harmonic_exponential` is
   `exp(x)·cos(y)` (`mms.rs:172-181`), `poisson_sine` is `sin(πx)·sin(πy)` with forcing
   `−2π²·sin(πx)·sin(πy)` (`mms.rs:186-196`). The two polynomial members exist only as
   degenerate exactness checks and are P1-exact, so their order slope is unobservable
   (`crates/sinbad-testkit/tests/residua_operator_demo.rs:318-321`). **If resolvent's L4
   admits only ring operations over L0 elements, it cannot serve sinbad's single
   strongest use case.** This is the sharpest API-shape finding in this document (§5.1).

3. **MMS's actual blocker is a numerical assembly seam, not a symbolic one.** `residua`'s
   volumetric source is `SourceField { per_region: BTreeMap<RegionTag, f64> }`
   (`crates/residua/src/lib.rs:358-361`) — piecewise constant per mesh region — and
   `assemble_source_load` adds `s·area/3` per vertex (`lib.rs:923-928`). A spatially
   varying `f(x,y)` cannot be assembled. So `poisson_sine`, the one member with non-zero
   forcing, **cannot be run against residua today** regardless of how `f` is derived; the
   residua demo only runs the harmonic members through the Dirichlet trace
   (`residua_operator_demo.rs:322`). Sinbad's own corpus plan says so:
   "The missing forcing seams in impedant/residua are themselves counted refusals until
   built" (`plans/scored-corpus-loop.md:252-253`). Shipping resolvent would not unblock
   MMS. Shipping a `SourceField::Function` seam would.

4. **plexus is genuinely blocked, in code, on symbolic `d/dt` — and the blocker is ~300
   lines.** `crates/plexus/src/index_reduction.rs:3-6`: "**Not implemented in this
   slice.** These passes need a *symbolic differentiation* (`d/dt`) layer that does not
   yet exist in the federation." The whole module is a `NoStructuralPass` identity
   (`index_reduction.rs:52-61`). The plan hardened to "plexus needs a small CAS"
   (`plans/dae-equation-compiler-architecture.md:283`, restated at `:620`) after
   discovering that anvil's straight-line tape gives numeric `∂/∂x`, not symbolic `d/dt`
   with variable growth (`:279-282`). That diagnosis is correct. But the same plan puts
   the numerical-stabilization path first and calls symbolic reduction the "north-star
   layer that makes it acausal and general, **not a prerequisite for the first transient
   result**" (`:578-582`).

5. **Sinbad already owns the exact-arithmetic slice it needs, and deliberately bounds
   it.** The only exact arithmetic in the workspace is `lazy_exact::Rational` determinant
   signs in `crates/meshwright/src/predicates.rs:33-70`. Circumcenters are computed
   exactly then **snapped back to 53 bits** because "nested circumcenters over
   full-mantissa `from_f64` coordinates … blow up the bit-length and make exact
   predicates progressively, unboundedly slower" (`crates/meshwright/src/triangulate.rs:
   500-512`, code at `:514-535`). Sinbad's posture toward unbounded exact arithmetic is
   *defensive*. Any resolvent `Q` that does not expose bit-length and an explicit
   rounding operation is unusable here.

---

## 1. What sinbad actually needs, by crate, with evidence

### 1.1 `sinbad-testkit` — MMS forcing generation (L4, build-time)

Today: five hand-authored `ManufacturedSolution`s, each a pair of boxed closures
`u: Fn(f64,f64)->f64` and `laplacian: Fn(f64,f64)->f64` (`mms.rs:69-70`), with the
Laplacian derived by a human and asserted in a doc comment ("`∇²u* = −π²·sin·sinh +
π²·sin·sinh = 0`", `mms.rs:157`). The harness itself is pure numerics: `error_l2`
(`mms.rs:232-258`) and `observed_order_loglog`, a least-squares slope of `ln‖e‖` vs `ln h`
(`mms.rs:330-349`). Neither touches algebra.

What a CAS replaces: build a DAG for `u*`, differentiate twice per spatial variable, and
emit the closure body. For `𝓛[u] = −∇·(κ∇u)` with a *smooth* `κ` the forcing is
`−∇κ·∇u − κ∇²u`, three more derivatives. Note that residua's `κ` is not smooth today —
`ScalarCoefficient { per_region: BTreeMap<RegionTag, f64>, background: f64 }` is
"piecewise-constant" by construction (`crates/residua/src/lib.rs:289-293`), so `∇κ = 0`
inside every element and the second term is all there is. A manufactured solution with a
spatially varying coefficient needs the same missing seam as finding #3. For fervor's
parabolic operator add `∂u/∂t`. That is the whole job.

Operations needed: DAG construction with named symbols; `diff(expr, sym)`; constant
folding; a deterministic topological traversal so the consumer can print Rust. Latency
class: **build-time**. Bignum coefficients are free here — the whole run is one xtask
invocation producing a committed file.

What is *not* needed: no simplification beyond folding (`sin(x)²+cos(x)²→1` never comes
up); no factoring; no root finding; no ideal theory.

### 1.2 `plexus` — Pantelides / dummy-derivative index reduction (L4, build-time)

Today `EquationSystem` is pure boolean incidence: "No numeric coefficients, no symbolic
expression — only the sparsity pattern of the Jacobian"
(`crates/plexus/src/system.rs:4-6`). The pipeline (Hopcroft–Karp matching → Tarjan SCC →
BLT → Cellier tearing, `crates/plexus/src/lib.rs:8-16`) is entirely graph-theoretic and
correct without any algebra. Pantelides is *also* structural — the graph tells you *which*
equations to differentiate. What is missing is the act of differentiating them.

Operations needed: an expression DAG with a `der(x, n)` variable convention; `d/dt` under
the chain rule where `d/dt(der(x,n)) = der(x,n+1)`; introduction of fresh variables;
structural equality for alias detection (`a = b`, `a = −b`); a canonical form so that the
resulting `Schedule` is content-addressable — the plan makes determinism a hard
requirement: "dummy-derivative selection must be *deterministic* for the `Schedule` to
content-address" (`plans/dae-equation-compiler-architecture.md:596-597`, commitment #11).

Latency class: **build-time**. Index reduction runs once per model compile, memoized
through `reprise`/`rutter`.

Urgency: the module is a stub blocked on exactly this. But per `:578-582` the crate is not
on the critical path, and per `:585-587` the plan's own worry is variable-count blowup, a
structural problem a CAS does not solve.

### 1.3 `solverang` event detection — univariate real root isolation (L2, per-operation)

`crates/numeric-contracts/src/dae.rs:364` specifies the event loop as "dense-output
crossing eval → **Sturm-sequence root isolation** → event iteration to a discrete fixed
point → consistent reinit → restart", consumed by `EventBearing::indicators`
(`dae.rs:366-370`). The dense output of a BDF / generalized-α step is a polynomial in `t`
of degree ≤ ~5 with f64 coefficients over `[t_n, t_n+h]`. Lifting those coefficients
losslessly to ℚ and running a certified isolation gives a *proven* crossing count per
step — a `Grade::Proven` rung in tiered-core's lattice (`crates/tiered-core/src/grade.rs:
15-18`) where a float Sturm sequence can only justify `Estimated`.

This is the one place sinbad would call resolvent's **engine**, not its expression layer.

Caveat, stated plainly: `crates/solverang/` in this repo contains `DESIGN.md` and
`STATUS.md` and no source. The integrator is not written. This demand is real in shape and
unimplemented in fact.

Latency class: **per-operation**, thousands of calls per transient solve. Coefficients are
exact lifts of f64, so numerators and denominators start at ~53 bits and a subresultant PRS
on degree 5 grows them fast. This is the demand that constrains resolvent's performance
envelope, and the only one that does.

### 1.4 `meshwright` — exact rational predicates (L0, inner-loop)

Already served by `lazy-exact`. `orient2d` and `in_circle` are exact rational determinants
(`crates/meshwright/src/predicates.rs:33-70`), the only exact arithmetic in the workspace.
Resolvent could replace `lazy_exact::Rational` here, but that is a *substitution*, not a
demand — and it collides with the deferred arrangements/resolvent unification decision.
Note what the code needs and resolvent must therefore have: `Rational::from_f64 -> Option`
(fails closed on non-finite, `predicates.rs:93-96`), `to_f64_lossy`
(`triangulate.rs:529-530`), `sign()` returning a three-valued enum (`predicates.rs:39`),
and `add/sub/mul/div` by reference without consuming.

Latency class: **inner-loop**. Every Bowyer–Watson cavity walk. `triangulate.rs:500-512`
documents that unbounded bit growth here is a real, observed performance failure.

### 1.5 Everything else — no demand found

- **`sinbad-study` / `sinbad-loom` (adjoint, optimization).** The discrete adjoint is
  linear algebra, not symbolic: `Kᵀλ = ∂g/∂u`, `∂J/∂pᵢ = ∂g/∂pᵢ − λ·∂r/∂pᵢ`
  (`crates/sinbad-study/src/adjoint.rs:9-17`), and `∂K/∂pᵢ = Aᵢ` falls out of `K` being
  linear in the per-region coefficients (`:16-17`). Gradients not covered by the adjoint
  are central finite differences (`crates/sinbad-study/src/objective.rs:146-166`) or the
  complex step (`crates/sinbad-testkit/src/adjoint.rs:270-275`), which is
  machine-precision-exact for any complex-analytic implementation with no symbolic work at
  all. Sinbad has a *working substitute* for symbolic differentiation and gates the adjoint
  against it (`adjoint.rs:277-282`).
- **`ballast` (rational macromodels).** Pole-residue forms over `Complex<f64>`, fitted to
  sampled data by Gustavsen vector fitting with an injected dense eigensolver
  (`crates/ballast/src/lib.rs:17-21, 33-40`). The inputs are measurements; exactness is
  meaningless. "Rational function" here is not the algebraic object.
- **`caliper`.** Newton on a truncated Fourier series with a hand-differentiated
  derivative (`crates/caliper/src/calibrate.rs:143-162`). Twenty lines. A CAS is overkill.
- **`league` (units).** Rational SI exponents as `{num: i16, den: i16}` with `const fn`
  gcd reduction (`crates/league/src/exp.rs:14-17, 20-33`) and a frozen wire form
  (`:11-13`). Fixed-width, `Copy`, `Hash`, `const`. Resolvent's `Q` is the wrong type here
  and must not be offered as one.
- **`impedant`, `fervor`, `sinbad-testkit::oracle`.** Hand-coded transcendental closed
  forms in f64 — Hammerstad/Wheeler microstrip (`crates/sinbad-testkit/src/oracle.rs:1-2`),
  Dowell AC resistance (`crates/impedant/src/acloss.rs:1-13`). These are *evaluated*, never
  manipulated.
- **`reckon`.** Bit-reproducible summation via binned superaccumulator and Shewchuk
  expansions (`crates/reckon/src/lib.rs:14-24`). This is exact f64 arithmetic and sinbad
  owns it. Resolvent must not compete here.

---

## 2. anvil: honest assessment of the L4 overlap

Anvil's header defers "Reverse-mode automatic differentiation" and "E-graph rewriting /
equality saturation" (`crates/anvil/src/lib.rs:19-20`). Both look like resolvent L4.
Skeptically: **they are not, and resolvent should not chase them.**

**Anvil's IR is not a DAG.** `CompiledConstraints` is a flat `Vec<ConstraintOp>` over
virtual registers `Reg(u16)` (`crates/anvil/src/opcodes.rs:14, 40-266`), emitted by a
fluent `OpcodeEmitter` that allocates a fresh register per operation
(`crates/anvil/src/lower.rs:47-52, 92-107`). There is no sharing, no interning, no
structural identity — `uses_register` / `defines_register` are linear scans
(`opcodes.rs:272-330`). Converting to a hash-consed DAG and back is a real translation,
not an adoption.

**The coefficient type is f64 and cannot be anything else.** `LoadConst { value: f64 }`
(`opcodes.rs:53-58`) feeds a Cranelift backend that emits machine instructions
(`crates/anvil/src/cranelift.rs`, 1,623 lines). Resolvent's L4 leaf is an exact L0 element.

**"Equal" means different things.** Anvil wants what Herbie does: rewrite to the *most
numerically accurate* form under an FP cost model — `SINBAD-GRAPH-COMPILER.md:85-90` names
`egg`/`egglog` for exactly this, "**and** (via a numerical cost model) the most *accurate*
form, not just the fastest." Those rewrites change the computed f64 value; they are
justified by error analysis, not by algebra. Resolvent's rewrites must not change the
value. Sharing an e-graph across those two cost models is a research project, not an API.

**Sinbad already ruled against the open symbolic form language.**
`plans/operator-primitives-plan.md:470` records the divergence from UFL/FFC as deliberate:
"**closed-world** (an enum of shapes an agent extends by a match arm), not an open symbolic
DSL — determinism + provenance over expressiveness." An operator is a serializable
descriptor; the enum arm is the extension point. A symbolic weak-form layer is not wanted.

**Conclusion.** anvil should call `egg` directly. Resolvent should be `egg`-*compatible* in
the sense of exposing a stable structural encoding of its DAG, and should stop there. The
only thing anvil would ever want from resolvent is *reverse-mode AD over a DAG*, and that
is 200 lines anvil can write against its own tape — which is what the milestone list
already assumes.

The `#34` "symbolically-derived Newton Jacobians as a headline feature" framing does not
find a consumer here either: **every operator in the workspace is linear.**
`OperatorProperties { linear: true }` for M1 (`crates/residua/src/lib.rs:1087-1091`), the
capability traits are documented as "For M1 (linear): `r(u) = K u − f`" and "For M1,
`J = K`" (`lib.rs:1102, 1114`), and grep for nonlinear physics returns integrator plumbing
only. There is no Jacobian to derive.

---

## 3. The determinism and fail-closed contract resolvent must honor

Non-negotiable if resolvent is to be callable from inside sinbad at all. These are
extracted from shipped code and the ratified conventions, not invented.

**D1 — Bitwise reproducibility, including across thread counts.**
`Reproducibility::Bitwise` is defined as "Bit-identical across runs and across thread
counts" (`crates/sinbad-pal/src/repro.rs:20-21`), backends "honor the requested level or
fail honestly (they never silently degrade)" (`:14-16`), and residua asserts it for
assembly (`crates/residua/src/lib.rs:26-30`). Consequence for resolvent: **no OS entropy,
no wall clock, no address-dependent hashing, no `HashMap` iteration in any path that
affects output, and no result that depends on how work was partitioned across threads.**
`BTreeMap` over `HashMap` in any ordering-visible position — sinbad does exactly this and
says why: "`BTreeMap` for deterministic order → stable hash"
(`crates/sinbad-ir/src/ir.rs:26`).

**D2 — Modular methods must be deterministic or seeded, explicitly.** Resolvent's
architecture is "Modular methods everywhere … reduce mod several primes"
(`IDEAS-crates.md:126-127`). Random prime selection and random evaluation points make a
Las Vegas algorithm's *work* random; they must not make its *output bits* random. Either
use a fixed deterministic prime sequence, or take a `u64` seed as an explicit parameter.
Never `thread_rng()`. Sinbad's whole PAL exists to forbid exactly that:
"Nothing above this seam calls `Instant::now()`, `thread_rng()`, or reads a global"
(`crates/sinbad-pal/src/lib.rs:8-9`).

**D3 — No panics on input-dependent paths.** Convention E1: library code "**MUST NOT**
`panic!`, `unwrap()`, `expect()`, or index-panic on any input-dependent path"
(`SINBAD-API-CONVENTIONS.md:53-55`). An adapter cannot absorb a panic. Resolvent eats this
cost in full. Degree overflow, exponent-packing overflow, coefficient blowup, and
non-finite `f64` inputs are all `Result`, never abort. `lazy-exact` already models the
shape sinbad expects: `Rational::from_f64` returns `Option`
(`crates/meshwright/src/predicates.rs:93-96`).

**D4 — Declines are typed and distinct from errors.** tiered-core: "Errors are never
declines; refusals are typed (semantic vs not-implemented) and **fail closed**"
(`crates/tiered-core/src/lib.rs:19-20`). The decline vocabulary is
`OutOfRegime | ToleranceUnreachable | CannotCertify | Budget`
(`crates/tiered-core/src/rung.rs:13-26`), where `Decline` "always means *the next rung may
succeed*" (`:11`). Consequence: resolvent needs a **bounded-work mode**. "This subresultant
PRS exceeded the coefficient budget" must be a distinct, recoverable outcome from "your
polynomial was malformed" — the caller then escalates to a different rung. A resolvent that
can only run to completion or panic cannot be a rung.

**D5 — Unaccounted error caps the grade.** "Unaccounted error-budget sources cap the
effective grade — a partial bound cannot masquerade as a total one"
(`crates/tiered-core/src/lib.rs:21-22`), and `Grade::Proven` requires a machine-checkable
guarantee that does not substitute for `Measured` in either direction
(`crates/tiered-core/src/grade.rs:44-55`). Consequence: every resolvent result must be
honestly self-labelled as *exact* or *approximate*. A method that is exact for well-behaved
input and heuristic otherwise must say which happened, per call. Resolvent's own
self-certification plan (`IDEAS-crates.md:143-149`: multiply factors back, check ideal
membership via stored cofactors, check gcd divisibility both ways) is exactly the evidence
that mints `Proven` — expose the certificate, not just the answer.

**D6 — Canonical bytes for content addressing.** Generated artifacts are stored by BLAKE3
of their bytes (`crates/rutter/src/lib.rs:11-14`), and frozen schemas that feed a hash strip
provenance before hashing (`SINBAD-API-CONVENTIONS.md:165-168`). Consequence: resolvent
needs a **canonical serialization that is a pure function of mathematical content** —
independent of interning order, node ids, arena addresses, insertion history, and build
configuration — plus an explicit schema version, because a resolvent upgrade that changes
canonical form is a re-key event for every downstream artifact.

**D7 — Confine `unsafe`.** 11 of 28 sinbad library crates carry `#![forbid(unsafe_code)]`
(residua, plexus, ballast, reckon, league, rutter, and others); anvil is the documented
exception and justifies each site with a `SAFETY:` comment
(`crates/anvil/src/lib.rs:45-52`). Resolvent's bignum will want `unsafe`. Confine it to one
leaf crate with an auditable inventory, and keep everything above it `forbid(unsafe_code)`.

---

## 4. The adapter sketch — the acceptance test

Three adapters, each in sinbad, each requiring zero changes to resolvent. Line counts are
honest estimates of the real thing, not of the sketch.

### 4.1 `sinbad-testkit` MMS forcing generator (xtask, build-time) — ~110 lines

```rust
// xtask/src/mms_gen.rs — runs offline, emits a committed .rs file.
use resolvent::expr::{Store, Expr, Sym};

struct Gen { st: Store, x: Sym, y: Sym, pi: Sym }

impl Gen {
    fn new() -> Self { /* intern three symbols */ }

    // u* = sin(pi*x) * sinh(pi*y)
    fn harmonic_trig(&mut self) -> Expr {
        let (x, y, pi) = (self.st.var(self.x), self.st.var(self.y), self.st.var(self.pi));
        let a = self.st.sin(self.st.mul(pi, x));
        let b = self.st.sinh(self.st.mul(pi, y));
        self.st.mul(a, b)
    }

    // f = -div(kappa grad u) = -(dk/dx * du/dx + dk/dy * du/dy) - kappa*(uxx + uyy)
    fn poisson_forcing(&mut self, u: Expr, kappa: Expr) -> Expr {
        let ux  = self.st.diff(u, self.x);
        let uy  = self.st.diff(u, self.y);
        let uxx = self.st.diff(ux, self.x);
        let uyy = self.st.diff(uy, self.y);
        let kx  = self.st.diff(kappa, self.x);
        let ky  = self.st.diff(kappa, self.y);
        let lap = self.st.add(uxx, uyy);
        let adv = self.st.add(self.st.mul(kx, ux), self.st.mul(ky, uy));
        self.st.neg(self.st.add(adv, self.st.mul(kappa, lap)))
    }
}

// Printer: resolvent hands back a topological walk; we choose the target language.
// ~55 lines: match on node kind, emit f64 Rust, bind Sym("pi") -> std::f64::consts::PI,
// bind Sym("x")/Sym("y") -> closure params, let-bind every shared node (that is what
// hash-consing buys us: shared subexpressions become `let t7 = ...;`).
fn emit_rust(st: &Store, e: Expr, out: &mut String) { /* ... */ }
```

Requires from resolvent: symbol interning, an elementary-function node set with `diff`,
constant folding, and `walk_topological(expr) -> impl Iterator<Item = (NodeId, NodeRef)>`.
It does **not** require a Rust printer, a `simplify()`, or any numeric evaluation.

### 4.2 `plexus` symbolic `d/dt` for Pantelides — ~130 lines

```rust
// crates/plexus/src/symdiff.rs
use resolvent::expr::{Store, Expr, Sym};

/// plexus's variable convention: state variable `v` at differentiation order `n`.
/// The adapter owns this mapping; resolvent never learns what a "state variable" is.
struct DerVars { by_key: BTreeMap<(VarId, u32), Sym>, by_sym: BTreeMap<Sym, (VarId, u32)> }

impl DerVars {
    fn sym(&mut self, st: &mut Store, v: VarId, n: u32) -> Sym { /* intern "v#n" */ }
}

/// d/dt of an equation: chain rule, with d/dt(der(v,n)) = der(v,n+1).
/// resolvent supplies the chain rule; we supply the leaf rule via its seam.
fn ddt(st: &mut Store, dv: &mut DerVars, e: Expr, t: Sym) -> Expr {
    // resolvent's `diff_with(e, t, leaf_rule)` calls back on each unrecognised symbol.
    st.diff_with(e, t, |s| match dv.by_sym.get(&s) {
        Some(&(v, n)) => Some(dv.sym(st, v, n + 1)),   // a state variable
        None          => None,                          // a parameter: derivative 0
    })
}

/// Pantelides: the matching says WHICH equations to differentiate; we differentiate.
fn pantelides_step(sys: &mut FlatSystem, unmatched: &[EqId]) { /* ~50 lines */ }

/// Alias elimination: a = b / a = -b, decided by structural equality on canonical form.
fn is_alias(st: &Store, e: Expr) -> Option<(Sym, Sym, bool)> { /* ~25 lines */ }
```

Requires from resolvent: a **leaf-rule callback on `diff`** (`diff_with`). Without it the
adapter must reimplement the chain rule, and the sketch fails. This is the single most
consumer-shaped requirement in this document, and it is general: any consumer whose
"variables" have their own differentiation semantics needs it. `d/dt` of an implicitly
time-dependent unknown is a textbook CAS capability, so it passes rule 2's second clause.

### 4.3 `solverang` event root isolation — ~70 lines

```rust
// crates/solverang/src/events/exact_roots.rs
use resolvent::{Q, UPoly, isolate_roots, Interval};

/// Dense-output polynomial coefficients (f64) -> certified crossings in [0, h].
fn crossings_in_step(coeffs: &[f64], h: f64, budget: Budget)
    -> Result<Vec<Interval<Q>>, Decline>
{
    let mut c = Vec::with_capacity(coeffs.len());
    for &a in coeffs {
        c.push(Q::from_f64(a).ok_or(Decline::CannotCertify)?);   // fails closed on NaN/inf
    }
    let p  = UPoly::from_coeffs(c);
    let hi = Q::from_f64(h).ok_or(Decline::CannotCertify)?;
    isolate_roots(&p, Interval::new(Q::ZERO, hi), budget)
        .map_err(|_| Decline::Budget)          // over budget -> next rung, not an error
}
```

Requires from resolvent: exact `f64 -> Q` lift that fails closed; `UPoly<Q>` from a
coefficient slice; `isolate_roots` over a bounded interval returning isolating rational
intervals; and **a work budget that produces a decline rather than running forever**
(D4). Seventy lines, and the budget parameter is the only thing that is not already
implied by `IDEAS-crates.md:136-138`.

**Verdict on the acceptance test:** all three adapters fit well under 200 lines, with two
preconditions on resolvent's API that are not currently implied by the spec: an
elementary-function node set in L4 with a leaf-rule callback on `diff` (§5.1, §5.2), and a
budget-and-decline mode on L2 entry points (§5.4).

---

## 5. API pressure — what sinbad forces resolvent to be

### 5.1 L4 must admit non-polynomial function symbols, or it serves no sinbad use case

Every manufactured solution is `sin`/`sinh`/`exp`/`cos` (`mms.rs:158-196`). Every anvil
opcode set includes `Sqrt, Sin, Cos, Atan2, Exp, Ln, Pow, Tan, Asin, Acos, Sinh, Cosh,
Tanh` (`crates/anvil/src/opcodes.rs:108-243`). If L4 is "hash-consed DAG over L1
polynomials", sinbad has nothing to call.

The general shape: L4's node set is `{ Const(L0 element), Symbol(interned name),
Ring ops, Apply(FuncId, args) }` with a `FuncId` table carrying an arity and a derivative
rule. Elementary functions ship in the table. `π` is a `Symbol`, not a constant — resolvent
does not know its numeric value and does not need to; the consumer's printer binds it. This
keeps L4 honest: it is a term algebra with a differentiation calculus, and the *exact*
subset (where every leaf is an L0 element and every node is a ring op) is where L1/L2/L3
become applicable. Resolvent should expose that predicate: `expr.is_polynomial_in(&syms)
-> Option<MPoly>`, the bridge from L4 down to L1.

### 5.2 `diff` needs a leaf-rule callback

Pantelides differentiates with respect to an implicit `t` that appears in no expression;
the derivative of a state symbol is *another symbol the caller mints*
(`plans/dae-equation-compiler-architecture.md:280-281` describes exactly this: "repeatedly
differentiates equations w.r.t. time and **grows new variables**"). A `diff(expr, sym)`
that treats every other symbol as constant cannot express it. `diff_with(expr, sym,
leaf_rule: impl FnMut(Sym) -> Option<Expr>)` costs resolvent nothing and is the difference
between adapter 4.2 existing and not.

### 5.3 Traversal, not codegen

Sinbad needs Rust closures; another consumer will need C, or WASM, or its own opcode tape.
Resolvent must expose `walk_topological` with stable node ids and a `NodeRef` enum, and
must ship **no** code emitter. Shared-subexpression let-binding falls out of hash-consing
for free and is the main value the DAG adds over a tree.

### 5.4 A budget parameter on every L2 entry point

D4 requires a decline that is distinct from an error. Concretely: `isolate_roots(p,
interval, budget) -> Result<Vec<Interval<Q>>, ResolventError>` where the error enum
distinguishes `BudgetExhausted { .. }` from `MalformedInput { .. }`. This is a general
"fail closed" property, not a sinbad idiosyncrasy — and it is the only way a slow-exact
kernel can sit under a real-time-ish integrator loop at all.

### 5.5 `Q` must expose its size and its rounding

`meshwright` snaps to 53 bits by hand because unbounded growth is an observed performance
failure (`triangulate.rs:500-512`). Resolvent's `Q` needs: `num_bits()`/`den_bits()`,
`from_f64 -> Option<Q>` (exact, fails closed on non-finite), `to_f64` with a documented
rounding mode, and a `round_to_f64_grid()` or equivalent. It must **not** silently apply a
growth policy the caller cannot see.

### 5.6 Interning is an owned value, never ambient

Hash-consing via a thread-local or `static` interner breaks D1 and D6. The `Store`/`Arena`
is a value the caller constructs and owns. Node ids are meaningful only relative to a
store, are never serialized, and canonical bytes are computed structurally.

### 5.7 What sinbad wants that resolvent must refuse

- **`Ctx` as first parameter** (`SINBAD-API-CONVENTIONS.md:86-90`). Resolvent must not take
  a capability handle. The adapter's functions take `&Ctx` and ignore it, or take it and
  thread it to nothing. Adapter eats ~5 lines.
- **`DiagCode` on every error** (`:62-70`). Sinbad-specific registry. Adapter writes one
  `impl From<ResolventError> for PlexusError` with a code mapping. ~20 lines.
- **`league::Quantity` on dimensioned results** (`:178-184`). Resolvent knows nothing about
  units and must not. Adapter attaches.
- **`thiserror` + no hand-rolled `Display`** (`:57-61`). Fine to adopt — it is a good
  default independent of sinbad — but adopt it because it is good, not because sinbad said
  so.

---

## 6. Conflicts and who ate the cost

| Conflict | Resolution | Who paid |
|---|---|---|
| Sinbad wants `Ctx` first-param everywhere; resolvent must be ambient-free | Resolvent takes no capability handle; adapter accepts and drops `Ctx` | adapter (~5 lines) |
| Sinbad wants `DiagCode` on every error; resolvent has no diagnostics registry | Resolvent's error enum is rich and stable; adapter maps variants → codes | adapter (~20 lines) |
| Sinbad's L4-shaped need is transcendental; resolvent's centre of gravity is polynomial | L4 gets an open `FuncId` table with derivative rules; the polynomial subset is recovered by `is_polynomial_in` | **resolvent** — this is a genuine scope addition |
| Sinbad's `Exp` is a `Copy` `i16/i16` rational; resolvent's `Q` is a bignum | Resolvent does not offer `Q` as a units type; league keeps its own | neither — no overlap once named |
| Sinbad's exact predicates want bounded bit-length; resolvent's `Q` grows | Resolvent exposes size and rounding; policy stays with the caller | **resolvent** (API surface), sinbad keeps its snap loop |
| Sinbad's e-graph need is FP-accuracy rewriting; resolvent's is exact rewriting | Resolvent does not own anvil's e-graph; anvil calls `egg` directly | sinbad (correctly) |
| Sinbad demands `Bitwise` reproducibility; modular methods want randomness | Deterministic prime sequence or explicit seed parameter; never `thread_rng()` | **resolvent** — non-negotiable |
| Sinbad's rung protocol needs declines; resolvent naturally runs to completion | Budget parameter + `BudgetExhausted` distinct from `MalformedInput` | **resolvent** |

---

## 7. What would settle the open questions

1. **Does anyone actually build the `SourceField::Function` seam?** Until residua can
   assemble a spatially varying `f`, MMS forcing generation has no consumer, and finding
   #1 collapses from "highest-value lead" to "generator with no sink." Watch
   `crates/residua/src/lib.rs:358-361`.
2. **Does the numerical-stabilization DAE path ship first, as planned?** If yes
   (`plans/dae-equation-compiler-architecture.md:578-582`), plexus's symbolic layer stays
   deferred indefinitely and the demand never becomes urgent. If the acausal/Modelica
   stratum gets prioritized instead, §1.2 becomes the real driver.
3. **Does solverang's integrator get written with exact Sturm isolation, or with a float
   bracket-and-bisect?** This decides whether resolvent has any **per-operation** consumer
   in sinbad at all, and therefore whether sinbad constrains resolvent's performance
   envelope or only its API shape.
4. **Do the transcendental MMS families ever need coefficient exactness?** Today the
   generated forcing is emitted as f64 Rust and evaluated in f64. If the emitted constants
   are always f64 literals, resolvent's L0 never enters the MMS picture and L4 could
   in principle be built over a plain `f64` leaf — which would make resolvent the wrong
   tool for the job and a 400-line `sinbad-symdiff` crate the right one. Resolvent's
   counter-argument has to be that the *same* DAG feeds the exact path (§1.3, §5.1
   `is_polynomial_in`) — if it does not, sinbad should not depend on resolvent for MMS.
5. **Does the arrangements/lazy-exact unification happen?** Sinbad depends on
   `lazy-exact` through `meshwright` (`/home/dev/sinbad/Cargo.toml:67-68`,
   `crates/meshwright/Cargo.toml:15-16`), not on resolvent. If resolvent and
   `lazy-exact` ever merge, sinbad becomes a resolvent consumer at L0 by transitivity
   without ever having asked for it — and D1/D3/D5/D7 above become load-bearing on
   resolvent's *entire* L0, not just on the paths sinbad calls directly. That is the
   scenario in which this document's determinism contract matters most.

---

## 8. Out-of-scope note: `sem1f-biquad-spike`

`/home/dev/sinbad/crates/sem1f-biquad-spike/` hand-rolls a biquadratic ℚ-algebra
`ℚ[α, γ]` with `α² = 1/A`, `γ² = 1/(sA)` over `lazy_exact::Rational`
(`src/main.rs:70-80`), and proves trig-polynomial identities by exact evaluation at
rational Weierstrass points — "exact agreement at more samples than the degree bound is a
proof" (`src/main.rs:15-18`). That is textbook L0 algebraic-extension arithmetic plus a
Schwartz–Zippel identity test, and it is the strongest algebraic-extension evidence in the
directory.

I am excluding it from sinbad's demand set, for two reasons. It is **not a workspace
member** (`Cargo.toml:5-45`), and it imports `cadabra_geom` — it belongs to the cadabra2
geometry line, and should be counted there (E2/E3), not here. Counting it as sinbad demand
would double-count the same evidence across two consumer evaluations and inflate the
two-consumer rule.
