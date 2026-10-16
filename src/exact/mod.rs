//! Exact backends and the traits that bind the tower together
//! (DESIGN.md §3.3).

mod rational;

pub use rational::Rational;

use crate::interval::Interval;
use crate::uncertain::Sign;

/// The operation surface formulas are written against, **by reference** so
/// generic formula bodies never deep-copy operands (DESIGN.md §3.5). This is
/// an ops-surface trait, not an algebraic claim — `Interval` implements it
/// too (interval arithmetic is not a ring; the shared surface is what lets
/// one formula body serve both the filter and the exact rung).
pub trait RingOps: Sized {
    /// Additive identity.
    fn zero() -> Self;
    /// Embed a small integer constant (exact in every implementor).
    fn from_i32(i: i32) -> Self;
    /// `self + rhs`.
    fn add(&self, rhs: &Self) -> Self;
    /// `self - rhs`.
    fn sub(&self, rhs: &Self) -> Self;
    /// `self * rhs`.
    fn mul(&self, rhs: &Self) -> Self;
    /// `-self`.
    fn neg(&self) -> Self;
}

impl RingOps for Interval {
    fn zero() -> Self {
        Interval::point(0.0)
    }
    fn from_i32(i: i32) -> Self {
        Interval::from(i)
    }
    fn add(&self, rhs: &Self) -> Self {
        *self + *rhs
    }
    fn sub(&self, rhs: &Self) -> Self {
        *self - *rhs
    }
    fn mul(&self, rhs: &Self) -> Self {
        *self * *rhs
    }
    fn neg(&self) -> Self {
        -*self
    }
}

/// The plain `f64` ring: every operation rounds to nearest, nothing is
/// bounded and nothing is certified.
///
/// This exists so one generic expression (a vector algebra, a predicate text)
/// can run at `f64` as well as at a certified rung, with the operations and
/// their order unchanged. A sign computed at `f64` alone is an uncertified
/// proposal; certification is the job of the `rungs` types and the exact
/// rings. `mul_add` is never used, so results do not depend on FMA hardware.
impl RingOps for f64 {
    fn zero() -> Self {
        0.0
    }
    fn from_i32(i: i32) -> Self {
        f64::from(i)
    }
    fn add(&self, rhs: &Self) -> Self {
        *self + *rhs
    }
    fn sub(&self, rhs: &Self) -> Self {
        *self - *rhs
    }
    fn mul(&self, rhs: &Self) -> Self {
        *self * *rhs
    }
    fn neg(&self) -> Self {
        -*self
    }
}

/// An exact ring: enough for polynomial predicates.
///
/// Float ingress is fallible — NaN/±∞ have no exact embedding — and
/// [`ExactRing::to_interval`] is THE bridge operation of the whole crate:
/// it must return an outward-correct double enclosure of the exact value
/// (subnormals and out-of-range magnitudes included).
pub trait ExactRing: RingOps + Clone + Ord {
    /// Exact embedding of a finite double; `None` for NaN/±∞.
    fn from_f64(x: f64) -> Option<Self>;
    /// Exact sign.
    fn sign(&self) -> Sign;
    /// Outward-correct double enclosure of the exact value.
    fn to_interval(&self) -> Interval;
}

/// An exact field: adds exact division (constructions divide).
pub trait ExactField: ExactRing {
    /// `self / rhs`. Panics if `rhs` is zero (exact zero is detectable —
    /// callers gate on [`ExactRing::sign`]).
    fn div(&self, rhs: &Self) -> Self;
}

#[cfg(test)]
mod f64_ring_tests {
    use super::RingOps;

    #[test]
    fn f64_ring_rounds_like_the_operators() {
        let (a, b) = (0.1_f64, 0.2_f64);
        assert_eq!(RingOps::add(&a, &b), a + b);
        assert_eq!(RingOps::sub(&a, &b), a - b);
        assert_eq!(RingOps::mul(&a, &b), a * b);
        assert_eq!(RingOps::neg(&a), -a);
        assert_eq!(<f64 as RingOps>::zero(), 0.0);
        assert_eq!(<f64 as RingOps>::from_i32(-7), -7.0);
    }
}
