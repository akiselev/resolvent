//! Shewchuk expansion arithmetic primitives (DESIGN.md §3.3).
//!
//! An *expansion* is a sum of non-overlapping doubles, largest last; the
//! primitives here are the exact building blocks for adaptive exact
//! predicate rungs. Only the primitives used by the shipped ladders are
//! implemented; the module grows with the predicates that need it.

use crate::eft::two_sum;
use crate::uncertain::Sign;

/// TwoProd: `Some((p, e))` with `p = fl(a·b)` and `p + e = a·b`
/// exactly, or `None` when that identity cannot hold.
///
/// Fail-closed on **underflow**, which the overflow guards elsewhere in this
/// module do not catch because underflow produces *finite* — merely wrong —
/// components. `two_prod(1e-200, 1e-200)` returns `(0.0, 0.0)`; both are
/// finite, so a downstream `is_finite` check passes and
/// [`sign_of_expansion`] then certifies `Zero` for a strictly positive
/// product. A prose-only precondition is not a guard.
///
/// The threshold is `|a·b| ≥ 2⁻¹⁰²² · 2⁵³ = 2⁻⁹⁶⁹`. Exactness needs the error
/// term itself to be representable: `e` carries bits down to
/// `2^(exp(p) − 106)`, so once `p` falls below `2⁻⁹⁶⁹` those bits drop off the
/// bottom of the subnormal range and `e` is silently rounded. A product that
/// is exactly zero because a *factor* is zero is exact and accepted.
///
/// With the `fma` target feature the error term is one hardware FMA. Without
/// it, `mul_add` is a libm call, so the error term comes from Dekker's
/// product over Veltkamp splits instead ([`two_prod_dekker`]); both give the
/// same `(p, e)` wherever this returns `Some`.
#[inline]
pub fn two_prod(a: f64, b: f64) -> Option<(f64, f64)> {
    #[cfg(target_feature = "fma")]
    {
        let p = a * b;
        if p == 0.0 {
            // Exact iff a factor is zero; otherwise the product underflowed.
            return if a == 0.0 || b == 0.0 {
                Some((p, 0.0))
            } else {
                None
            };
        }
        let e = f64::mul_add(a, b, -p);
        if !p.is_finite() || !e.is_finite() || p.abs() < TWO_PROD_MIN {
            return None;
        }
        Some((p, e))
    }
    #[cfg(not(target_feature = "fma"))]
    {
        two_prod_dekker(a, b)
    }
}

/// [`two_prod`] without FMA: Dekker's exact product over Veltkamp splits,
/// with the same refusals.
///
/// The split multiplies by `2²⁷ + 1`, which overflows for `|x| ≥ 2⁹⁹⁶`. Since
/// `p` is finite, at most one factor is that large and the other is below
/// `2²⁸`, so the factor of `2²⁸` is moved from one to the other: `a·b` is
/// unchanged and both rescalings are exact. A product of `2⁹⁹⁵` or more is
/// computed `2²⁸` smaller and scaled back, so the rounded-up high halves
/// cannot overflow `xh·yh`. Every partial product is a
/// multiple of `ulp(a)·ulp(b)`, which the `TWO_PROD_MIN` guard keeps at or
/// above the smallest subnormal, so no partial product underflows.
#[inline]
fn two_prod_dekker(a: f64, b: f64) -> Option<(f64, f64)> {
    const SPLIT_MAX: f64 = 6.696_928_794_914_171e299; // 2^996
    const DOWN: f64 = 3.725_290_298_461_914e-9; // 2^-28
    const UP: f64 = 268_435_456.0; // 2^28
    let p = a * b;
    if p == 0.0 {
        // Exact iff a factor is zero; otherwise the product underflowed.
        return if a == 0.0 || b == 0.0 {
            Some((p, 0.0))
        } else {
            None
        };
    }
    if !p.is_finite() || p.abs() < TWO_PROD_MIN {
        return None;
    }
    if p.abs() >= SPLIT_MAX / 2.0 {
        // Rounded-up high halves could overflow `xh·yh`: compute the product
        // 2²⁸ smaller and scale both components back, exactly.
        let (a, b) = if a.abs() >= b.abs() {
            (a * DOWN, b)
        } else {
            (a, b * DOWN)
        };
        let (p, e) = two_prod_dekker(a, b)?;
        return Some((p * UP, e * UP));
    }
    let (x, y) = if a.abs() >= SPLIT_MAX {
        (a * DOWN, b * UP)
    } else if b.abs() >= SPLIT_MAX {
        (a * UP, b * DOWN)
    } else {
        (a, b)
    };
    let (xh, xl) = veltkamp_split(x);
    let (yh, yl) = veltkamp_split(y);
    let e = ((xh * yh - p) + xh * yl + xl * yh) + xl * yl;
    if !e.is_finite() {
        return None;
    }
    Some((p, e))
}

