//! M0 property tests: every interval operation encloses the exact rational
//! result; `to_interval` is an outward-correct enclosure (DESIGN.md §7 M0).

use proptest::prelude::*;
use resolvent::exact::{ExactField, ExactRing, Rational, RingOps};
use resolvent::interval::Interval;
use resolvent::uncertain::{Sign, Uncertain};

/// The branches of [`finite_f64`] that are not *deliberately* zero.
///
/// Split out so [`nonzero_f64`] can avoid filtering away the two `Just(±0.0)`
/// branches. Filtering them rejected ~25% of every draw, and proptest aborts
/// a test once local rejects pass `max_local_rejects` (65,536 by default) —
/// so any `nonzero` property silently stopped well short of its configured
/// case count instead of failing loudly. At 200,000 cases both `div_encloses`
/// and `zero_products_never_claim_wrong_exactness` aborted for exactly this
/// reason, which reads like a soundness failure and is not one.
fn finite_f64_nonzero_branches() -> impl Strategy<Value = f64> {
    prop_oneof![
        // Ordinary magnitudes.
        -1e12f64..1e12f64,
        // Tiny/subnormal territory.
        prop::num::f64::ANY.prop_map(|x| if x.is_finite() { x * 1e-300 } else { 0.0 }),
        // Huge.
        prop::num::f64::ANY.prop_filter("finite", |x| x.is_finite()),
        // Exact specials.
        Just(f64::MAX),
        Just(f64::MIN_POSITIVE),
        Just(f64::from_bits(1)), // smallest subnormal
    ]
}

/// Finite, non-NaN doubles across the full range including subnormals,
/// zeros, and huge magnitudes. Weighted 6:1:1 to preserve the original
/// eight-branch `prop_oneof!` distribution.
fn finite_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        6 => finite_f64_nonzero_branches(),
        1 => Just(0.0f64),
        1 => Just(-0.0f64),
    ]
}

/// A filter is still required — the ranged and scaled branches can land on
/// zero — but it now rejects rarely instead of on a quarter of all draws.
fn nonzero_f64() -> impl Strategy<Value = f64> {
    finite_f64_nonzero_branches().prop_filter("nonzero", |x| *x != 0.0)
}

// A proptest helper, not production code: every strategy in this file draws
// from `finite_f64*`, and the callers below guard with `is_finite()`, so the
// `expect` cannot fire — and clippy's `allow-expect-in-tests` does not reach
// helpers that lack `#[test]` themselves.
#[allow(clippy::expect_used)] // test helper; all callers supply finite doubles
fn q(x: f64) -> Rational {
    Rational::from_f64(x).expect("interval_props strategies only draw finite doubles")
}

/// The exact value `v` must satisfy `inf ≤ v ≤ sup`, comparing exactly.
fn encloses(iv: Interval, v: &Rational) {
    if iv.inf().is_finite() {
        assert!(
            &q(iv.inf()) <= v,
            "inf {} !≤ exact (interval {iv:?})",
            iv.inf()
        );
    } else {
        assert_eq!(iv.inf(), f64::NEG_INFINITY);
    }
    if iv.sup().is_finite() {
        assert!(
            v <= &q(iv.sup()),
            "exact !≤ sup {} (interval {iv:?})",
            iv.sup()
        );
    } else {
        assert_eq!(iv.sup(), f64::INFINITY);
    }
}

