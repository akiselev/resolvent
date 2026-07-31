# ADR-014 — `AlgebraicReal` exposes no `Hash` and no general arithmetic; multiplicity is not a field

**Status:** Ratified 2026-07-31
**Reversibility:** one-way (adding these later is additive; removing them later is breaking,
and shipping them wrong corrupts consumers silently)
**Amended:** 2026-07-31 — multiplicity is returned in a named `IsolatedRoot` struct rather
than a bare tuple; `SqrtExt<T>`'s public generic parameter is decided; §5's
"instantiation, not a rewrite" claim is corrected against ADR-006's `Reducible` fix
(critique-plan C14; critique-engineering §4).
**Gates lanes:** A1, A2, A3, K3, M8-N.
**Evidence:** `docs/research/algorithms-and-representation.md` §8.2 (F7, F8, F9);
`docs/research/consumer-requirements.md` §0.2, §2.2, §7 D5;
`docs/research/critique-plan.md` C14; `docs/research/critique-engineering.md` §4.

---

## Context

Three traps, all of which look like missing features and all of which are load-bearing
omissions.

**F7 — `Eq`/`Hash` inconsistency.** Two `AlgebraicReal`s can be *equal* while carrying
different defining polynomials: `x² − 2` and `x⁴ − 4` both have the root `√2`. Under
ADR-013, `Eq` is the gcd-plus-sign-change test, so those two compare **equal**. A `Hash`
that hashes the polynomial hashes them **differently**, and a `HashMap` silently holds two
entries for one number. **No unit test catches this**; it shows up as nondeterministic
behaviour in a consumer. The only consistent fix is a canonical form — the *minimal*
polynomial plus a root index — and computing the minimal polynomial requires factorization
over ℚ.

**F8 — degree blowup under arithmetic.** `α + β` has a defining polynomial of degree
≤ `deg α · deg β`, computed as `Res_y(f(y), g(x − y))`; products are the same. Without
reducing to the minimal polynomial at every step, degree 4 + degree 4 → 16 → 256 → 65536
after **three** operations. Reducing at every step means a factorization per operation.

**F9 — multiplicity is not a property of a value.** `√2` has no multiplicity. Multiplicity
is a property of the *polynomial the root was isolated from*.

And the decisive empirical observation from R2: **the consumer never does algebraic-number
arithmetic.** `RealRoot` (`crates/lazy-exact/src/roots.rs:317-612`) has no `add`, no `mul`,
no `div`, and **four curve families ship without them**. Points are carried as *(isolated
root ξ, a representation over ξ)*, never as a materialized element of ℚ(ξ). The mechanism
is a normal form plus a sign ladder:

```rust
// conics.rs:51-56 — an element of ℚ(ξ)[√h], only ever SIGNED, never added
pub struct YRep { a: QPoly, b: QPoly, h: QPoly, d: QPoly }  // y = (a(ξ) + b(ξ)√h(ξ)) / d(ξ)
```

signed by `sign_radical1`/`sign_radical2` (`roots.rs:622-683`). **That is why the consumer's
predicates stay in degree 4.** Field arithmetic in ℚ(α) is not must-have for geometry. It
*is* must-have for SMT NRA (multi-level sample points) — which is the sharpest consumer
incompatibility in the whole requirement set, and the biggest available scoping win.

---

## Decision

### 1. No `Hash` on `AlgebraicReal`

```rust
// NOT implemented:  impl Hash for AlgebraicReal
pub struct CanonicalAlgebraicReal { /* minimal polynomial + root index */ }
impl Hash for CanonicalAlgebraicReal {}
impl Eq   for CanonicalAlgebraicReal {}
impl AlgebraicReal {
    /// Costs a factorization over ℚ. Loudly documented. Opt-in.
    pub fn canonicalize(&self) -> Result<CanonicalAlgebraicReal>;
}
```

A consumer that needs a hash map keyed by algebraic numbers pays the factorization once,
explicitly, at a call site that says so. **A "cheap" `Hash` is not offered at any price**,
because the failure it causes is silent, nondeterministic, and untestable from the
consumer's side.

