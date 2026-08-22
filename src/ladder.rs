//! The certify-or-escalate ladder (DESIGN.md §3.2, §3.5): try the interval
//! rung, escalate to exact only on `Unknown`.

use crate::exact::{ExactRing, Rational, RingOps};
use crate::interval::Interval;
use crate::uncertain::{Sign, Uncertain};

/// Run the filter; escalate to the exact fallback only when it cannot
/// certify. The whole predicate architecture in one function.
pub fn certify<T>(filter: impl FnOnce() -> Uncertain<T>, exact: impl FnOnce() -> T) -> T {
    match filter() {
        Uncertain::Certain(t) => t,
        Uncertain::Unknown => exact(),
    }
}

/// Generic 2×2 determinant over any [`RingOps`] implementor — the "one
/// formula text, several instantiations" pattern.
pub fn det2<T: RingOps>(a: &T, b: &T, c: &T, d: &T) -> T {
    a.mul(d).sub(&b.mul(c))
}

/// Generic 3×3 determinant.
#[allow(clippy::too_many_arguments)]
pub fn det3<T: RingOps>(
    m00: &T,
    m01: &T,
    m02: &T,
    m10: &T,
    m11: &T,
    m12: &T,
    m20: &T,
    m21: &T,
    m22: &T,
) -> T {
    let c0 = det2(m11, m12, m21, m22);
    let c1 = det2(m10, m12, m20, m22);
    let c2 = det2(m10, m11, m20, m21);
    m00.mul(&c0).sub(&m01.mul(&c1)).add(&m02.mul(&c2))
}

/// Filtered exact sign of a 2×2 determinant of doubles: interval rung, then
/// exact rationals. Inputs must be finite.
///
/// # Panics
/// **API-contract panic** on a non-finite input, the same contract
/// [`Interval::point`] states — and the interval rung reaches `point` first,
/// so a non-finite argument has already panicked before the exact rung's
/// `expect` is evaluated. Sign of a determinant of `±∞`/NaN is not a
/// question this crate answers.
#[allow(clippy::expect_used)] // documented contract: inputs must be finite
pub fn sign_of_det2_f64(a: f64, b: f64, c: f64, d: f64) -> Sign {
    try_sign_of_det2_f64(a, b, c, d).expect("determinant inputs must be finite")
}

/// Total form of [`sign_of_det2_f64`]; `None` if any input is non-finite.
#[allow(clippy::expect_used)] // finiteness is checked before exact conversion
pub fn try_sign_of_det2_f64(a: f64, b: f64, c: f64, d: f64) -> Option<Sign> {
    if ![a, b, c, d].into_iter().all(f64::is_finite) {
        return None;
    }
    let fi = || {
        det2(
            &Interval::point(a),
            &Interval::point(b),
            &Interval::point(c),
            &Interval::point(d),
        )
        .sign()
    };
    let fe = || {
        let q = |x: f64| Rational::from_f64(x).expect("finite input");
        det2(&q(a), &q(b), &q(c), &q(d)).sign()
    };
    Some(certify(fi, fe))
}

/// Filtered exact sign of a 3×3 determinant of doubles.
///
/// # Panics
/// Same **API-contract panic** as [`sign_of_det2_f64`]: inputs must be
/// finite, and the interval rung's [`Interval::point`] enforces it first.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::expect_used)] // documented contract: inputs must be finite
pub fn sign_of_det3_f64(
    m00: f64,
    m01: f64,
    m02: f64,
    m10: f64,
    m11: f64,
    m12: f64,
    m20: f64,
    m21: f64,
    m22: f64,
) -> Sign {
    try_sign_of_det3_f64([m00, m01, m02, m10, m11, m12, m20, m21, m22])
        .expect("determinant inputs must be finite")
}

/// Total filtered exact sign of a 3x3 determinant in row-major order.
#[allow(clippy::expect_used)] // finiteness is checked before exact conversion
pub fn try_sign_of_det3_f64(values: [f64; 9]) -> Option<Sign> {
    if !values.into_iter().all(f64::is_finite) {
        return None;
    }
    let [m00, m01, m02, m10, m11, m12, m20, m21, m22] = values;
    let fi = || {
        let p = Interval::point;
        det3(
            &p(m00),
            &p(m01),
            &p(m02),
            &p(m10),
            &p(m11),
            &p(m12),
            &p(m20),
            &p(m21),
            &p(m22),
        )
        .sign()
    };
    let fe = || {
        let q = |x: f64| Rational::from_f64(x).expect("finite input");
        det3(
            &q(m00),
            &q(m01),
            &q(m02),
            &q(m10),
            &q(m11),
            &q(m12),
            &q(m20),
            &q(m21),
            &q(m22),
        )
        .sign()
    };
    Some(certify(fi, fe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn det2_degenerate_exact_zero() {
        // Collinear-style degenerate case: ad == bc exactly.
        assert_eq!(sign_of_det2_f64(2.0, 6.0, 1.0, 3.0), Sign::Zero);
    }

    #[test]
    fn det2_near_degenerate_signs() {
        // ad and bc differ by one ulp — the filter cannot decide; the exact
        // rung must.
        let a = 1.0 + f64::EPSILON;
        assert_eq!(sign_of_det2_f64(a, 1.0, 1.0, 1.0), Sign::Positive);
        assert_eq!(sign_of_det2_f64(1.0, a, 1.0, 1.0), Sign::Negative);
    }

    #[test]
    fn det3_zero_row_dependence() {
        // Row 3 = row 1 + row 2 exactly.
        assert_eq!(
            sign_of_det3_f64(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 5.0, 7.0, 9.0),
            Sign::Zero
        );
    }
}
