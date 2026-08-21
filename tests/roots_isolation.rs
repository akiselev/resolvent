//! Regressions for `isolate_roots`' two contract clauses: the returned roots
//! are **pairwise distinct** and in **ascending order**.
//!
//! Both were violated. `isolate_square_free` deflated an exact rational root
//! and then recursed over the deflated polynomial's own Cauchy bound, which
//! re-isolated every root already found (`q`'s roots are `p`'s roots minus
//! `mid` — including the ones in `out`) over intervals that straddled them.
//! Duplicates compared `Equal` with overlapping intervals no refinement could
//! separate, and the overlap broke the ascending sort as well. A sweep-line
//! consumer would see two events at one point, or events out of order.

use core::cmp::Ordering;
use resolvent::exact::RingOps;
use resolvent::{QPoly, Rational, RealRoot, isolate_roots};

/// Expanded monic-ish product of `(x - n/d)` over the given rational roots.
fn from_roots(roots: &[(i64, i64)]) -> QPoly {
    let mut p = QPoly::from_i64s(&[1]);
    for &(n, d) in roots {
        let r = Rational::from_ratio(n, d);
        p = p.mul_poly(&QPoly::new(vec![r.neg(), Rational::one()]));
    }
    p
}

// A `#[test]`-adjacent helper, not production code: clippy's
// `allow-panic-in-tests` only covers functions carrying `#[test]`, and this
// `panic!` *is* the failure report for the assertions below.
#[allow(clippy::panic)] // test helper; the panic is the assertion
fn check(p: &QPoly, expected: usize, label: &str) {
    let mut rs = isolate_roots(p).unwrap_or_else(|e| panic!("{label}: {e:?}"));
    assert_eq!(rs.len(), expected, "{label}: wrong root count");

    // Pairwise distinct, and ascending.
    for i in 0..rs.len().saturating_sub(1) {
        let (head, tail) = rs.split_at_mut(i + 1);
        let ord = head[i].cmp_root(&mut tail[0]);
        assert_eq!(
            ord,
            Ordering::Less,
            "{label}: roots {i} and {} are not strictly ascending",
            i + 1
        );
    }
    // Disjointness is what the ascending sort key relies on.
    for i in 0..rs.len().saturating_sub(1) {
        assert!(
            rs[i].hi() <= rs[i + 1].lo(),
            "{label}: isolating intervals {i} and {} overlap",
            i + 1
        );
    }
}

#[test]
fn exact_midpoint_root_does_not_duplicate_earlier_roots() {
    // 2x^3 + x^2 - 2x - 1 = (x-1)(x+1)(2x+1); roots {-1, -1/2, 1}.
    // Returned 4 roots, the last two both isolating 1 over [0, 3/2] and [0, 2].
    check(&QPoly::from_i64s(&[-1, -2, 1, 2]), 3, "2x^3+x^2-2x-1");
}

#[test]
fn exact_root_at_zero_sorts_before_a_smaller_root() {
    // x^2 + 2x = x(x+2); roots {-2, 0}. Returned 0 (exact, key 0) before
    // -2 (interval [-3, 3], key 3).
    check(&QPoly::from_i64s(&[0, 2, 1]), 2, "x^2+2x");
}

#[test]
fn products_of_distinct_rational_roots() {
    // Deterministic sweep. The two defects showed at ~1.5% and ~5% of random
    // rational-root products, so a few hundred cases is a real gate.
    let mut state: u64 = 0x5eed_1234_9abc_def0;
    let mut next = |m: i64| -> i64 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as i64) % m
    };

    let mut cases = 0;
    for _ in 0..400 {
        let k = 2 + (next(4).unsigned_abs() as usize % 4); // 2..=5 roots
        let mut roots: Vec<(i64, i64)> = Vec::new();
        for _ in 0..k {
            let n = next(13) - 6; // -6..=6
            let d = 1 + next(4).abs(); // 1..=4
            // Keep them distinct as rationals so the count is known.
            if roots.iter().any(|&(a, b)| a * d == n * b) {
                continue;
            }
            roots.push((n, d));
        }
        if roots.len() < 2 {
            continue;
        }
        let p = from_roots(&roots);
        check(&p, roots.len(), &format!("roots {roots:?}"));
        cases += 1;
    }
    assert!(cases > 200, "sweep degenerated to {cases} cases");
}

#[test]
fn every_isolated_interval_really_contains_its_root() {
    // Guards against "fixing" uniqueness by dropping a real root.
    for spec in [
        vec![(0, 1), (-2, 1)],
        vec![(1, 1), (-1, 1), (-1, 2)],
        vec![(3, 2), (-3, 2), (0, 1), (5, 4)],
    ] {
        let p = from_roots(&spec);
        let rs = isolate_roots(&p).unwrap();
        assert_eq!(rs.len(), spec.len());
        for (n, d) in spec {
            let want = Rational::from_ratio(n, d);
            assert!(
                rs.iter().any(|r| r.lo() <= &want && &want <= r.hi()),
                "root {n}/{d} was not enclosed by any returned interval"
            );
        }
    }
}

/// `RealRoot` is refined against whatever polynomial it was isolated with,
/// which after in-place deflation is not always the fully deflated one. That
/// is sound — the interval still isolates exactly one of that polynomial's
/// roots — and this pins it.
#[test]
fn refinement_converges_after_in_place_deflation() {
    let p = QPoly::from_i64s(&[-1, -2, 1, 2]); // exact root at -1 hit mid-bisection
    let mut rs = isolate_roots(&p).unwrap();
    for r in &mut rs {
        let before = r.to_interval();
        for _ in 0..40 {
            r.refine();
        }
        let after: resolvent::Interval = r.to_interval();
        assert!(after.inf() >= before.inf() && after.sup() <= before.sup());
        assert!(
            RealRoot::is_exact(r) || after.sup() - after.inf() < 1e-9,
            "refinement did not converge"
        );
    }
}