/// `x = hi + lo` exactly, each half holding at most 26 significant bits.
/// Requires `|x| < 2⁹⁹⁶`.
#[inline]
fn veltkamp_split(x: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0; // 2^27 + 1
    let c = SPLITTER * x;
    let hi = c - (c - x);
    (hi, x - hi)
}

/// `2⁻⁹⁶⁹` — the smallest `|a·b|` for which [`two_prod`]'s error term is
/// exactly representable. Written as a product of exact powers of two.
const TWO_PROD_MIN: f64 = f64::MIN_POSITIVE * ((1_u64 << 53) as f64);

/// Grow an expansion by one term. Returns `None` when the expansion cannot be
/// represented exactly.
///
/// Fail-closed on overflow. [`two_sum`] is exact only while no intermediate
/// overflows, and its `s - a` step can overflow for large opposite-sign
/// operands *even when the sum is finite* (see `eft`'s module docs). The error
/// term is then non-finite and the component it would contribute is garbage.
/// Propagating that silently would corrupt the expansion and — because
/// [`sign_of_expansion`] can neither compare NaN as positive nor as negative —
/// surface as a **wrong sign** rather than as a failure. So refuse instead.
pub fn grow_expansion(e: &[f64], b: f64) -> Option<Vec<f64>> {
    let mut h = Vec::with_capacity(e.len() + 1);
    let mut q = b;
    for &ei in e {
        let (qn, hi) = two_sum(q, ei);
        if !qn.is_finite() || !hi.is_finite() {
            return None;
        }
        if hi != 0.0 {
            h.push(hi);
        }
        q = qn;
    }
    h.push(q);
    Some(h)
}

/// Sum of two expansions. `None` if any step cannot be represented exactly
/// (see [`grow_expansion`]).
pub fn expansion_sum(e: &[f64], f: &[f64]) -> Option<Vec<f64>> {
    let mut h = e.to_vec();
    for &fi in f {
        h = grow_expansion(&h, fi)?;
    }
    Some(h)
}

