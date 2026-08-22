//! The **`Scalar` seam** — write numeric code once, then instantiate it on the
//! fast `f64` tier or a certified-exact real from the same generic codebase.
//!
//! # Ownership
//!
//! Resolvent owns both this consumer-neutral scalar vocabulary and its exact
//! implementations. Scientific meaning remains in Scientia and geometry
//! policy remains in CADabra; those consumers depend downward on this crate
//! instead of carrying parallel scalar or exact-algebra packages.
//!
//! # What exact arithmetic can and cannot provide
//!
//! [`Scalar`] is deliberately the **exact-closed** field surface: `+ − × ÷`,
//! order, [`abs`](Scalar::abs), exact integer/rational ingress. Every operation
//! here maps a rational to a rational, so a `Real` built from these stays
//! certifiable. Operations that are **not** exact-closed over ℚ — `√` mints an
//! algebraic irrational, and `sin`/`cos`/`exp`/`ln` leave the algebraic numbers
//! entirely — live on the separate [`ApproxScalar`] extension, implemented for
//! the floating tier ONLY. The certified-exact real intentionally does *not*
//! implement [`ApproxScalar`]: its exact story for `√` is an algebraic-extension
//! coordinate type (`resolvent::SqrtExt` and friends), a documented follow-on,
//! not a silent lossy fallback.
//!
//! # The differentiability rung — [`Dual`] is itself a [`Scalar`]
//!
//! [`Dual<S>`](dual::Dual) is a forward-mode differentiable number carrying
//! `(value, deriv)` with chain-rule arithmetic that **itself implements
//! [`Scalar`]**. So the *same* generic kernel instantiates at `Dual<f64>` (fast
//! forward-mode AD) and `Dual<Real>` (**exact** derivatives — certified equal to
//! the analytic sensitivity, no finite-difference noise). This is what makes
//! "exact", "fast", and "differentiable" three instantiations of one codebase.
//! See the [`dual`] module for the design and the reverse-mode-over-DAG
//! follow-on.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::ops::{Add, Div, Mul, Neg, Sub};

/// The exact-closed scalar surface numeric code is written against.
///
/// Bounds are the by-value field operations plus order and identity — exactly
/// what generic assembly, dense elimination, and residual/norm computations
/// call. Implemented for [`f64`] (here) and for `resolvent::Real<E>` (in
/// `resolvent`).
///
/// `Send + Sync + 'static` are required so a `Scalar` can flow through parallel
/// assembly and be stored in the ecosystem's operator/solver trait objects;
/// this matches the ecosystem's existing `numeric_contracts::Scalar` bound set,
/// so the seam can become its supertrait during adoption.
///
/// **Not `Copy`.** `f64` is `Copy`, but the certified-exact real is a
/// reference-counted DAG handle (`Clone`, not `Copy`). Generic code therefore
/// clones operands explicitly; that is the one ergonomic cost of scalar
/// genericity, and it compiles to a trivial register move on the `f64` tier.
pub trait Scalar:
    Clone
    + PartialEq
    + PartialOrd
    + Send
    + Sync
    + 'static
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// The additive identity.
    fn zero() -> Self;

    /// The multiplicative identity.
    fn one() -> Self;

    /// Exact embedding of a small integer constant. Exact in every
    /// implementor (`i32 ⊂ f64` losslessly; exact in the rational rung).
    fn from_i32(i: i32) -> Self;

    /// Lift a **finite** double (a mesh coordinate, a material constant).
    ///
    /// Contract for generic callers: `x` is finite. The `f64` implementation
    /// is the IEEE identity and therefore preserves NaN/±∞; exact `Real`
    /// implementations panic because those values have no embedding. Use
    /// [`FallibleScalar::try_from_f64`] at untrusted generic ingress.
    fn from_f64(x: f64) -> Self;

    /// A documented-**lossy** `f64` readout: the value itself on `f64`, the
    /// midpoint of the current enclosure on the exact rung. For display,
    /// seeding an initial guess, or comparing tiers — never for a decision that
    /// must be certified (use the scalar's own comparison for that).
    fn to_f64(&self) -> f64;

    /// Absolute value. Exact-closed (`|q|` is rational for rational `q`).
    fn abs(&self) -> Self;

    /// Exact rational constant `num/den`. Exact on the rational rung (exact
    /// integer ÷ exact integer), nearest double on `f64`. `den` must be nonzero.
    ///
    /// Provided via [`from_i32`](Scalar::from_i32) and `÷`, so it is
    /// automatically exact wherever `÷` is.
    fn from_ratio(num: i32, den: i32) -> Self {
        assert!(den != 0, "Scalar::from_ratio: zero denominator");
        Self::from_i32(num) / Self::from_i32(den)
    }

    /// `self * self`.
    fn squared(&self) -> Self {
        self.clone() * self.clone()
    }

    /// Certified test against the additive identity (forces exactness on the
    /// exact rung — an exact-zero query, correctly decided).
    fn is_zero(&self) -> bool {
        self.eq(&Self::zero())
    }
}

