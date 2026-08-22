//! [`Dual`] — a forward-mode differentiable number that is **itself** a
//! [`Scalar`], so "differentiable" becomes a third instantiation of the *same*
//! generic numeric codebase rather than a fork.
//!
//! # The thesis: `Dual<S: Scalar>` is `Scalar`
//!
//! A [`Dual`] carries a value together with one directional derivative
//! (tangent): `(value, deriv)`. Every field operation propagates the derivative
//! by the calculus chain rule — product rule for [`Mul`], quotient rule for
//! [`Div`], sign for [`abs`](Scalar::abs), and (on the floating tier only) the
//! standard rules for the [`ApproxScalar`] transcendentals. Because those rules
//! are expressed purely through the `S`-level arithmetic, `Dual<S>` satisfies
//! [`Scalar`] for **any** `S: Scalar`. The consequence is the whole point of the
//! seam: one generic kernel `fn kernel<S: Scalar>(…)` instantiates at
//!
//! - `Dual<f64>`  → fast forward-mode automatic differentiation, and
//! - `Dual<Real>` → **exact** derivatives, with no finite-difference truncation
//!   noise and no floating round-off — the sensitivity is certified equal to the
//!   closed-form analytic derivative, bit for bit,
//!
//! from the *identical* source. "Exact", "fast", and "differentiable" are three
//! instantiations of one codebase, not three codebases.
//!
//! # How to use it
//!
//! Seed the parameter you are differentiating with respect to as a
//! [`variable`](Dual::variable) (derivative `1`); seed every other input as a
//! [`constant`](Dual::constant) (derivative `0`). Run the ordinary generic
//! kernel. Each output's [`deriv`](Dual::deriv) is the partial derivative of that
//! output with respect to the seeded parameter.
//!
//! ```
//! use resolvent::{Dual, Scalar};
//!
//! // g(x) = (x^3 - 2x) / (x^2 + 1); differentiate at x = 2.
//! fn g<S: Scalar>(x: S) -> S {
//!     let num = x.clone() * x.squared() - S::from_i32(2) * x.clone();
//!     let den = x.squared() + S::one();
//!     num / den
//! }
//! let y = g(Dual::variable(2.0_f64));
//! assert!((y.value() - 4.0 / 5.0).abs() < 1e-12);   // g(2)  = 4/5
//! assert!((y.deriv() - 34.0 / 25.0).abs() < 1e-12); // g'(2) = 34/25
//! ```
//!
//! # Comparison rides the *standard part*
//!
//! [`PartialEq`] and [`PartialOrd`] on a `Dual` look at the **value** only; the
//! derivative is an infinitesimal tangent that rides along and never
//! participates in a comparison. This is deliberate and load-bearing: a generic
//! kernel's control flow (pivot choice, sign test, `max`-norm) must branch on the
//! value, and the derivative then propagates through whichever branch the value
//! selected — exactly how forward-mode AD is supposed to behave. It also keeps
//! `PartialEq`/`PartialOrd` mutually consistent (`a == b ⇔ partial_cmp == Equal`).
//!
//! # The boundary — forward-mode is O(#params)
//!
//! One `Dual<S>` carries **one** tangent, so recovering the gradient of a scalar
//! output with respect to `p` parameters costs `p` forward solves (one seed per
//! parameter). That is the right tool for a handful of design variables and the
//! clean first proof that "differentiable = a third `Scalar` instantiation", but
//! it does **not** scale to many-parameter shape optimization.
//!
//! # Next increment: reverse-mode adjoint over the construction DAG
//!
//! The design-velocity engine the ecosystem actually wants is **reverse mode**:
//! one adjoint sweep yields `∂(QoI)/∂p` for *all* parameters at once, O(1) in the
//! number of parameters (per linear solve), instead of O(#params) forward passes.
//! The natural substrate is already present one layer down: `resolvent::Real` is
//! an **expression-DAG handle** (`crates/resolvent/src/real.rs`) — every `Real`
//! records the operation and operands that built it. A reverse-mode number would:
//!
//! 1. seed the output cotangent `= 1`;
//! 2. walk that construction DAG in reverse topological order, accumulating each
//!    node's adjoint by the transpose of its local Jacobian (`+`, `−`, `×`, `÷`,
//!    `abs`), which are the *same* local rules used here, transposed; and
//! 3. read off `∂output/∂leaf` at every input leaf simultaneously.
//!
//! Because the DAG is exact, the accumulated adjoints are **exact `∂x/∂p`** —
//! the certified reverse-mode sensitivity. This composes directly with the two
//! ecosystem adjoints named in `cadabra-unification-plan.md` §4: the physics
//! adjoint (`residua::AdjointAction::apply_transpose`) and the geometry adjoint
//! (cadabra's closed-form design velocity), giving one reverse-mode graph
//! `design → geometry → mesh → field → QoI → gradient`. That is the larger
//! follow-on; forward-mode [`Dual`] here is the bounded first rung.

use crate::{ApproxScalar, Scalar};
use core::cmp::Ordering;
use core::ops::{Add, Div, Mul, Neg, Sub};