/// Exact sign of the value of an expansion (largest component last decides
/// after zero-elimination performed by construction).
///
/// Returns `None` if any component is non-finite: such an expansion carries no
/// decidable sign, and the scanning loop below would otherwise *skip* it —
/// NaN is neither `> 0.0` nor `< 0.0` — and report the sign of a lower-order
/// component, or `Zero`. Silently wrong is the one outcome this tower must
/// never produce.
pub fn sign_of_expansion(e: &[f64]) -> Option<Sign> {
    if e.iter().any(|c| !c.is_finite()) {
        return None;
    }
    // Components are non-overlapping: the last nonzero one dominates.
    for &c in e.iter().rev() {
        if c > 0.0 {
            return Some(Sign::Positive);
        }
        if c < 0.0 {
            return Some(Sign::Negative);
        }
    }
    Some(Sign::Zero)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grow_and_sign() {
        // 0.1 + 0.2 - 0.3 is nonzero in f64 but its exact expansion sum
        // tracks the true value of the *doubles*, so build it exactly:
        let e = grow_expansion(&[0.1], 0.2).expect("no overflow");
        let e = grow_expansion(&e, -(0.1 + 0.2)).expect("no overflow");
        // Exact: 0.1 + 0.2 - fl(0.1+0.2) = -rounding_error(0.1+0.2) ≠ 0.
        assert_ne!(sign_of_expansion(&e), Some(Sign::Zero));
        assert!(sign_of_expansion(&e).is_some());
    }

    /// Fail-closed rather than silently-wrong: an expansion built across the
    /// overflow boundary must refuse to produce a sign. Pre-fix,
    /// `grow_expansion` pushed a NaN component and `sign_of_expansion`
    /// *skipped* it (NaN is neither `> 0.0` nor `< 0.0`), reporting the sign
    /// of a lower-order term.
    #[test]
    fn overflow_refuses_instead_of_lying() {
        // The eft-module regression pair: finite sum, poisoned error term.
        assert_eq!(grow_expansion(&[-f64::MAX], 7.172286964063675e307), None);
        // A directly poisoned expansion has no decidable sign.
        assert_eq!(sign_of_expansion(&[1.0, f64::NAN]), None);
        assert_eq!(sign_of_expansion(&[1.0, f64::INFINITY]), None);
        // ...and the honest cases still decide.
        assert_eq!(sign_of_expansion(&[]), Some(Sign::Zero));
        assert_eq!(sign_of_expansion(&[0.0, -2.0]), Some(Sign::Negative));
    }

    #[test]
    fn two_prod_exact() {
        let (p, e) = two_prod(1.0 + f64::EPSILON, 1.0 + f64::EPSILON).expect("no underflow");
        // (1+ε)² = 1 + 2ε + ε²; fl gives 1+2ε, error ε².
        assert_eq!(p, 1.0 + 2.0 * f64::EPSILON);
        assert_eq!(e, f64::EPSILON * f64::EPSILON);
    }

    /// Underflow produces *finite* wrong components, so the overflow guards
    /// never fire. Pre-fix, `two_prod(1e-200, 1e-200)` gave `(0.0, 0.0)` and
    /// `sign_of_expansion` certified `Zero` for a strictly positive product.
    #[test]
    fn two_prod_refuses_on_underflow() {
        assert_eq!(two_prod(1e-200, 1e-200), None);
        assert_eq!(two_prod(f64::MIN_POSITIVE, f64::MIN_POSITIVE), None);
        // A zero factor is a genuinely exact zero product, and is accepted.
        assert_eq!(two_prod(0.0, 1e300), Some((0.0, 0.0)));
        assert_eq!(two_prod(1e-200, 0.0), Some((0.0, 0.0)));
        // Overflow is refused too.
        assert_eq!(two_prod(1e300, 1e300), None);
        // Ordinary magnitudes are unaffected.
        assert_eq!(two_prod(3.0, 0.5), Some((1.5, 0.0)));
    }

    fn exact_product_identity(a: f64, b: f64, p: f64, e: f64) -> bool {
        use crate::RingOps;
        use crate::exact::Rational;
        let r = |x: f64| Rational::from_f64(x).expect("finite");
        r(a).mul(&r(b)) == r(p).add(&r(e)) && p == a * b
    }

    /// The software path must be exact without hardware FMA, including at
    /// magnitudes where the Veltkamp split constant `2²⁷+1` would overflow.
    #[test]
    fn two_prod_dekker_exact_at_extremes() {
        for &(a, b) in &[
            (1.0 + f64::EPSILON, 1.0 + f64::EPSILON),
            (1e300, 1e-10),
            (f64::MAX, 0.5),
            (-f64::MAX, -0.999_999_999_999),
            (1.797e308, 1e-308),
            (f64::MIN_POSITIVE, 3.0e60),
            (5e-324, 1.0e300),
            (0.1, 0.3),
        ] {
            let (p, e) = two_prod_dekker(a, b).unwrap_or_else(|| panic!("refused {a:e} * {b:e}"));
            assert!(exact_product_identity(a, b, p, e), "{a:e} * {b:e}");
            assert_eq!(two_prod(a, b), Some((p, e)), "{a:e} * {b:e}");
        }
        assert_eq!(two_prod_dekker(1e-200, 1e-200), None);
        assert_eq!(two_prod_dekker(1e300, 1e300), None);
        assert_eq!(two_prod_dekker(0.0, f64::MAX), Some((0.0, 0.0)));
    }

    proptest::proptest! {
        #[test]
        fn two_prod_dekker_matches_exact(
            a in proptest::num::f64::NORMAL | proptest::num::f64::SUBNORMAL,
            b in proptest::num::f64::NORMAL | proptest::num::f64::SUBNORMAL,
        ) {
            if let Some((p, e)) = two_prod_dekker(a, b) {
                proptest::prop_assert!(exact_product_identity(a, b, p, e));
                proptest::prop_assert_eq!(Some((p, e)), two_prod(a, b));
            } else {
                proptest::prop_assert_eq!(None, two_prod(a, b));
            }
        }

        /// Random normals mostly overflow or underflow; this keeps `a·b` in
        /// range across the whole exponent span, near both refusal edges.
        #[test]
        fn two_prod_dekker_matches_exact_in_range(
            a in proptest::num::f64::NORMAL,
            frac in -2.0f64..2.0,
            k in -1022i32..1023,
        ) {
            let b = frac * 2f64.powi(k);
            if let Some((p, e)) = two_prod_dekker(a, b) {
                proptest::prop_assert!(exact_product_identity(a, b, p, e));
                proptest::prop_assert_eq!(Some((p, e)), two_prod(a, b));
            } else {
                let p = a * b;
                proptest::prop_assert!(
                    !p.is_finite() || p.abs() < TWO_PROD_MIN || (p == 0.0 && b != 0.0)
                );
            }
        }
    }
}
