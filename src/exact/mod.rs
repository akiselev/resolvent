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