/// Deterministic regression for the enclosure failure `sub_encloses` found by
/// chance during the cadabra3 absorption (2026-08-09).
///
/// `eft::two_sum`'s intermediate `s - a` overflows for large opposite-sign
/// operands even though the sum is finite, so the error term is NaN; the
/// pre-fix `if e > 0.0` / `if e < 0.0` tests are both false for NaN, and the
/// bound came back unwidened — half an ulp *inside* the true value. Pinned
/// here as a plain `#[test]` rather than relying on a proptest seed, because
/// the generator reaches this pair only rarely (100k cases did not).
#[test]
fn sub_encloses_across_the_overflow_boundary() {
    let cases = [
        (7.172286964063675e307, f64::MAX),
        (f64::MAX, -7.172286964063675e307),
        (-7.172286964063675e307, -f64::MAX),
        (f64::MAX, -f64::MAX),
    ];
    for (a, b) in cases {
        encloses(Interval::point(a) - Interval::point(b), &q(a).sub(&q(b)));
        encloses(Interval::point(a) + Interval::point(b), &q(a).add(&q(b)));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn add_encloses(a in finite_f64(), b in finite_f64()) {
        let iv = Interval::point(a) + Interval::point(b);
        encloses(iv, &q(a).add(&q(b)));
    }

    #[test]
    fn sub_encloses(a in finite_f64(), b in finite_f64()) {
        let iv = Interval::point(a) - Interval::point(b);
        encloses(iv, &q(a).sub(&q(b)));
    }

    #[test]
    fn mul_encloses(a in finite_f64(), b in finite_f64()) {
        let iv = Interval::point(a) * Interval::point(b);
        encloses(iv, &q(a).mul(&q(b)));
    }

    #[test]
    fn div_encloses(a in finite_f64(), b in nonzero_f64()) {
        let iv = Interval::point(a) / Interval::point(b);
        encloses(iv, &q(a).div(&q(b)));
    }

    #[test]
    fn square_encloses(a in finite_f64()) {
        let iv = Interval::point(a).square();
        encloses(iv, &q(a).mul(&q(a)));
    }

    #[test]
    fn interval_ops_enclose_endpoint_combinations(
        a in finite_f64(), b in finite_f64(),
        c in finite_f64(), d in finite_f64(),
    ) {
        // Non-degenerate intervals: ops must enclose all corner values.
        let i1 = Interval::new(a.min(b), a.max(b));
        let i2 = Interval::new(c.min(d), c.max(d));
        let sum = i1 + i2;
        let prod = i1 * i2;
        for x in [a, b] {
            for y in [c, d] {
                encloses(sum, &q(x).add(&q(y)));
                encloses(prod, &q(x).mul(&q(y)));
            }
        }
    }

    #[test]
    fn to_interval_encloses_and_is_tight(n in any::<i64>(), d in 1..i64::MAX) {
        let v = Rational::from_ratio(n, d);
        let iv = v.to_interval();
        encloses(iv, &v);
        // Tightness: at most 1 ulp wide (or a point).
        if iv.inf().is_finite() && iv.sup().is_finite() && !iv.is_point() {
            assert_eq!(iv.sup(), iv.inf().next_up());
        }
    }

    #[test]
    fn to_interval_roundtrip_exact_doubles(x in finite_f64()) {
        let iv = q(x).to_interval();
        assert!(iv.is_point());
        assert_eq!(iv.inf(), x, "exact double must roundtrip to a point");
    }

    #[test]
    fn sign_never_lies(a in finite_f64(), b in finite_f64(), c in finite_f64(), d in finite_f64()) {
        // Filtered det2 sign vs exact rational sign.
        let filtered = {
            let p = Interval::point;
            resolvent::ladder::det2(&p(a), &p(b), &p(c), &p(d)).sign()
        };
        let exact = resolvent::ladder::det2(&q(a), &q(b), &q(c), &q(d)).sign();
        match filtered {
            Uncertain::Certain(s) => assert_eq!(s, exact, "filter certified a wrong sign"),
            Uncertain::Unknown => {} // always allowed
        }
    }

    #[test]
    fn zero_products_never_claim_wrong_exactness(a in nonzero_f64()) {
        // tiny * tiny may round to zero: sign() must not certify Zero.
        let t = Interval::point(a) * Interval::point(f64::from_bits(1));
        if let Uncertain::Certain(Sign::Zero) = t.sign() {
            // Only allowed if the exact product is really zero.
            assert_eq!(q(a).mul(&q(f64::from_bits(1))).sign(), Sign::Zero);
        }
    }
}