`CanonicalAlgebraicReal` is also the serialization form (ADR-012 §9) — minimal polynomial
plus 0-based ascending root index — so the canonicalization work is shared between the two
uses.

### 2. No general arithmetic on `AlgebraicReal` by default

Not offered: `Add`, `Sub`, `Mul`, `Div`, `Neg` on `AlgebraicReal`.

Offered instead — the operations a consumer actually calls:

```rust
impl AlgebraicReal {
    pub fn sign_of(&self, h: &UPoly<Integer>) -> Sign;   // 20 call sites in the consumer
    pub fn is_root_of(&self, h: &UPoly<Integer>) -> bool;
    pub fn cmp_rational(&self, q: &Rational) -> Ordering;
}
/// Exact sign of Σ cᵢ(α)·√hᵢ(α) at arbitrary depth. Generalizes the consumer's
/// depth-1 and depth-2 ladders; depth ≥ 3 exists nowhere today.
pub fn sign_radical_tower(
    coeffs: &[UPoly<Integer>], radicands: &[UPoly<Integer>], at: &AlgebraicReal,
) -> Sign;
pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal]) -> Rational;
```

The escape hatch for consumers who genuinely need field arithmetic (SMT NRA):

```rust
pub mod tower {
    /// Materialize an element of ℚ(α) as an AlgebraicReal over ℚ.
    /// Costs a resultant of degree deg(α)·deg(expr) and a factorization to reduce
    /// to the minimal polynomial. Do not call this in an inner loop.
    pub fn materialize(expr: &TowerElement, at: &AlgebraicReal) -> Result<AlgebraicReal>;
}
```

Opt-in, in its own module, with the cost in the doc comment. The **documented fast path is
the sign ladder; materialization is the general fallback**, and the API docs say which is
which so consumers do not silently pick the slow one.

### 3. Multiplicity is a pair element, never a field

```rust
pub struct IsolatedRoot { pub value: AlgebraicReal, pub multiplicity: u32 }
pub fn isolate_roots(p: &UPoly<Integer>, b: Budget) -> Result<Vec<IsolatedRoot>>;
```

*Amended 2026-07-31: a named struct, not a bare tuple.* The safety property this section
exists for is that multiplicity is **not part of `AlgebraicReal`** and cannot participate in
`Eq`, `Ord`, `Hash` or `sign_of` — and `IsolatedRoot` preserves that in full. What the bare
tuple additionally did, without buying anything, was force every consumer that *stores* a
root and later asks its multiplicity to thread a parallel structure. The nearest prior art
does exactly that (`arrangements/crates/lazy-exact/src/roots.rs:438`,
`RealRoot::multiplicity(&self) -> u32`, a method on a stored value), so under ADR-018's
option B the adapter would have had to define its own pair type — which falsifies the
"a merge is a rename plus `&mut → &self`" claim — and under option C it would be a
mechanical edit at every storage site. The struct keeps `root.multiplicity` working, keeps
the value storable as one thing, and is strictly better for docs and for `serde`. It is
additive and free, and it is only free *now*.

Never `AlgebraicReal { multiplicity: u32 }`. The prior art conflates them
(`roots.rs:321`) and reads multiplicity off a *resultant* root to infer crossing parity
(`conics.rs:569`) — which works only because the conflation exists. Two consequences of
separating them, both correct:

- Two values that are equal but were isolated from different polynomials with different
  multiplicities compare **`Equal`**. Any comparison that includes multiplicity in a
  tie-break is wrong (F9), and with multiplicity off the type this is impossible to get
  wrong.
- A consumer that wants intersection multiplicity gets it from the *resultant analysis*,
  where it belongs, not from a number.

### 4. `SqrtExt` is first-class and is never subsumed into `AlgebraicReal`

Stated as a decision rather than left implicit, because nothing else forbids it and the
regression it would cause is silent.

