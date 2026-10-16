//! Per-rung cost of the adaptive predicate ladder: the same predicate text
//! evaluated at each rung, and `adaptive_sign` on generic and degenerate data.
//!
//! Dependency-free (harness = false). Timings are observational; on a loaded
//! host compare the ratios in the last column, not the absolute times.

use resolvent::exact::Rational;
use resolvent::ladder::det3;
use resolvent::rungs::{
    DdBound, PolyPredicate, RungArith, RunningError, StackExpansion, adaptive_sign,
};
use std::hint::black_box;
use std::time::Instant;

struct Orient2;
struct Orient3;
struct InCircle;

impl PolyPredicate for Orient2 {
    fn arity(&self) -> usize {
        6
    }
    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        let (bax, bay) = (i[2].sub(&i[0]), i[3].sub(&i[1]));
        let (cax, cay) = (i[4].sub(&i[0]), i[5].sub(&i[1]));
        bax.mul(&cay).sub(&bay.mul(&cax))
    }
}

impl PolyPredicate for Orient3 {
    fn arity(&self) -> usize {
        12
    }
    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        let d = |k: usize| [i[k].sub(&i[9]), i[k + 1].sub(&i[10]), i[k + 2].sub(&i[11])];
        let (a, b, c) = (d(0), d(3), d(6));
        det3(
            &a[0], &a[1], &a[2], &b[0], &b[1], &b[2], &c[0], &c[1], &c[2],
        )
    }
}

impl PolyPredicate for InCircle {
    fn arity(&self) -> usize {
        8
    }
    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        let (adx, ady) = (i[0].sub(&i[6]), i[1].sub(&i[7]));
        let (bdx, bdy) = (i[2].sub(&i[6]), i[3].sub(&i[7]));
        let (cdx, cdy) = (i[4].sub(&i[6]), i[5].sub(&i[7]));
        let alift = adx.mul(&adx).add(&ady.mul(&ady));
        let blift = bdx.mul(&bdx).add(&bdy.mul(&bdy));
        let clift = cdx.mul(&cdx).add(&cdy.mul(&cdy));
        let bc = bdx.mul(&cdy).sub(&cdx.mul(&bdy));
        let ca = cdx.mul(&ady).sub(&adx.mul(&cdy));
        let ab = adx.mul(&bdy).sub(&bdx.mul(&ady));
        alift.mul(&bc).add(&blift.mul(&ca)).add(&clift.mul(&ab))
    }
}

fn xorshift(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    ((*state >> 11) as f64 / (1u64 << 52) as f64) - 1.0
}

fn lift_all<N: RungArith>(sets: &[Vec<f64>]) -> Vec<Vec<N>> {
    sets.iter()
        .map(|s| s.iter().map(|&x| N::lift(x)).collect())
        .collect()
}

fn time_eval<P: PolyPredicate, N: RungArith>(p: &P, sets: &[Vec<N>], reps: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..reps {
        for s in sets {
            black_box(p.eval(black_box(s)));
        }
    }
    start.elapsed().as_secs_f64() * 1e6 / (reps * sets.len()) as f64
}

fn report<P: PolyPredicate>(name: &str, p: &P, make: impl Fn(&mut u64) -> Vec<f64>) {
    let mut st = 0x243F6A8885A308D3u64;
    let sets: Vec<Vec<f64>> = (0..2000).map(|_| make(&mut st)).collect();
    let t1 = time_eval(p, &lift_all::<RunningError>(&sets), 50);
    let t2 = time_eval(p, &lift_all::<DdBound>(&sets), 50);
    let t3 = time_eval(p, &lift_all::<StackExpansion<32>>(&sets), 20);
    let t4 = time_eval(p, &lift_all::<Rational>(&sets), 2);
    println!(
        "{name:<10} eval us: r1 {t1:8.3} | r2 {t2:8.3} ({:5.1}x) | r3 {t3:8.3} ({:5.1}x) | r4 {t4:9.3} ({:6.1}x)",
        t2 / t1,
        t3 / t1,
        t4 / t1
    );
    let start = Instant::now();
    for s in &sets {
        black_box(adaptive_sign(p, black_box(s)).expect("valid"));
    }
    let adaptive = start.elapsed().as_secs_f64() * 1e6 / sets.len() as f64;
    println!(
        "{name:<10} adaptive_sign us/call: {adaptive:8.3} ({:5.1}x r1 eval)",
        adaptive / t1
    );
}

fn main() {
    println!("-- generic (random full-mantissa) inputs --");
    report("orient2", &Orient2, |s| {
        (0..6).map(|_| xorshift(s)).collect()
    });
    report("orient3", &Orient3, |s| {
        (0..12).map(|_| xorshift(s)).collect()
    });
    report("incircle", &InCircle, |s| {
        (0..8).map(|_| xorshift(s)).collect()
    });
    println!("-- exact zeros through powers of two (rung 3 decides) --");
    report("orient2", &Orient2, |s| {
        let (x, y) = (xorshift(s), xorshift(s));
        vec![x, y, 2.0 * x, 2.0 * y, 4.0 * x, 4.0 * y]
    });
    println!("-- integer inputs, exact zero (rung 1 decides) --");
    report("orient2", &Orient2, |_| vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);
}
