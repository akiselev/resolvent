//! Rung 2: double-double with a rigorous running error bound.

use super::{MARGIN_DOWN, MAX_MAGNITUDE, MAX_PRODUCT_OPERAND, RungArith, round_up_bound, within};
use crate::eft::two_sum;
use crate::exact::RingOps;
use crate::expansion::two_prod_dekker;
use crate::uncertain::{Sign, USign, Uncertain};

/// `2^-106`, the square of the `f64` unit roundoff.
pub(super) const U2: f64 = 1.232_595_164_407_831e-32;
/// Relative error constant of double-double addition, times `U2`. The accurate
/// algorithm used here is below `3 U2 / (1 - 4 u)` of the result; `8` leaves
/// room for the bound's own arithmetic.
const ADD_ERR: f64 = 8.0 * U2;
/// Relative error constant of double-double multiplication, times `U2`
/// (the algorithm is below `7 U2`; the dropped `lo*lo` term is included).
const MUL_ERR: f64 = 16.0 * U2;

#[inline]
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

/// A double-double `hi + lo` (a normalized unevaluated sum of two `f64`) with
/// an absolute error bound `e`: `|hi + lo - exact| <= e`, `exact` as in
/// [`RunningError`](super::RunningError).
///
/// Unlike rung 1 the bound is a priori (a fixed multiple of `2^-106` times the
/// operand magnitudes), so it is positive for almost every operation and a
/// zero is certified only when the whole computation is exactly zero. Exact
/// zeros reached by cancellation belong to the expansion rung. The same guard
/// thresholds as rung 1 apply; values outside them are poisoned.
#[derive(Clone, Copy, Debug)]
pub struct DdBound {
    hi: f64,
    lo: f64,
    e: f64,
}

impl DdBound {
    const POISON: DdBound = DdBound {
        hi: f64::NAN,
        lo: 0.0,
        e: f64::INFINITY,
    };

    /// An input value with zero error. Poisoned if non-finite or beyond
    /// [`MAX_MAGNITUDE`](super::MAX_MAGNITUDE).
    pub fn exact(x: f64) -> Self {
        if x.abs() <= MAX_MAGNITUDE {
            DdBound {
                hi: x,
                lo: 0.0,
                e: 0.0,
            }
        } else {
            Self::POISON
        }
    }

    /// The high word.
    pub fn hi(&self) -> f64 {
        self.hi
    }

    /// The low word.
    pub fn lo(&self) -> f64 {
        self.lo
    }

    /// The absolute error bound (`+inf` when poisoned or unbounded).
    pub fn error(&self) -> f64 {
        self.e
    }

    fn is_sound(&self) -> bool {
        self.e < f64::INFINITY && self.hi.abs() <= MAX_MAGNITUDE && self.lo.abs() <= MAX_MAGNITUDE
    }

    /// `|hi| + |lo|`, an upper bound (before rounding) of the magnitude.
    fn magnitude(&self) -> f64 {
        self.hi.abs() + self.lo.abs()
    }

    /// The certified sign, or [`Uncertain::Unknown`].
    pub fn sign(&self) -> USign {
        self.certified()
            .map_or(Uncertain::Unknown, |(s, _)| Uncertain::Certain(s))
    }

    /// A certified lower bound on `|exact|`: `0.0` unless the sign is
    /// certified, otherwise slightly below `|hi + lo| - e`.
    pub fn margin(&self) -> f64 {
        self.certified().map_or(0.0, |(_, m)| m)
    }

    fn certified(&self) -> Option<(Sign, f64)> {
        if !self.is_sound() {
            return None;
        }
        // Renormalize defensively: `h = fl(hi + lo)`, `l` the exact residual.
        let (h, l) = two_sum(self.hi, self.lo);
        if h == 0.0 {
            return (l == 0.0 && self.e == 0.0).then_some((Sign::Zero, 0.0));
        }
        // |l| <= ulp(h)/2, so |h + l| >= |h| (1 - 2^-53).
        let m = h.abs() * MARGIN_DOWN;
        if m > self.e {
            let sign = if h > 0.0 {
                Sign::Positive
            } else {
                Sign::Negative
            };
            Some((sign, ((m - self.e) * MARGIN_DOWN).max(0.0)))
        } else {
            None
        }
    }
}

impl RingOps for DdBound {
    fn zero() -> Self {
        DdBound {
            hi: 0.0,
            lo: 0.0,
            e: 0.0,
        }
    }

    fn from_i32(i: i32) -> Self {
        DdBound {
            hi: f64::from(i),
            lo: 0.0,
            e: 0.0,
        }
    }

    fn add(&self, rhs: &Self) -> Self {
        if !self.is_sound() || !rhs.is_sound() {
            return Self::POISON;
        }
        let (s1, s2) = two_sum(self.hi, rhs.hi);
        let (t1, t2) = two_sum(self.lo, rhs.lo);
        let (s1, s2) = quick_two_sum(s1, s2 + t1);
        let (hi, lo) = quick_two_sum(s1, s2 + t2);
        if !within(hi, MAX_MAGNITUDE) {
            return Self::POISON;
        }
        let scale = self.magnitude() + rhs.magnitude();
        DdBound {
            hi,
            lo,
            e: round_up_bound(self.e + rhs.e + ADD_ERR * scale, scale != 0.0),
        }
    }

    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.neg())
    }

    fn mul(&self, rhs: &Self) -> Self {
        if !self.is_sound() || !rhs.is_sound() {
            return Self::POISON;
        }
        let (a, b) = (self.magnitude(), rhs.magnitude());
        if a > MAX_PRODUCT_OPERAND || b > MAX_PRODUCT_OPERAND {
            return Self::POISON;
        }
        let Some((p, r)) = two_prod_dekker(self.hi, rhs.hi) else {
            return Self::POISON;
        };
        let (hi, lo) = quick_two_sum(p, r + (self.hi * rhs.lo + self.lo * rhs.hi));
        let e = a * rhs.e + b * self.e + self.e * rhs.e + MUL_ERR * a * b;
        let inexact = (a != 0.0 && b != 0.0) || self.e != 0.0 || rhs.e != 0.0;
        DdBound {
            hi,
            lo,
            e: round_up_bound(e, inexact),
        }
    }

    fn neg(&self) -> Self {
        DdBound {
            hi: -self.hi,
            lo: -self.lo,
            e: self.e,
        }
    }
}

impl RungArith for DdBound {
    fn lift(x: f64) -> Self {
        Self::exact(x)
    }
}