```rust
pub struct SqrtExt<T> { /* a + b·√r */ }
impl<T: Ordered + Field> SqrtExt<T> {
    pub fn sign(&self) -> Sign;                          // by squaring, exact
    pub fn cmp_cross(&self, o: &SqrtExt<T>) -> Ordering; // TOTAL across different r
}
pub type SqrtExtQ = SqrtExt<Rational>;                   // the documented default
```

**The generic parameter on `SqrtExt` is kept, deliberately, and this is decided here rather
than left to drift.** *Added 2026-07-31.* `plans/architecture.md` §5.4 wrote it generic and
`plans/api-shape.md` wrote it monomorphic; ADR-018 forbids a public generic parameter on
`AlgebraicReal` by name and was silent about `SqrtExt`, which is the type it *also* requires
stay first-class. A public generic parameter is the same one-way door for the same reason,
so it needs the same explicit answer.

Generic wins here and monomorphic does not, for one reason that does not apply to
`AlgebraicReal`: `SqrtExt` is a *construction over a base field*, and its two genuinely
wanted instantiations — `SqrtExt<Rational>` for the degree-2 fast path, and `SqrtExt<T>`
over a tower base for `sign_radical_tower`'s recursive step — are both resolvent's own.
`AlgebraicReal` by contrast has exactly one coefficient domain (ℤ) and its generic parameter
would exist only to admit a *consumer's* scalar, which is ADR-018's option A. The
instantiation set stays closed to resolvent (ADR-006 Tier G) and `SqrtExtQ` is the alias
every consumer-facing signature uses, so an adapter never writes the parameter.

Degree-2 radicals are **not** routed through defining-polynomial + isolating-interval
machinery. The evidence is direct and quantified: `circle_segments.rs` is 931 LOC that uses
`SqrtExt` exclusively and never imports `RealRoot` or `QPoly`, and `SqrtExt::cmp_cross`
(`crates/lazy-exact/src/sqrt_ext.rs:187-222`) has 31 call sites. Routing that through the
general machinery would replace an exact sign-by-squaring with a gcd, an isolation, and a
refinement loop, on the cheapest and most common case in the entire consumer.

`SqrtExt` is also the return type of the `cmp_y_right_of` witness fast path — evaluating a
branch at a *rational* abscissa yields a degree-2 radical, not an algebraic number
(`conics.rs:382-395`). That path must not allocate a defining polynomial.

Conversion `SqrtExt<Rational> → AlgebraicReal` exists and is explicit. The reverse does not.

### 5. `UPoly<C>` is generic from day zero so `UPoly<NumberField>` is an instantiation

`AlgebraicReal`'s coefficient domain is **ℤ only**. But `UPoly<C>` is generic (ADR-006
Tier G) from the first commit, so the SMT NRA requirement — isolating roots of polynomials
with coefficients in ℚ(α₁,…,α_k) — arrives later as an *added instantiation*
(`UPoly<NumberFieldElem>` behind the `number-fields` feature) plus a second algebraic-number
type, not as a rewrite of `AlgebraicReal`.

**Correction, 2026-07-31: "an added instantiation, not a rewrite" is true for correctness
and false for speed, and the difference is the whole of M8.** `UPoly<NumberFieldElem>`
compiles the moment `NumberFieldElem: Ring` exists — that part of the claim stands, and it
is worth what it cost. What does *not* come free is the modular fast path: `Reducible` over
ℚ(α) lands in `GF(p)[x]/(f mod p)`, which is a field only at an inert prime, and for the
multiquadratic towers geometry produces **no prime is inert at all** (ADR-006 §Context
defect 3). The fast path over an algebraic extension is **multi-modular over split factors**
— a different algorithm with its own bad-prime predicate (ADR-010 §4) — and it is a **lane**
(M8-N), sized as one. Without that lane, `UPoly<NumberFieldElem>` silently gets the Tier-G
reference path: correct, and not fast enough for SMT NRA's inner loop, which is the consumer
M8 exists for.