/// A forward-mode dual number `value + deriv·ε` (with `ε² = 0`) that is itself a
/// [`Scalar`].
///
/// `value` is the ordinary result; `deriv` is its derivative with respect to
/// whichever input was seeded as the active [`variable`](Dual::variable). See
/// this module's documentation for the design (comparison on the standard part, the
/// O(#params) boundary, and the reverse-mode follow-on).
#[derive(Clone, Debug)]
pub struct Dual<S: Scalar> {
    /// The value (the *standard part*).
    pub value: S,
    /// The first-order derivative (the *tangent*) carried alongside the value.
    pub deriv: S,
}

impl<S: Scalar> Dual<S> {
    /// A dual number with an explicit value and derivative.
    #[inline]
    pub fn new(value: S, deriv: S) -> Self {
        Dual { value, deriv }
    }

    /// A **constant** input: value `v`, derivative `0`. Use for every input the
    /// output is *not* being differentiated with respect to.
    #[inline]
    pub fn constant(value: S) -> Self {
        Dual {
            value,
            deriv: S::zero(),
        }
    }

    /// The **active variable**: value `v`, derivative `1` (a unit seed). Use for
    /// the single input the output *is* being differentiated with respect to.
    #[inline]
    pub fn variable(value: S) -> Self {
        Dual {
            value,
            deriv: S::one(),
        }
    }

    /// The value (standard part).
    #[inline]
    pub fn value(&self) -> S {
        self.value.clone()
    }

    /// The derivative (tangent) with respect to the seeded variable.
    #[inline]
    pub fn deriv(&self) -> S {
        self.deriv.clone()
    }
}

// Comparison rides the standard part (value) ONLY — see the module docs.
impl<S: Scalar> PartialEq for Dual<S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<S: Scalar> PartialOrd for Dual<S> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<S: Scalar> Add for Dual<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        // (u + v)' = u' + v'
        Dual {
            value: self.value + rhs.value,
            deriv: self.deriv + rhs.deriv,
        }
    }
}

impl<S: Scalar> Sub for Dual<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        // (u - v)' = u' - v'
        Dual {
            value: self.value - rhs.value,
            deriv: self.deriv - rhs.deriv,
        }
    }
}

impl<S: Scalar> Mul for Dual<S> {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // product rule: (u·v)' = u'·v + u·v'
        let value = self.value.clone() * rhs.value.clone();
        let deriv = self.deriv * rhs.value + self.value * rhs.deriv;
        Dual { value, deriv }
    }
}

impl<S: Scalar> Div for Dual<S> {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        // quotient rule: (u/v)' = (u'·v − u·v') / v²
        let value = self.value.clone() / rhs.value.clone();
        let num = self.deriv * rhs.value.clone() - self.value * rhs.deriv;
        let deriv = num / (rhs.value.clone() * rhs.value);
        Dual { value, deriv }
    }
}

impl<S: Scalar> Neg for Dual<S> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Dual {
            value: -self.value,
            deriv: -self.deriv,
        }
    }
}

impl<S: Scalar> Scalar for Dual<S> {
    #[inline]
    fn zero() -> Self {
        Dual {
            value: S::zero(),
            deriv: S::zero(),
        }
    }

    #[inline]
    fn one() -> Self {
        // A literal one is constant → derivative 0.
        Dual {
            value: S::one(),
            deriv: S::zero(),
        }
    }

    #[inline]
    fn from_i32(i: i32) -> Self {
        Dual {
            value: S::from_i32(i),
            deriv: S::zero(),
        }
    }

    #[inline]
    fn from_f64(x: f64) -> Self {
        Dual {
            value: S::from_f64(x),
            deriv: S::zero(),
        }
    }

    /// Documented-lossy readout of the **value** (standard part) only; the
    /// derivative is dropped. For display / seeding, never for a certified
    /// decision — matching the base [`Scalar::to_f64`] contract.
    #[inline]
    fn to_f64(&self) -> f64 {
        self.value.to_f64()
    }

    /// `|value|` with the derivative `sign(value)·deriv`, under the convention
    /// `sign(0) := +1`.
    ///
    /// Non-smooth at `value == 0`, where the returned derivative is `deriv`
    /// unchanged. That is a valid Clarke generalized derivative but it is **not**
    /// the right derivative of `|u(t)|`: for `u(t) = −t` at `t = 0` this returns
    /// `−1`, whereas the right derivative is `+1`. Callers that need a one-sided
    /// derivative at a kink must supply the side themselves — the dual number
    /// does not carry it.
    #[inline]
    fn abs(&self) -> Self {
        if self.value >= S::zero() {
            self.clone()
        } else {
            -self.clone()
        }
    }
}

impl<S: crate::FallibleScalar> crate::FallibleScalar for Dual<S> {
    fn try_from_f64(x: f64) -> Option<Self> {
        S::try_from_f64(x).map(Dual::constant)
    }
}

