//! Consumer-neutral symbolic, filtered, lazy, and exact algebra shared by
//! Scientia and CADabra.
//!
//! Predicates can run a fast interval filter and escalate to exact arithmetic
//! only when the enclosure is inconclusive. For example, a rounded `f64`
//! determinant may report collinearity even though the exact values are not
//! collinear:
//!
//! ```
//! use resolvent::exact::{ExactRing, RingOps};
//! use resolvent::{Rational, Sign};
//!
//! let (ax, ay) = (-3089173.0, 6656906.0);
//! let (bx, by) = (7841570.0, 4347616.0);
//! let (cx, cy) = (5406514.99152003, 4862059.362224572);
//! let f64_cross = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
//! assert_eq!(f64_cross, 0.0);
//!
//! let (ax, ay, bx, by, cx, cy) = (
//!     Rational::from_f64(ax).unwrap(),
//!     Rational::from_f64(ay).unwrap(),
//!     Rational::from_f64(bx).unwrap(),
//!     Rational::from_f64(by).unwrap(),
//!     Rational::from_f64(cx).unwrap(),
//!     Rational::from_f64(cy).unwrap(),
//! );
//! let exact_cross = bx.sub(&ax).mul(&cy.sub(&ay)).sub(&by.sub(&ay).mul(&cx.sub(&ax)));
//! assert_ne!(exact_cross.sign(), Sign::Zero);
//! ```

#![forbid(unsafe_code)]

pub mod bernstein;
mod dual;
pub mod eft;
mod error;
pub mod exact;
pub mod expansion;
mod expr;
pub mod interval;
pub mod ladder;
pub mod metrics;
pub mod polymat;
pub mod ratmat;
pub mod real;
mod real_scalar;
mod receipt;
pub mod roots;
mod scalar;
pub mod sqrt_ext;
pub mod uncertain;

pub use bernstein::Bernstein;
pub use dual::Dual;
pub use error::{AlgebraBudget, AlgebraError};
pub use exact::{ExactField, ExactRing, Rational, RingOps};
pub use expr::Expr;
pub use interval::{AtomicInterval, Interval};
pub use ladder::certify;
pub use polymat::PolyMat;
pub use ratmat::Mat;
pub use real::{Formula, Real};
pub use receipt::{AlgebraOperation, AlgebraReceipt};
pub use roots::{
    MAX_RATIONAL_COLLAPSE_STEPS, QPoly, RealRoot, RealRootCertificate, RootCertificateError,
    RootError, isolate_roots, isolate_roots_with_budget, sign_radical1, sign_radical2,
    simplest_rational_in, try_sign_radical1_at, try_sign_radical2_at,
};
pub use scalar::{ApproxScalar, Scalar};
pub use sqrt_ext::SqrtExt;
pub use uncertain::{Sign, UBool, UOrd, USign, Uncertain};

/// Admit a finite IEEE-754 value as its exact binary rational value.
pub fn rational_from_f64(value: f64) -> Option<Rational> {
    Rational::from_f64(value)
}

/// Convert an exact rational to `f64` when the finite conversion is available.
pub fn rational_to_f64(value: &Rational) -> Option<f64> {
    let value = value.to_f64_lossy();
    value.is_finite().then_some(value)
}
