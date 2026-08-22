//! Tri-state certified logic: the value-based replacement for CGAL's
//! `Uncertain<T>` + throwing conversion (DESIGN.md §3.2).

use core::cmp::Ordering;

/// Sign of an exact quantity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    /// Strictly negative.
    Negative,
    /// Exactly zero.
    Zero,
    /// Strictly positive.
    Positive,
}

impl Sign {
    /// Sign of a finite `f64` (panics on NaN — signs of NaN are meaningless).
    pub fn of_f64(x: f64) -> Sign {
        Self::try_of_f64(x).expect("Sign::of_f64(NaN)")
    }

    /// Sign of an IEEE value, or `None` for NaN.
    pub fn try_of_f64(x: f64) -> Option<Sign> {
        if x.is_nan() {
            return None;
        }
        if x > 0.0 {
            Some(Sign::Positive)
        } else if x < 0.0 {
            Some(Sign::Negative)
        } else {
            Some(Sign::Zero)
        }
    }

    /// Negation.
    #[must_use]
    pub fn negated(self) -> Sign {
        match self {
            Sign::Negative => Sign::Positive,
            Sign::Zero => Sign::Zero,
            Sign::Positive => Sign::Negative,
        }
    }

    /// Sign of a product.
    #[must_use]
    pub fn times(self, rhs: Sign) -> Sign {
        match (self, rhs) {
            (Sign::Zero, _) | (_, Sign::Zero) => Sign::Zero,
            (a, b) if a == b => Sign::Positive,
            _ => Sign::Negative,
        }
    }

    /// The corresponding `Ordering` against zero.
    pub fn as_ordering(self) -> Ordering {
        match self {
            Sign::Negative => Ordering::Less,
            Sign::Zero => Ordering::Equal,
            Sign::Positive => Ordering::Greater,
        }
    }
}

/// A value that is either certainly known or undetermined at the current
/// precision. Filter rungs return `Uncertain<T>`; exact rungs return `T`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Uncertain<T> {
    /// The value is certified.
    Certain(T),
    /// The filter could not decide; escalate to a more precise rung.
    Unknown,
}

/// Certified sign.
pub type USign = Uncertain<Sign>;
/// Certified boolean.
pub type UBool = Uncertain<bool>;
/// Certified ordering.
pub type UOrd = Uncertain<Ordering>;

impl<T> Uncertain<T> {
    /// `true` iff certified.
    pub fn is_certain(&self) -> bool {
        matches!(self, Uncertain::Certain(_))
    }

    /// Map the certified value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Uncertain<U> {
        match self {
            Uncertain::Certain(t) => Uncertain::Certain(f(t)),
            Uncertain::Unknown => Uncertain::Unknown,
        }
    }

    /// Monadic bind.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Uncertain<U>) -> Uncertain<U> {
        match self {
            Uncertain::Certain(t) => f(t),
            Uncertain::Unknown => Uncertain::Unknown,
        }
    }

    /// The certified value, or compute it with the (exact) fallback.
    pub fn certain_or(self, exact: impl FnOnce() -> T) -> T {
        match self {
            Uncertain::Certain(t) => t,
            Uncertain::Unknown => exact(),
        }
    }

    /// The certified value, if any.
    pub fn certain(self) -> Option<T> {
        match self {
            Uncertain::Certain(t) => Some(t),
            Uncertain::Unknown => None,
        }
    }
}

impl<T> From<T> for Uncertain<T> {
    fn from(t: T) -> Self {
        Uncertain::Certain(t)
    }
}