/// Total generic scalar ingress for data whose validity is not proven by the
/// caller's type.
pub trait FallibleScalar: Scalar {
    /// Lift a finite IEEE value, or `None` for NaN/±∞.
    fn try_from_f64(x: f64) -> Option<Self>;

    /// Lift a rational constant, or `None` for a zero denominator.
    fn try_from_ratio(num: i32, den: i32) -> Option<Self> {
        (den != 0).then(|| Self::from_i32(num) / Self::from_i32(den))
    }
}

/// Operations that are **not** exact-closed over ℚ: `√` mints an algebraic
/// irrational; the transcendentals leave the algebraic numbers entirely.
///
/// Implemented for the floating tier (`f64`) only. The certified-exact real
/// deliberately does **not** implement this trait — see the crate docs. Numeric
/// code that needs these operations is, by that fact, float-tier code; bounding
/// on `ApproxScalar` instead of [`Scalar`] makes that requirement explicit at
/// the type level.
pub trait ApproxScalar: Scalar {
    /// Square root. Precondition: `self >= 0`.
    fn sqrt(&self) -> Self;
    /// Natural exponential.
    fn exp(&self) -> Self;
    /// Natural logarithm. Precondition: `self > 0`.
    fn ln(&self) -> Self;
    /// Sine (radians).
    fn sin(&self) -> Self;
    /// Cosine (radians).
    fn cos(&self) -> Self;
}

impl Scalar for f64 {
    #[inline]
    fn zero() -> Self {
        0.0
    }
    #[inline]
    fn one() -> Self {
        1.0
    }
    #[inline]
    fn from_i32(i: i32) -> Self {
        f64::from(i)
    }
    #[inline]
    fn from_f64(x: f64) -> Self {
        x
    }
    #[inline]
    fn to_f64(&self) -> f64 {
        *self
    }
    #[inline]
    fn abs(&self) -> Self {
        f64::abs(*self)
    }
}

impl FallibleScalar for f64 {
    fn try_from_f64(x: f64) -> Option<Self> {
        x.is_finite().then_some(x)
    }
}

impl ApproxScalar for f64 {
    #[inline]
    fn sqrt(&self) -> Self {
        f64::sqrt(*self)
    }
    #[inline]
    fn exp(&self) -> Self {
        f64::exp(*self)
    }
    #[inline]
    fn ln(&self) -> Self {
        f64::ln(*self)
    }
    #[inline]
    fn sin(&self) -> Self {
        f64::sin(*self)
    }
    #[inline]
    fn cos(&self) -> Self {
        f64::cos(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_field_surface() {
        assert_eq!(<f64 as Scalar>::zero(), 0.0);
        assert_eq!(<f64 as Scalar>::one(), 1.0);
        assert_eq!(f64::from_i32(-3), -3.0);
        assert_eq!(f64::from_f64(2.5), 2.5);
        assert_eq!(Scalar::abs(&-4.0f64), 4.0);
        assert_eq!((2.0f64).squared(), 4.0);
        assert!(<f64 as Scalar>::is_zero(&0.0));
        assert!(!<f64 as Scalar>::is_zero(&1e-300));
    }

    #[test]
    fn f64_from_ratio_rounds() {
        // 1/9 is not representable in binary floating point.
        let third = <f64 as Scalar>::from_ratio(1, 9);
        assert!((third - 1.0 / 9.0).abs() < 1e-16);
    }

    #[test]
    fn f64_approx_tier() {
        assert!((ApproxScalar::sqrt(&2.0f64) - std::f64::consts::SQRT_2).abs() < 1e-15);
        assert!((ApproxScalar::exp(&0.0f64) - 1.0).abs() < 1e-15);
    }

    /// A generic function bounded on the seam compiles and runs on `f64`.
    fn dot<S: Scalar>(a: &[S], b: &[S]) -> S {
        let mut acc = S::zero();
        for (x, y) in a.iter().zip(b) {
            acc = acc + x.clone() * y.clone();
        }
        acc
    }

    #[test]
    fn generic_over_f64() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        assert_eq!(dot(&a, &b), 32.0);
    }
}
