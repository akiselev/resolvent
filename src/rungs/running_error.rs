//! Rung 1: `f64` with a rigorous running error bound.

use super::{MARGIN_DOWN, MAX_MAGNITUDE, MAX_PRODUCT_OPERAND, RungArith, round_up_bound, within};
use crate::eft::two_sum;
use crate::exact::RingOps;
use crate::expansion::two_prod_dekker;
use crate::uncertain::{Sign, USign, Uncertain};

/// An `f64` value `v` with an absolute error bound `e`, such that
/// `|v - exact| <= e` where `exact` is the exact real value of the same
/// expression evaluated over the exact binary values of the inputs.
///
/// Every `+` and `*` captures its own rounding error exactly (TwoSum, Dekker
/// TwoProduct) and adds its magnitude to the bound, so an expression whose
/// every operation is exact keeps `e == 0` and can certify an exact zero:
/// axis-aligned, integer and clustered-dyadic data.
///
/// The bound covers rounding in the arithmetic of the expression, including
/// the rounding of the bound itself (see the module docs of
/// [`crate::rungs`]). It does not cover errors in the inputs: those are
/// whatever `f64` the caller supplied. Values outside the guard thresholds are
/// poisoned (`e == +inf`) and never certify a sign.
#[derive(Clone, Copy, Debug)]
pub struct RunningError {
    v: f64,
    e: f64,
}

impl RunningError {
    const POISON: RunningError = RunningError {
        v: f64::NAN,
        e: f64::INFINITY,
    };

    /// An input value with zero error. Poisoned if non-finite or beyond
    /// [`MAX_MAGNITUDE`](super::MAX_MAGNITUDE).
    pub fn exact(x: f64) -> Self {
        if x.abs() <= MAX_MAGNITUDE {
            RunningError { v: x, e: 0.0 }
        } else {
            Self::POISON
        }
    }

    /// The computed value.
    pub fn value(&self) -> f64 {
        self.v
    }

    /// The absolute error bound (`+inf` when poisoned or unbounded).
    pub fn error(&self) -> f64 {
        self.e
    }

    /// `true` when the value and bound are usable.
    fn is_sound(&self) -> bool {
        self.e < f64::INFINITY && self.v.abs() <= MAX_MAGNITUDE
    }

    /// The certified sign, or [`Uncertain::Unknown`] when `|v| <= e` (unless
    /// both are zero: an exactly zero value with zero error is an exact zero).
    pub fn sign(&self) -> USign {
        if !self.is_sound() {
            Uncertain::Unknown
        } else if self.v > self.e {
            Uncertain::Certain(Sign::Positive)
        } else if self.v < -self.e {
            Uncertain::Certain(Sign::Negative)
        } else if self.v == 0.0 && self.e == 0.0 {
            Uncertain::Certain(Sign::Zero)
        } else {
            Uncertain::Unknown
        }
    }

    /// A certified lower bound on `|exact|`: `0.0` unless the sign is
    /// certified, otherwise slightly below `|v| - e`.
    pub fn margin(&self) -> f64 {
        match self.sign() {
            Uncertain::Certain(Sign::Zero) | Uncertain::Unknown => 0.0,
            Uncertain::Certain(_) => ((self.v.abs() - self.e) * MARGIN_DOWN).max(0.0),
        }
    }
}

impl RingOps for RunningError {
    fn zero() -> Self {
        RunningError { v: 0.0, e: 0.0 }
    }

    fn from_i32(i: i32) -> Self {
        RunningError {
            v: f64::from(i),
            e: 0.0,
        }
    }

    fn add(&self, rhs: &Self) -> Self {
        if !self.is_sound() || !rhs.is_sound() {
            return Self::POISON;
        }
        let (s, r) = two_sum(self.v, rhs.v);
        if !within(s, MAX_MAGNITUDE) {
            return Self::POISON;
        }
        RunningError {
            v: s,
            e: round_up_bound(self.e + rhs.e + r.abs(), false),
        }
    }

    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.neg())
    }

    fn mul(&self, rhs: &Self) -> Self {
        if !self.is_sound()
            || !rhs.is_sound()
            || self.v.abs() > MAX_PRODUCT_OPERAND
            || rhs.v.abs() > MAX_PRODUCT_OPERAND
        {
            return Self::POISON;
        }
        let Some((p, r)) = two_prod_dekker(self.v, rhs.v) else {
            return Self::POISON;
        };
        // exact(a)*exact(b) = v_a*v_b + v_a*d_b + v_b*d_a + d_a*d_b with
        // |d| <= e, and v_a*v_b = p + r exactly.
        if self.e == 0.0 && rhs.e == 0.0 {
            return RunningError { v: p, e: r.abs() };
        }
        let e = self.v.abs() * rhs.e + rhs.v.abs() * self.e + self.e * rhs.e + r.abs();
        RunningError {
            v: p,
            e: round_up_bound(e, true),
        }
    }

    fn neg(&self) -> Self {
        RunningError {
            v: -self.v,
            e: self.e,
        }
    }
}

impl RungArith for RunningError {
    fn lift(x: f64) -> Self {
        Self::exact(x)
    }
}
