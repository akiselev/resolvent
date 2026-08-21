//! Shewchuk expansion arithmetic primitives (DESIGN.md §3.3).
//!
//! An *expansion* is a sum of non-overlapping doubles, largest last; the
//! primitives here are the exact building blocks for adaptive exact
//! predicate rungs. Only the primitives used by the shipped ladders are
//! implemented; the module grows with the predicates that need it.

use crate::eft::two_sum;
use crate::uncertain::Sign;

/// TwoProd via FMA: `Some((p, e))` with `p = fl(a·b)` and `p + e = a·b`
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
/// Without the `fma` target feature `mul_add` is a correct (slower) libm call.
#[inline]
pub fn two_prod(a: f64, b: f64) -> Option<(f64, f64)> {
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
}
