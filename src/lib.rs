//! Consumer-neutral exact algebra shared by Scientia and CADabra.

#![forbid(unsafe_code)]

mod error;
mod expr;
mod polynomial;
mod receipt;

pub use error::{AlgebraBudget, AlgebraError};
pub use expr::{Expr, Sign};
pub use num_bigint::BigInt;
pub use num_rational::BigRational as Rational;
pub use polynomial::{Polynomial, RootInterval};
pub use receipt::{AlgebraOperation, AlgebraReceipt};

/// Admit a finite IEEE-754 value as its exact binary rational value.
pub fn rational_from_f64(value: f64) -> Option<Rational> {
    Rational::from_float(value)
}

/// Convert an exact rational to `f64` when the finite conversion is available.
pub fn rational_to_f64(value: &Rational) -> Option<f64> {
    use num_traits::ToPrimitive;
    value.to_f64().filter(|value| value.is_finite())
}
