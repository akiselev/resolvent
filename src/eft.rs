//! Error-free transformations and directed bounds for `+`/`-` under
//! round-to-nearest (DESIGN.md §3.2, default tier).
//!
//! TwoSum is an *exact* transformation for all finite doubles — the rounding
//! error of an addition is always representable, including in the subnormal
//! range (underflowing additions are exact) — **provided no intermediate
//! overflows**. Two distinct guards are therefore needed:
//!
//! 1. **Final-sum overflow.** When round-to-nearest returns `+∞` the true sum
//!    exceeds `f64::MAX`, so `MAX` is a sound lower bound (symmetrically for
//!    `-∞`). The bounds below short-circuit to a clamp.
//! 2. **Intermediate overflow with a finite sum.** `two_sum` computes
//!    `bv = s - a`, which can overflow to `±∞` for large opposite-sign
//!    operands *even when `s` itself is finite* — e.g. `a = 7.172286964063675e307`,
//!    `b = -f64::MAX` gives a finite `s` but `bv = -∞`, `av = +∞`, and
//!    `e = -∞ + ∞ = NaN`. Every NaN comparison is false, so a naive
//!    `if e > 0.0` silently takes the "exact" branch and returns `s`
//!    unwidened — leaving `sup` half an ulp *below* the true sum. A
//!    non-enclosing interval can then certify a wrong sign, which is the one
//!    failure this whole tower exists to prevent. The bounds below detect a
//!    non-finite error term and fall back to [`blind_lo`]/[`blind_hi`].
//!
//! Guard 2 was added after a `sub_encloses` property-test failure; see
//! `intermediate_overflow_still_encloses` in this module's tests.

/// Knuth TwoSum: returns `(s, e)` with `s = fl(a + b)` and `s + e = a + b`
/// exactly.
///
/// Precondition: no intermediate overflows. `s` finite is **not** sufficient —
/// `s - a` can still overflow (see the module docs). When it does, `e` is
/// non-finite and carries no information; callers must check.
#[inline]
pub fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bv = s - a;
    let av = s - bv;
    let e = (a - av) + (b - bv);
    (s, e)
}

/// Lower bound of the exact sum `a + b`, one-sided-widened.
#[inline]
pub fn add_lo(a: f64, b: f64) -> f64 {
    let s = a + b;
    if s == f64::INFINITY {
        // true sum > MAX: MAX is a valid (finite) lower bound.
        return f64::MAX;
    }
    if s == f64::NEG_INFINITY {
        // true sum < -MAX: no finite lower bound is sound.
        return f64::NEG_INFINITY;
    }
    let (s, e) = two_sum(a, b);
    if !e.is_finite() {
        // Guard 2 (module docs): an intermediate overflowed despite a finite
        // `s`, so `e` carries no information. `s` is correctly rounded, hence
        // within half an ulp of the true sum; one step down encloses it.
        // Checked positively (`is_finite`) rather than by `e.is_nan()`, so a
        // ±∞ error term takes this branch too instead of the sign test.
        return blind_lo(s);
    }
    if e < 0.0 { s.next_down() } else { s }
}

/// Upper bound of the exact sum `a + b`, one-sided-widened.
#[inline]
pub fn add_hi(a: f64, b: f64) -> f64 {
    let s = a + b;
    if s == f64::NEG_INFINITY {
        return f64::MIN; // true sum < -MAX: -MAX is a valid upper bound.
    }
    if s == f64::INFINITY {
        return f64::INFINITY;
    }
    let (s, e) = two_sum(a, b);
    if !e.is_finite() {
        // Mirror of `add_lo`; see there and the module docs.
        return blind_hi(s);
    }
    if e > 0.0 { s.next_up() } else { s }
}

/// Lower bound of a rounded-to-nearest result whose error direction is
/// unknown (blind outward widening). Sound for any correctly-rounded op:
/// the true value is within half an ulp, so one step down covers it.
/// `next_down(+∞) = MAX` makes the overflow case sound automatically.
#[inline]
pub fn blind_lo(r: f64) -> f64 {
    r.next_down()
}

/// Upper bound counterpart of [`blind_lo`].
#[inline]
pub fn blind_hi(r: f64) -> f64 {
    r.next_up()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_sum_exactness_small() {
        let (s, e) = two_sum(0.1, 0.2);
        // s + e reconstructs exactly in higher precision: check the defining
        // identity via integer-exact doubles.
        assert_eq!(s, 0.1 + 0.2);
        // e is the negation of the rounding error of s.
        assert_ne!(e, 0.0); // 0.1+0.2 is inexact in binary64
    }

    /// Regression (absorption, 2026-08-09): `two_sum`'s intermediate
    /// `bv = s - a` overflows for large opposite-sign operands **even though
    /// the sum itself is finite**, poisoning the error term to NaN. Every NaN
    /// comparison is false, so the pre-fix `if e > 0.0` took the "exact"
    /// branch and returned an unwidened bound sitting half an ulp *inside*
    /// the true value — a non-enclosing interval, which can certify a wrong
    /// sign. Found by `interval_props::sub_encloses`; see the module docs.
    #[test]
    fn intermediate_overflow_still_encloses() {
        let a = 7.172286964063675e307;
        let b = -f64::MAX;

        let s = a + b;
        assert!(s.is_finite(), "the rounded sum does not overflow");
        let (ts, e) = two_sum(a, b);
        assert_eq!(ts, s);
        assert!(!e.is_finite(), "but the error term is poisoned");

        // Both bounds must straddle `s`, because the true sum is strictly
        // between `s` and `s.next_up()` (the deficit is exactly half an ulp).
        assert_eq!(add_lo(a, b), s.next_down());
        assert_eq!(add_hi(a, b), s.next_up());
    }

    /// The blind fallback must not fire for ordinary operands: an exactly
    /// representable sum still returns a degenerate (unwidened) bound, so the
    /// fix costs no precision away from the overflow boundary.
    #[test]
    fn blind_fallback_does_not_fire_normally() {
        for (a, b) in [(1.0, 1.0), (0.5, 0.25), (-3.0, 7.0), (1e300, -1e300)] {
            let (_, e) = two_sum(a, b);
            assert!(e.is_finite(), "{a} + {b} has a usable error term");
            assert_eq!(add_lo(a, b), a + b);
            assert_eq!(add_hi(a, b), a + b);
        }
    }

    #[test]
    fn add_overflow_clamps() {
        assert_eq!(add_lo(f64::MAX, f64::MAX), f64::MAX);
        assert_eq!(add_hi(f64::MAX, f64::MAX), f64::INFINITY);
        assert_eq!(add_hi(f64::MIN, f64::MIN), f64::MIN);
        assert_eq!(add_lo(f64::MIN, f64::MIN), f64::NEG_INFINITY);
    }

    #[test]
    fn add_exact_stays_point() {
        // 1 + 1 is exact: no widening on either side.
        assert_eq!(add_lo(1.0, 1.0), 2.0);
        assert_eq!(add_hi(1.0, 1.0), 2.0);
    }

    #[test]
    fn subnormal_addition_is_exact() {
        let tiny = f64::from_bits(3); // 3 * 2^-1074
        let (s, e) = two_sum(tiny, tiny);
        assert_eq!(e, 0.0); // subnormal + subnormal is exact
        assert_eq!(s, f64::from_bits(6));
        assert_eq!(add_lo(tiny, tiny), s);
        assert_eq!(add_hi(tiny, tiny), s);
    }
}
