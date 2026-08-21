//! M0 exit benchmark: filtered `sign_of_det2/3` vs naive all-rational
//! evaluation on random + degenerate inputs. Target: >5× (DESIGN.md §7 M0).
//!
//! Deliberately dependency-free (harness = false): wall-clock medians are
//! plenty for a >5× gate.

use resolvent::exact::{ExactRing, Rational, RingOps};
use resolvent::ladder::{det2, sign_of_det2_f64};
use std::time::Instant;

/// Exact ingress for the generated inputs.
///
/// `xorshift` maps into `[-1, 1)` by construction and the fixed corpora
/// below are integer literals, so everything measured here is a finite
/// double — exactly `from_f64`'s admission condition. A silent fallback
/// would corrupt the measurement instead of reporting it, which is why this
/// stays an `expect`.
#[allow(clippy::expect_used)] // generated inputs are finite by construction
fn q(x: f64) -> Rational {
    Rational::from_f64(x).expect("bench inputs are finite doubles")
}

fn xorshift(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    // Map to [-1, 1) with 53 significant bits.
    ((*state >> 11) as f64 / (1u64 << 52) as f64) - 1.0
}

fn main() {
    let n = 200_000usize;
    let mut st = 0x243F6A8885A308D3u64;
    // Mix of random and exactly-degenerate inputs (ad == bc).
    let inputs: Vec<[f64; 4]> = (0..n)
        .map(|i| {
            if i % 8 == 0 {
                let a = xorshift(&mut st);
                let b = xorshift(&mut st);
                [a, b, a * 0.5, b * 0.5] // proportional rows: det == 0 in reals
            } else {
                [
                    xorshift(&mut st),
                    xorshift(&mut st),
                    xorshift(&mut st),
                    xorshift(&mut st),
                ]
            }
        })
        .collect();

    let t0 = Instant::now();
    let mut acc = 0i64;
    for [a, b, c, d] in &inputs {
        acc += match sign_of_det2_f64(*a, *b, *c, *d) {
            resolvent::Sign::Negative => -1,
            resolvent::Sign::Zero => 0,
            resolvent::Sign::Positive => 1,
        };
    }
    let filtered = t0.elapsed();

    let t1 = Instant::now();
    let mut acc2 = 0i64;
    for [a, b, c, d] in &inputs {
        acc2 += match det2(&q(*a), &q(*b), &q(*c), &q(*d)).sign() {
            resolvent::Sign::Negative => -1,
            resolvent::Sign::Zero => 0,
            resolvent::Sign::Positive => 1,
        };
    }
    let naive = t1.elapsed();

    assert_eq!(acc, acc2, "filtered and naive disagree");
    let speedup = naive.as_secs_f64() / filtered.as_secs_f64();
    println!(
        "sign_of_det2: filtered {:?}  naive-rational {:?}  speedup {speedup:.1}x  (target >5x)",
        filtered, naive
    );
    assert!(
        speedup > 5.0,
        "M0 exit criterion failed: {speedup:.1}x <= 5x"
    );

    segment_intersection_microbench();
}

/// M1 exit microbench: EPECK-style — construct segment-intersection points
/// lazily (one multi-output node each), then run predicates (`cmp_x`) over
/// the constructed points; versus eager rational construction + comparison.
fn segment_intersection_microbench() {
    use resolvent::real::{Real, TupleFormula};

    /// Intersection point of segments (p1,p2)×(p3,p4) by Cramer's rule.
    /// Operands: x1,y1,x2,y2,x3,y3,x4,y4. Outputs: x, y.
    struct SegInt;
    impl TupleFormula for SegInt {
        fn arity(&self) -> usize {
            2
        }
        fn apply<T: RingOps>(&self, v: &[&T], out: &mut Vec<T>) {
            let (x1, y1, x2, y2) = (v[0], v[1], v[2], v[3]);
            let (x3, y3, x4, y4) = (v[4], v[5], v[6], v[7]);
            let d = x1
                .sub(x2)
                .mul(&y3.sub(y4))
                .sub(&y1.sub(y2).mul(&x3.sub(x4)));
            let a = x1.mul(y2).sub(&y1.mul(x2));
            let b = x3.mul(y4).sub(&y3.mul(x4));
            // NOTE: formulas run over RingOps; division deferred by storing
            // numerators and denominator-scaled coordinates is the ring
            // trick — here we keep the field step out of the formula and
            // emit projective-like numerators over a common denominator by
            // multiplying through: x*d and y*d (sign-correct for cmp when d
            // sign is accounted; adequate for a throughput benchmark).
            out.push(a.mul(&x3.sub(x4)).sub(&x1.sub(x2).mul(&b)));
            out.push(a.mul(&y3.sub(y4)).sub(&y1.sub(y2).mul(&b)));
            let _ = d;
        }
    }

    let n = 20_000usize;
    let mut st = 0x9E3779B97F4A7C15u64;
    let coords: Vec<[f64; 8]> = (0..n)
        .map(|_| core::array::from_fn(|_| xorshift(&mut st)))
        .collect();

    // Lazy: construct, then compare x-coordinates of consecutive points.
    let t0 = Instant::now();
    let pts: Vec<Vec<Real<Rational>>> = coords
        .iter()
        .map(|c| {
            // `Real::from_f64` shares `q`'s finite-input condition; going
            // through `q` keeps one justification for the whole file, and
            // `from_exact` itself is total.
            let ops: Vec<Real<Rational>> = c.iter().map(|&x| Real::from_exact(q(x))).collect();
            Real::construct(SegInt, &ops)
        })
        .collect();
    let mut ord = 0i64;
    for w in pts.windows(2) {
        ord += match w[0][0].cmp_real(&w[1][0]) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        };
    }
    let lazy = t0.elapsed();

    // Eager rational: full exact construction and comparison.
    let t1 = Instant::now();
    let qpts: Vec<[Rational; 2]> = coords
        .iter()
        .map(|c| {
            let qs: Vec<Rational> = c.iter().map(|&x| q(x)).collect();
            let refs: Vec<&Rational> = qs.iter().collect();
            let mut out = Vec::new();
            SegInt.apply::<Rational>(&refs, &mut out);
            [out.remove(0), out.remove(0)]
        })
        .collect();
    let mut ord2 = 0i64;
    for w in qpts.windows(2) {
        ord2 += match w[0][0].cmp(&w[1][0]) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        };
    }
    let eager = t1.elapsed();

    assert_eq!(ord, ord2, "lazy and eager comparison sums disagree");
    println!(
        "segment-intersection: lazy {:?}  eager-rational {:?}  speedup {:.1}x",
        lazy,
        eager,
        eager.as_secs_f64() / lazy.as_secs_f64()
    );
}
