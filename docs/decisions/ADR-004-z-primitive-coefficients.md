# ADR-004 — Coefficients are ℤ-primitive; ℚ is a boundary façade

**Status:** Ratified 2026-07-31
**Reversibility:** one-way
**Gates lanes:** U1, U3, U5, A1, T1, T2, T3.
**Resolves:** the ℤ-vs-ℚ contradiction in the contradiction register (ADR-021 §3, item 4).
`AlgebraicReal`'s defining polynomial is `UPoly<Integer>`; `SqfrPoly<Rational>` is a
transport type only and appears in no stored field.
**Evidence:** `docs/research/prior-art-and-licensing.md` §1.3; `docs/research/consumer-requirements.md` §3.1 "Difference 1"; `docs/research/algorithms-and-representation.md` §9.1.

---

## Context

The nearest reference implementation in this workspace does the opposite of what this ADR
decides, which is exactly why it has to be written down.

`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:43-45`:

```rust
pub struct QPoly {
    coeffs: Vec<Rational>,
}
```

— dense univariate over ℚ, where `Rational` wraps `dashu::rational::RBig`. Its Euclidean
gcd monic-normalizes at every step (`roots.rs:169-182`), multiplying denominators as it
goes. At degree ≤ 4 with small coefficients, that is entirely fine, and it ships.

It does not survive the workload resolvent exists for. Two independent reasons:

**1. Rational arithmetic renormalizes with a bignum GCD on every operation** — and GCD is
precisely where pure-Rust is structurally behind GMP. `dashu` has Lehmer (quadratic worst
case); GMP has a subquadratic half-GCD (ADR-002). So the one operation that ℚ-primitive
code performs most often is the one operation where the permissive bignum is weakest. The
license constraint and the coefficient-domain choice are coupled.

**2. Coefficient growth.** A resultant of two degree-`d` bivariate polynomials with `τ`-bit
coefficients has degree up to `2d²` in the remaining variable and coefficients of
`O(dτ + d log d)` bits. Two degree-10 curves with 32-bit coefficients already give a
degree-200 polynomial with ~500-bit coefficients. Every intermediate in a ℚ-primitive
Euclidean chain carries a denominator that also grows, and each step pays a GCD to
renormalize it. This is the "coefficient explosion and a dead project" the source spec
warns about, in its cheapest form.

The standard CAS answer is not novel — Cohen, von zur Gathen & Gerhard, and Geddes/Czapor/
Labahn all teach it. It is written down as a *decision* because an agent reading the nearest
available Rust implementation will copy its shape.

---

## Decision

**resolvent's polynomial coefficients are ℤ-primitive. ℚ exists only as a thin façade at
the API boundary.**

Concretely:

1. **Ingress**: a `UPoly<Rational>` handed to resolvent is immediately converted by
   `clear_denominators()` to `(UPoly<Integer>, Integer)` — the primitive part over ℤ plus
   the single scalar content factor that was removed. `UPoly<Rational>` is a *transport*
   type, not a working type.
2. **Egress**: the content factor is reattached once, at the boundary, if the caller asked
   for a rational answer. For sign, comparison, ordering, root isolation, and divisibility
   the content is a positive scalar and is irrelevant — so most calls never reattach it.
3. **No inner loop anywhere in resolvent calls a rational GCD.** The internal algorithms
   over ℤ use pseudo-division, subresultant PRS, content/primitive-part normalization, and
   modular reconstruction — never `Rational::add`.
4. **`AlgebraicReal`'s defining polynomial is `UPoly<Integer>`**, squarefree and primitive,
   with positive leading coefficient. Its *isolating interval endpoints* are `Rational`,
   which is correct and cheap: bisection on dyadic endpoints keeps denominators as powers
   of two, and rational endpoints are compared, not accumulated.
5. **The canonical associate is defined over ℤ**: content removed, leading coefficient
   positive. This is what makes the associate test an `==` rather than the O(k²) all-2×2-
   minors hand-roll the consumer currently uses (`conics.rs:259-270`).