/// Chain rule for the non-exact-closed operations, on the **floating tier only**
/// (`Dual<S>` is [`ApproxScalar`] exactly when `S` is — so `Dual<Real>` is not,
/// preserving the exact boundary of the seam).
impl<S: ApproxScalar> ApproxScalar for Dual<S> {
    #[inline]
    fn sqrt(&self) -> Self {
        // d/dx √u = u' / (2√u)
        let value = self.value.sqrt();
        let two = S::from_i32(2);
        let deriv = self.deriv.clone() / (two * value.clone());
        Dual { value, deriv }
    }

    #[inline]
    fn exp(&self) -> Self {
        // d/dx e^u = u'·e^u
        let value = self.value.exp();
        let deriv = self.deriv.clone() * value.clone();
        Dual { value, deriv }
    }

    #[inline]
    fn ln(&self) -> Self {
        // d/dx ln u = u' / u
        let value = self.value.ln();
        let deriv = self.deriv.clone() / self.value.clone();
        Dual { value, deriv }
    }

    #[inline]
    fn sin(&self) -> Self {
        // d/dx sin u = u'·cos u
        let value = self.value.sin();
        let deriv = self.deriv.clone() * self.value.cos();
        Dual { value, deriv }
    }

    #[inline]
    fn cos(&self) -> Self {
        // d/dx cos u = −u'·sin u
        let value = self.value.cos();
        let deriv = -(self.deriv.clone() * self.value.sin());
        Dual { value, deriv }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type D = Dual<f64>;

    #[test]
    fn constant_and_variable_seeds() {
        assert_eq!(D::constant(3.0).deriv(), 0.0);
        assert_eq!(D::variable(3.0).deriv(), 1.0);
        assert_eq!(<D as Scalar>::one().deriv(), 0.0);
        assert_eq!(<D as Scalar>::from_i32(7).deriv(), 0.0);
    }

    #[test]
    fn product_rule() {
        // d/dx [x · x] = 2x  at x = 5  → 10
        let x = D::variable(5.0);
        let y = x.clone() * x;
        assert_eq!(y.value(), 25.0);
        assert_eq!(y.deriv(), 10.0);
    }

    #[test]
    fn quotient_rule() {
        // f(x) = (2x) / (x + 1); f'(x) = 2 / (x+1)^2. At x = 3 → 2/16 = 0.125
        let x = D::variable(3.0);
        let num = D::from_i32(2) * x.clone();
        let den = x + D::one();
        let f = num / den;
        assert!((f.value() - 6.0 / 4.0).abs() < 1e-12);
        assert!((f.deriv() - 0.125).abs() < 1e-12);
    }

    #[test]
    fn abs_derivative_tracks_sign() {
        // |x| at x = -2 → value 2, derivative -1
        let neg = Scalar::abs(&D::variable(-2.0));
        assert_eq!(neg.value(), 2.0);
        assert_eq!(neg.deriv(), -1.0);
        // at x = +2 → value 2, derivative +1
        let pos = Scalar::abs(&D::variable(2.0));
        assert_eq!(pos.value(), 2.0);
        assert_eq!(pos.deriv(), 1.0);
    }

    #[test]
    fn comparison_uses_standard_part_only() {
        // Same value, different derivative → equal and non-ordered.
        let a = D::new(2.0, 1.0);
        let b = D::new(2.0, 99.0);
        assert_eq!(a, b);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
        // Ordering follows the value, ignoring the tangent.
        let big = D::new(3.0, -100.0);
        assert!(big > a);
    }

    #[test]
    fn chain_rule_through_transcendentals() {
        // h(x) = sin(x²);  h'(x) = 2x·cos(x²).  At x = 0.7.
        let x0 = 0.7_f64;
        let h = ApproxScalar::sin(&D::variable(x0).squared());
        let expected = 2.0 * x0 * (x0 * x0).cos();
        assert!((h.value() - (x0 * x0).sin()).abs() < 1e-12);
        assert!((h.deriv() - expected).abs() < 1e-12);
    }

    #[test]
    fn chain_rule_sqrt_exp_ln() {
        // d/dx √x = 1/(2√x); at x=4 → 0.25
        let s = ApproxScalar::sqrt(&D::variable(4.0));
        assert!((s.deriv() - 0.25).abs() < 1e-12);
        // d/dx e^x = e^x; at x=1 → e
        let e = ApproxScalar::exp(&D::variable(1.0));
        assert!((e.deriv() - std::f64::consts::E).abs() < 1e-12);
        // d/dx ln x = 1/x; at x=2 → 0.5
        let l = ApproxScalar::ln(&D::variable(2.0));
        assert!((l.deriv() - 0.5).abs() < 1e-12);
    }

    /// A `Dual<f64>` flows through a kernel bounded only on `S: Scalar`.
    fn poly<S: Scalar>(x: S) -> S {
        // p(x) = x³ − x;  p'(x) = 3x² − 1
        x.squared() * x.clone() - x
    }

    #[test]
    fn generic_kernel_over_dual() {
        let y = poly(D::variable(2.0));
        assert_eq!(y.value(), 6.0); // 8 - 2
        assert_eq!(y.deriv(), 11.0); // 3·4 - 1
    }
}