This is the resolution of the sharpest consumer incompatibility: geometry provably never
needs algebraic numbers over ℚ(α); SMT provably does. Ship the simple one, keep the general
one an instantiation away.

---

## Consequences

- **The consumer's predicates stay in degree 4** — or, at arbitrary degree, in degree
  `deg(ξ)` rather than `deg(ξ)^{2^k}`. That is the entire performance story for geometry
  predicates and it is a consequence of an API omission, not of an optimization.
- **`AlgebraicReal` cannot be a `HashMap` key without an explicit conversion.** Some
  consumers will find this annoying. It is the correct annoyance: the alternative is silent
  corruption.
- **`sign_radical_tower` at arbitrary depth is a real implementation obligation**, not just
  an omission of arithmetic. Depth 1 and 2 exist in the prior art by squaring; depth ≥ 3
  exists nowhere and the consumer currently writes ~150 lines of by-hand ladder per curve
  family. If resolvent ships only `sign_of(P)` for `P ∈ ℚ[x]`, every consumer re-derives
  those ladders. This ADR only pays off if the tower is built.
- **`materialize` must exist even though it is slow**, because R2 §8 Q2 is unsettled: nobody
  has measured whether the ladder actually beats materialization at realistic degree. Both
  paths existing is what makes that measurement possible, and it is also a free differential
  oracle (the two must agree on sign at every test point).
- **Multiplicity semantics move to the resultant lane.** Whether intersection multiplicity
  comes from resultant-root multiplicity or from factoring the resultant is an open question
  (R2 §8 Q5); this ADR makes sure the answer is not accidentally baked into the number type
  before it is known.

---

## Alternatives considered and why rejected

**Implement `Hash` on the defining polynomial and document "only hash canonical values".**
Rejected. It is an `Eq`/`Hash` contract violation, and the Rust ecosystem's collections
rely on that contract. A documented footgun in a foundational type is a bug generator, and
the failure is nondeterministic.

**Implement `Hash` by always canonicalizing internally.** Rejected: `Hash::hash` takes
`&self`, cannot fail, and cannot afford a factorization. A `Hash` impl that may run a
polynomial-time factorization is a denial-of-service in a hash map.

**Full field arithmetic with automatic minimal-polynomial reduction after every operation.**
Rejected as the default. It costs a factorization per operation and it hides that cost
behind `+`. Kept as `tower::materialize`, opt-in, with the cost stated.

**Full field arithmetic without reduction** (let degrees grow). Rejected: three operations
reach degree 65536.

**Make `AlgebraicReal` generic over its coefficient domain now** (tower-generic from day
one). Rejected. It is 5–10× slower on the geometry path, which is 100% of near-term usage,
and it puts a generic parameter on the headline type — which ADR-018 identifies as the
single most expensive thing to add if it turns out wrong. The `UPoly<C>`-generic-from-day-
zero decision gets the same future capability at none of the present cost.

**Keep `multiplicity` on the number "because the consumer does it".** Rejected. The consumer
does it because it has one caller and no separation between "root of this polynomial" and
"real number"; the research explicitly flags it as wrong in the prior art.

---

## What would reverse this

- **A consumer needing `Hash` in a hot loop.** Response: cache `CanonicalAlgebraicReal`
  alongside the value at the consumer's level, or add an interning table in resolvent that
  hands out `AlgebraicId`s. Both additive; neither requires a `Hash` on the un-canonical
  type.
- **The R2 §8 Q2 measurement showing materialization is *not* materially slower than the
  ladder at realistic degree.** Then the ladder stops being load-bearing and general
  arithmetic could be offered with per-operation reduction. That would be a genuine
  reversal, and it is exactly the measurement to run before building `sign_radical_tower`
  at arbitrary depth.
- **SMT NRA becoming the primary consumer.** Then `UPoly<NumberFieldElem>` and a
  tower-generic algebraic number become first-class, alongside — not instead of — the ℤ-only
  one. That is the instantiation path this ADR keeps open, so it is a planned extension
  rather than a reversal.