6. **Root isolation works in ℤ[x] on dyadic intervals**, not in ℚ[x] on arbitrary rational
   intervals. With dyadic endpoints the interval transforms are `x → x+1` (an integer
   Taylor shift) and `x → 2^k x` (a coefficient shift). With arbitrary rational endpoints
   every subdivision multiplies denominators. The prior art takes the expensive route
   (`roots.rs:270-288` composes affine maps with `Rational` coefficients on arbitrary
   `(lo,hi)`) and R3 §7.3 names this the single highest-leverage thing to change.

`Rational` remains a first-class public type — consumers hand in rational curve
coefficients and receive rational interval bounds and rational witnesses
(`rational_between`). It is simply not the type the engine computes in.

---

## Consequences

- **Every Layer-2 algorithm is written over ℤ.** Pseudo-division rather than division;
  subresultant PRS rather than monic Euclid; content tracking as an explicit obligation.
  This is more code and more care than the ℚ version, and the extra care is the point.
- **Modular methods become natural rather than bolted on.** Reduction mod `p` of a ℤ
  polynomial is a coefficient-wise map; reduction of a ℚ polynomial first requires clearing
  denominators anyway. ADR-010 assumes this ADR.
- **Content bookkeeping is a real correctness surface.** Losing a content factor is a
  silent wrong answer for divisibility and an invisible one for sign. Mitigation: the
  canonical-associate invariant is enforced at construction (ADR-011) and property-tested
  (`primitive_part(k·p) == primitive_part(p)` for all nonzero `k`;
  `content(p) * primitive_part(p) == p`).
- **Signs need care.** ℤ-primitive normalization fixes `lc > 0`, which flips the sign of the
  polynomial when the input had `lc < 0`. Every sign-returning API must be defined against
  the *normalized* polynomial and must document it, or `sign_of` silently inverts. This is
  a property test, not a review item.
- **The pressure on `dashu`'s Lehmer GCD drops sharply** but does not vanish — rational
  reconstruction and content computation still call `gcd` on large integers. The
  measurement in ADR-002 (`gcd`/`gcd_ext` at 64 / 256 / 1k / 4k / 16k bits) tells us how
  much residual exposure remains.
- **Consumers see no difference** except that the fast paths are fast. The transport type is
  still ℚ.

---

## Alternatives considered and why rejected

**ℚ-primitive (copy the prior art).** Rejected. It couples the hottest operation to the
weakest primitive (§Context 1), it makes coefficient growth invisible until it is fatal,
and it makes the modular path a retrofit rather than the default. The prior art's own
scope note is honest about why it works there: its consumer is "the degree-≤4 conic/quartic
event algebra" (`roots.rs:13-14`).

**Generic over the coefficient ring with ℚ as a supported working domain.** Rejected as a
*default*, kept as a capability. `UPoly<Rational>` will compile and produce correct answers
via the generic reference path (ADR-006 Tier G) — but it is not the path any resolvent
algorithm chooses internally, and the docs say so. Making it available is free; making it
the default is the failure this ADR prevents.

**Dyadic-rational-primitive (ℤ[1/2]).** Considered because bisection naturally produces
dyadics. Rejected as the *coefficient* domain: it buys nothing over ℤ plus an explicit
exponent, and it complicates modular reduction (a denominator `2^k` must be inverted mod
`p`, which is fine but is an extra invariant to carry through every algorithm). Dyadics are
used where they belong — interval endpoints and the `scale_pow2` transform.

**Fixed-precision integers with promotion.** Rejected. It reintroduces a silent-overflow
failure mode into the coefficient layer, which is the same class of bug ADR-008 goes to
some length to eliminate in the exponent layer.

---

## What would reverse this

Effectively nothing, because reversal is a rewrite of every algorithm in
`resolvent-algebra` and `resolvent-real`. That is what "one-way door" means here, and it is
why the decision is made before fan-out rather than discovered during it.

The nearest thing to a partial reversal that would still be coherent: if the R2 §8 Q1
corpus measurement shows the real geometry workload never exceeds degree ~8 with ~64-bit
coefficients, then a well-implemented ℚ subresultant PRS might win *at that size* and the
modular machinery would be complexity paid for nothing on the geometry path. Even then this
ADR stands, because (a) ℤ-primitive is not slower than ℚ-primitive at any size — it is the
denominators, not the domain, that cost — and (b) consumers #12 (SMT NRA) and #27 (medial
axis) provably exceed that size. The measurement changes the *sequencing* of the modular
lane, not the coefficient domain.
