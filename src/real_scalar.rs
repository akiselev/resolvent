//! The exact rung of the [`Scalar`] seam: `impl Scalar for Real<E>`.
//!
//! [`Scalar`] (defined in the zero-dependency `resolvent` leaf crate) is the
//! surface numeric code is written against; here the certified-exact
//! [`Real<E>`] satisfies it, so one generic kernel runs on `f64` OR `Real`.
//!
//! Note the deliberate omission: `Real` does **not** implement
//! [`resolvent::ApproxScalar`]. `√` and the transcendentals are not
//! exact-closed over ℚ (a rational's square root is generally irrational), so
//! offering them on the exact rung would silently drop certification. The exact
//! story for `√` is a one-root algebraic-extension coordinate ([`SqrtExt`]), a
//! documented follow-on — not a `Scalar` method.
//!
//! [`SqrtExt`]: crate::SqrtExt

use crate::exact::{ExactField, RingOps};
use crate::real::Real;
use crate::{FallibleScalar, Scalar};

impl<E: ExactField + Send + Sync + 'static> Scalar for Real<E> {
    fn zero() -> Self {
        Real::from_exact(<E as RingOps>::zero())
    }

    fn one() -> Self {
        Real::from_exact(E::from_i32(1))
    }

    fn from_i32(i: i32) -> Self {
        Real::from_exact(E::from_i32(i))
    }

    // **API-contract panic**, and one the seam forces: `Scalar::from_f64`'s
    // signature is `fn(f64) -> Self`, so there is no `Option` to return and
    // no error to propagate. `resolvent::Scalar::from_f64` states the
    // contract ("Contract: `x` is finite … on the exact rung this panics"),
    // and `Real::from_f64` is the total form for callers that cannot promise
    // it. Widening the trait to `Option<Self>` is a `resolvent` design
    // change (PLAN.md §2.2), not something to paper over here.
    #[allow(clippy::expect_used)] // documented seam contract; `Real::from_f64` is total
    fn from_f64(x: f64) -> Self {
        Real::from_f64(x).expect(
            "Scalar::from_f64 on Real requires a finite double: \
             NaN/±∞ have no exact embedding (the fallible float-ingress boundary)",
        )
    }

    fn to_f64(&self) -> f64 {
        self.to_f64_lossy()
    }

    fn abs(&self) -> Self {
        Real::abs(self)
    }
}

impl<E: ExactField + Send + Sync + 'static> FallibleScalar for Real<E> {
    fn try_from_f64(x: f64) -> Option<Self> {
        Real::from_f64(x)
    }
}

#[cfg(test)]
mod tests {
    use crate::Scalar;
    use crate::exact::Rational;
    use crate::real::Real;
    use crate::uncertain::Sign;

    type R = Real<Rational>;

    #[test]
    fn identities_and_ingress() {
        assert_eq!(<R as Scalar>::zero().sign(), Sign::Zero);
        assert_eq!(<R as Scalar>::one().sign(), Sign::Positive);
        assert_eq!(R::from_i32(-5).sign(), Sign::Negative);
        assert!(<R as Scalar>::is_zero(&<R as Scalar>::zero()));
    }

    #[test]
    fn from_ratio_is_exact() {
        // 1/9 is not representable in f64, but Real carries it exactly.
        let ninth = <R as Scalar>::from_ratio(1, 9);
        let mut nine = ninth.clone();
        for _ in 0..8 {
            nine = nine + ninth.clone();
        }
        assert_eq!(nine, <R as Scalar>::one());
        // And it is NOT the rounded f64 1/9 re-embedded.
        let rounded = <R as Scalar>::from_f64(1.0f64 / 9.0);
        assert_ne!(ninth, rounded);
    }

    #[test]
    fn abs_is_exact_closed() {
        let neg = &<R as Scalar>::zero() - &<R as Scalar>::from_ratio(3, 7);
        let a = Scalar::abs(&neg);
        assert_eq!(a, <R as Scalar>::from_ratio(3, 7));
        assert_eq!(a.sign(), Sign::Positive);
    }
}
