//! Soundness, determinism and guard tests for the arithmetic rungs
//! (`resolvent::rungs`): every certified answer at every rung is compared to
//! exact rational evaluation of the same predicate text.

use proptest::prelude::*;
use resolvent::exact::{ExactRing, Rational, RingOps};
use resolvent::ladder::det3;
use resolvent::rungs::{
    AdaptiveError, AdaptiveSign, DdBound, PolyPredicate, Rung, RungArith, RunningError,
    StackExpansion, adaptive_sign, adaptive_sign_from, eval_rung,
};
use resolvent::{Sign, Uncertain};

type Exp32 = StackExpansion<32>;

// ---------------------------------------------------------------------------
// Predicates: one text, generic over the rung's number type.
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Kind {
    /// degree 2, 6 inputs
    Orient2,
    /// degree 3, 12 inputs
    Orient3,
    /// degree 4, 8 inputs
    InCircle,
    /// degree 2, 8 inputs: a*b + c*d - e*f - g*h
    SumOfProducts,
    /// degree 4, 8 inputs: (a*b - c*d) * (e*f - g*h)
    Quartic,
}

const KINDS: [Kind; 5] = [
    Kind::Orient2,
    Kind::Orient3,
    Kind::InCircle,
    Kind::SumOfProducts,
    Kind::Quartic,
];

struct Pred(Kind);

impl PolyPredicate for Pred {
    fn arity(&self) -> usize {
        match self.0 {
            Kind::Orient2 => 6,
            Kind::Orient3 => 12,
            Kind::InCircle | Kind::SumOfProducts | Kind::Quartic => 8,
        }
    }

    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        match self.0 {
            Kind::Orient2 => {
                let (bax, bay) = (i[2].sub(&i[0]), i[3].sub(&i[1]));
                let (cax, cay) = (i[4].sub(&i[0]), i[5].sub(&i[1]));
                bax.mul(&cay).sub(&bay.mul(&cax))
            }
            Kind::Orient3 => {
                let d = |k: usize| [i[k].sub(&i[9]), i[k + 1].sub(&i[10]), i[k + 2].sub(&i[11])];
                let (a, b, c) = (d(0), d(3), d(6));
                det3(
                    &a[0], &a[1], &a[2], &b[0], &b[1], &b[2], &c[0], &c[1], &c[2],
                )
            }
            Kind::InCircle => {
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
            Kind::SumOfProducts => i[0]
                .mul(&i[1])
                .add(&i[2].mul(&i[3]))
                .sub(&i[4].mul(&i[5]))
                .sub(&i[6].mul(&i[7])),
            Kind::Quartic => {
                let f = i[0].mul(&i[1]).sub(&i[2].mul(&i[3]));
                let g = i[4].mul(&i[5]).sub(&i[6].mul(&i[7]));
                f.mul(&g)
            }
        }
    }
}

/// `n` inputs: the first `n/2` summed minus the last `n/2`.
struct CancelSum(usize);

impl PolyPredicate for CancelSum {
    fn arity(&self) -> usize {
        self.0
    }
    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        let mut acc = N::zero();
        for x in i {
            acc = acc.add(x);
        }
        acc
    }
}

// ---------------------------------------------------------------------------
// Exact reference
// ---------------------------------------------------------------------------

fn q(x: f64) -> Rational {
    Rational::from_f64(x).expect("finite")
}

fn rabs(r: &Rational) -> Rational {
    if ExactRing::sign(r) == Sign::Negative {
        r.neg()
    } else {
        r.clone()
    }
}

fn exact_value<P: PolyPredicate>(p: &P, inputs: &[f64]) -> Rational {
    let lifted: Vec<Rational> = inputs.iter().map(|&x| q(x)).collect();
    p.eval(&lifted)
}

fn exact_sum(components: &[f64]) -> Rational {
    components
        .iter()
        .fold(Rational::zero(), |acc, &c| acc.add(&q(c)))
}

fn lifted<N: RungArith>(inputs: &[f64]) -> Vec<N> {
    inputs.iter().map(|&x| N::lift(x)).collect()
}

fn check_margin(margin: f64, exact: &Rational, what: &str) {
    assert!(
        margin >= 0.0 && margin.is_finite(),
        "{what}: margin {margin:e}"
    );
    assert!(
        q(margin) <= rabs(exact),
        "{what}: margin {margin:e} exceeds |exact|"
    );
}

/// Everything the rungs promise, checked against rational evaluation.
fn check_all<P: PolyPredicate>(p: &P, inputs: &[f64], what: &str) {
    let exact = exact_value(p, inputs);
    let esign = ExactRing::sign(&exact);

    // Rung 1: |v - exact| <= e, certified sign and margin.
    let r1 = p.eval(&lifted::<RunningError>(inputs));
    if r1.error().is_finite() && r1.value().is_finite() {
        let dev = rabs(&q(r1.value()).sub(&exact));
        assert!(dev <= q(r1.error()), "{what}: rung-1 bound violated");
    }
    if let Uncertain::Certain(s) = r1.sign() {
        assert_eq!(s, esign, "{what}: rung-1 sign");
        check_margin(r1.margin(), &exact, what);
    }

    // Rung 2.
    let r2 = p.eval(&lifted::<DdBound>(inputs));
    if r2.error().is_finite() && r2.hi().is_finite() {
        let v = q(r2.hi()).add(&q(r2.lo()));
        assert!(
            rabs(&v.sub(&exact)) <= q(r2.error()),
            "{what}: rung-2 bound violated"
        );
    }
    if let Uncertain::Certain(s) = r2.sign() {
        assert_eq!(s, esign, "{what}: rung-2 sign");
        check_margin(r2.margin(), &exact, what);
    }

    // Rung 3: exact whenever it did not ask to escalate.
    let r3 = p.eval(&lifted::<Exp32>(inputs));
    if !r3.must_escalate() {
        assert_eq!(exact_sum(r3.components()), exact, "{what}: rung-3 value");
        assert_eq!(r3.sign(), Some(esign), "{what}: rung-3 sign");
        let mags: Vec<f64> = r3.components().iter().map(|c| c.abs()).collect();
        assert!(
            mags.windows(2).all(|w| w[0] < w[1]),
            "{what}: not increasing"
        );
    } else {
        assert_eq!(r3.sign(), None, "{what}: flagged expansion has no sign");
    }

    // The generic driver, rung by rung and as a whole.
    for rung in [
        Rung::Bound,
        Rung::DoubleDouble,
        Rung::Expansion,
        Rung::Rational,
    ] {
        if let Some(a) = eval_rung(p, inputs, rung).expect("valid inputs") {
            assert_eq!(a.sign, esign, "{what}: eval_rung({rung:?}) sign");
            assert_eq!(a.rung, rung);
            check_margin(a.margin, &exact, what);
        } else {
            assert_ne!(rung, Rung::Rational, "{what}: rational rung always decides");
        }
    }
    let a = adaptive_sign(p, inputs).expect("valid inputs");
    assert_eq!(a.sign, esign, "{what}: adaptive_sign");
    check_margin(a.margin, &exact, what);
    if a.sign == Sign::Zero {
        assert_eq!(a.margin, 0.0, "{what}: zero margin");
    }
    // Starting higher can only land on the same sign at the same-or-higher rung.
    let b = adaptive_sign_from(p, inputs, Rung::Expansion).expect("valid inputs");
    assert_eq!(b.sign, esign, "{what}: adaptive_sign_from");
    assert!(b.rung >= Rung::Expansion);
}

// ---------------------------------------------------------------------------
// Input classes
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }
    /// Uniform in [-1, 1) with 53 significant bits.
    fn unit(&mut self) -> f64 {
        ((self.next() >> 11) as f64 / (1u64 << 52) as f64) - 1.0
    }
    fn full(&mut self) -> f64 {
        self.unit() * 2f64.powi(self.range(-8, 8) as i32)
    }
    fn dyadic(&mut self) -> f64 {
        self.range(-256, 256) as f64 / 8.0
    }
}

fn seeded(seed: u64) -> Rng {
    Rng(seed | 1)
}

fn arity(kind: Kind) -> usize {
    Pred(kind).arity()
}

fn random_inputs(kind: Kind, rng: &mut Rng) -> Vec<f64> {
    (0..arity(kind)).map(|_| rng.full()).collect()
}

fn integer_inputs(kind: Kind, rng: &mut Rng) -> Vec<f64> {
    (0..arity(kind))
        .map(|_| rng.range(-50, 50) as f64)
        .collect()
}

/// Dyadic coordinates with repeated values, so axis-aligned configurations
/// and exact zeros are common.
fn axis_inputs(kind: Kind, rng: &mut Rng) -> Vec<f64> {
    let pool: Vec<f64> = (0..3).map(|_| rng.dyadic()).collect();
    (0..arity(kind))
        .map(|_| pool[rng.below(3) as usize])
        .collect()
}

const CIRCLE: [(f64, f64); 12] = [
    (3.0, 4.0),
    (4.0, 3.0),
    (5.0, 0.0),
    (0.0, 5.0),
    (-3.0, 4.0),
    (-4.0, 3.0),
    (-5.0, 0.0),
    (0.0, -5.0),
    (3.0, -4.0),
    (4.0, -3.0),
    (-3.0, -4.0),
    (-4.0, -3.0),
];

/// Inputs whose exact predicate value is zero. `full` selects full-mantissa
/// data (zero only in exact arithmetic) over small dyadic data.
fn degenerate_inputs(kind: Kind, rng: &mut Rng, full: bool) -> Vec<f64> {
    let pt = |rng: &mut Rng| if full { rng.full() } else { rng.dyadic() };
    match kind {
        Kind::Orient2 => {
            let d = [pt(rng), pt(rng)];
            let (a, b, c) = if full {
                // Collinear with the origin through powers of two.
                let m = [0.0, 1.0, 2.0, 4.0, 8.0, -1.0, -2.0];
                let mut mm = || m[rng.below(m.len() as u64) as usize];
                (mm(), mm(), mm())
            } else {
                (
                    rng.range(-3, 3) as f64,
                    rng.range(-3, 3) as f64,
                    rng.range(-3, 3) as f64,
                )
            };
            let o = if full {
                [0.0, 0.0]
            } else {
                [rng.dyadic(), rng.dyadic()]
            };
            let at = |m: f64| [o[0] + m * d[0], o[1] + m * d[1]];
            let (pa, pb, pc) = (at(a), at(b), at(c));
            vec![pa[0], pa[1], pb[0], pb[1], pc[0], pc[1]]
        }
        Kind::Orient3 => {
            let a = [pt(rng), pt(rng), pt(rng)];
            let b = [pt(rng), pt(rng), pt(rng)];
            let (s, t) = (rng.range(-2, 2) as f64, rng.range(-2, 2) as f64);
            let mut c = [pt(rng), pt(rng), pt(rng)];
            let d: Vec<f64> = if full {
                // Coplanar with the origin through a power-of-two multiple.
                let m = 2f64.powi(rng.range(-2, 2) as i32);
                c = [m * a[0], m * a[1], m * a[2]];
                vec![0.0; 3]
            } else {
                (0..3)
                    .map(|k| a[k] + s * (b[k] - a[k]) + t * (c[k] - a[k]))
                    .collect()
            };
            // Predicate order: a, b, c, then the reference point (index 9..12).
            let mut v: Vec<f64> = a.iter().chain(&b).chain(&c).copied().collect();
            v.extend(d);
            v
        }
        Kind::InCircle => {
            let shift = [rng.dyadic(), rng.dyadic()];
            let k = rng.range(-3, 3) as i32;
            let mut idx = Vec::new();
            while idx.len() < 4 {
                let j = rng.below(12) as usize;
                if !idx.contains(&j) {
                    idx.push(j);
                }
            }
            idx.iter()
                .flat_map(|&j| {
                    let (x, y) = CIRCLE[j];
                    [x * 2f64.powi(k) + shift[0], y * 2f64.powi(k) + shift[1]]
                })
                .collect()
        }
        Kind::SumOfProducts => {
            let (a, b, c, d) = (pt(rng), pt(rng), pt(rng), pt(rng));
            vec![a, b, c, d, a, b, c, d]
        }
        Kind::Quartic => {
            let (a, b, e, f, g, h) = (pt(rng), pt(rng), pt(rng), pt(rng), pt(rng), pt(rng));
            vec![a, b, a, b, e, f, g, h]
        }
    }
}

fn nudge(x: f64, ulps: i64) -> f64 {
    let bits = x.to_bits() as i64;
    let moved = if x.is_sign_negative() {
        bits - ulps
    } else {
        bits + ulps
    };
    f64::from_bits(moved as u64)
}

/// Perturb one nonzero input by 1..=100 ulps.
fn perturb(mut v: Vec<f64>, rng: &mut Rng) -> Vec<f64> {
    let nz: Vec<usize> = (0..v.len()).filter(|&i| v[i] != 0.0).collect();
    if nz.is_empty() {
        return v;
    }
    let i = nz[rng.below(nz.len() as u64) as usize];
    let n = rng.range(1, 100);
    v[i] = nudge(v[i], if rng.below(2) == 0 { n } else { -n });
    v
}

fn scaled(v: &[f64], exp: i32) -> Option<Vec<f64>> {
    let out: Vec<f64> = v.iter().map(|x| x * 2f64.powi(exp)).collect();
    // Scaling must be exact (no subnormal rounding) and finite.
    let exact = out
        .iter()
        .zip(v)
        .all(|(o, x)| o.is_finite() && (*x == 0.0 || (*o != 0.0 && o / 2f64.powi(exp) == *x)));
    exact.then_some(out)
}

// ---------------------------------------------------------------------------
// Soundness properties (every rung against rational evaluation)
// ---------------------------------------------------------------------------

/// Cases per property; `RESOLVENT_RUNG_CASES` raises it for soak runs.
fn cases() -> u32 {
    std::env::var("RESOLVENT_RUNG_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn soundness_random_inputs(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            check_all(&Pred(kind), &random_inputs(kind, &mut rng), &format!("random {kind:?}"));
        }
    }

    #[test]
    fn soundness_integer_inputs(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            check_all(&Pred(kind), &integer_inputs(kind, &mut rng), &format!("integer {kind:?}"));
        }
    }

    #[test]
    fn soundness_axis_aligned_dyadic_inputs(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            check_all(&Pred(kind), &axis_inputs(kind, &mut rng), &format!("axis {kind:?}"));
        }
    }

    #[test]
    fn soundness_exact_zeros(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            for full in [false, true] {
                let v = degenerate_inputs(kind, &mut rng, full);
                let p = Pred(kind);
                prop_assert_eq!(ExactRing::sign(&exact_value(&p, &v)), Sign::Zero, "{:?} full={}", kind, full);
                check_all(&p, &v, &format!("zero {kind:?} full={full}"));
            }
        }
    }

    #[test]
    fn soundness_near_degenerate(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            for full in [false, true] {
                let v = perturb(degenerate_inputs(kind, &mut rng, full), &mut rng);
                check_all(&Pred(kind), &v, &format!("near-zero {kind:?} full={full}"));
            }
        }
    }

    /// Huge and tiny magnitudes must trip the guards without ever producing a
    /// wrong sign.
    #[test]
    fn soundness_extreme_magnitudes(seed in any::<u64>(), up in 0u32..2) {
        let mut rng = seeded(seed);
        for kind in KINDS {
            let full = rng.below(2) == 0;
            let base = if rng.below(2) == 0 {
                degenerate_inputs(kind, &mut rng, full)
            } else {
                perturb(degenerate_inputs(kind, &mut rng, true), &mut rng)
            };
            let k = rng.range(100, 1000) as i32;
            let exp = if up == 1 { k } else { -k };
            if let Some(v) = scaled(&base, exp) {
                check_all(&Pred(kind), &v, &format!("scaled 2^{exp} {kind:?}"));
            }
        }
    }
}

/// Random straight-line programs over wild inputs: operation-level soundness
/// of all three inexact rungs, including chains where bounds compound.
#[derive(Clone, Copy, Debug)]
enum Op {
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Neg(usize),
}

struct Program {
    inputs: usize,
    ops: Vec<Op>,
}

impl PolyPredicate for Program {
    fn arity(&self) -> usize {
        self.inputs
    }
    fn eval<N: RungArith>(&self, i: &[N]) -> N {
        let mut vals: Vec<N> = i.to_vec();
        for op in &self.ops {
            let v = match *op {
                Op::Add(a, b) => vals[a].add(&vals[b]),
                Op::Sub(a, b) => vals[a].sub(&vals[b]),
                Op::Mul(a, b) => vals[a].mul(&vals[b]),
                Op::Neg(a) => vals[a].neg(),
            };
            vals.push(v);
        }
        vals.pop().expect("nonempty")
    }
}

fn wild(rng: &mut Rng, pool: &[f64]) -> f64 {
    match rng.below(6) {
        0 => rng.unit() * 2f64.powi(rng.range(-1070, 960) as i32),
        1 => rng.unit() * 2f64.powi(rng.range(-60, 60) as i32),
        2 => nudge(
            pool[rng.below(pool.len() as u64) as usize],
            rng.range(-3, 3),
        ),
        3 => -pool[rng.below(pool.len() as u64) as usize],
        4 => 0.0,
        _ => rng.range(-9, 9) as f64,
    }
}

fn random_program(rng: &mut Rng) -> (Program, Vec<f64>) {
    let inputs = 4;
    let base: Vec<f64> = (0..3)
        .map(|_| rng.unit() * 2f64.powi(rng.range(-30, 30) as i32))
        .collect();
    let vals: Vec<f64> = (0..inputs).map(|_| wild(rng, &base)).collect();
    let mut ops = Vec::new();
    let n = rng.range(2, 9) as usize;
    for k in 0..n {
        let live = inputs + k;
        let (a, b) = (
            rng.below(live as u64) as usize,
            rng.below(live as u64) as usize,
        );
        // Keep the last op consuming the most recent value so work is not dead.
        let a = if k == n - 1 { live - 1 } else { a };
        ops.push(match rng.below(4) {
            0 => Op::Add(a, b),
            1 => Op::Sub(a, b),
            2 => Op::Mul(a, b),
            _ => Op::Neg(a),
        });
    }
    (Program { inputs, ops }, vals)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases() * 4))]

    #[test]
    fn soundness_random_programs(seed in any::<u64>()) {
        let mut rng = seeded(seed);
        let (prog, vals) = random_program(&mut rng);
        check_all(&prog, &vals, &format!("{:?} over {:?}", prog.ops, vals));
    }
}

// ---------------------------------------------------------------------------
// Targeted behaviour
// ---------------------------------------------------------------------------

#[test]
fn exact_zero_is_certified_by_the_bound_rung_when_every_operation_is_exact() {
    let p = Pred(Kind::Orient2);
    let a = adaptive_sign(&p, &[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]).unwrap();
    assert_eq!(a.sign, Sign::Zero);
    assert_eq!(a.rung, Rung::Bound);
    assert_eq!(a.margin, 0.0);
    let r1 = p.eval(&lifted::<RunningError>(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]));
    assert_eq!(r1.error(), 0.0);
    assert_eq!(r1.value(), 0.0);
}

#[test]
fn generic_cases_are_decided_at_rung_one() {
    let a = adaptive_sign(&Pred(Kind::Orient2), &[0.1, 0.2, 3.3, 0.7, 1.9, 4.4]).unwrap();
    assert_eq!(a.rung, Rung::Bound);
    assert_eq!(a.sign, Sign::Positive);
    assert!(a.margin > 1.0);
}

#[test]
fn full_mantissa_exact_zero_needs_the_expansion_rung() {
    let d = [0.7310585786300049_f64, 0.2689414213699951_f64];
    // Origin-collinear at multiples 1, 2, 4: all differences inexact.
    let v = [d[0], d[1], 2.0 * d[0], 2.0 * d[1], 4.0 * d[0], 4.0 * d[1]];
    let a = adaptive_sign(&Pred(Kind::Orient2), &v).unwrap();
    assert_eq!(a.sign, Sign::Zero);
    assert_eq!(a.rung, Rung::Expansion);
}

#[test]
fn every_rung_is_reachable_and_always_correct() {
    let mut seen = [false; 4];
    let mut rng = seeded(0x9E3779B97F4A7C15);
    for _ in 0..4000 {
        for kind in KINDS {
            let full = rng.below(2) == 0;
            let v = perturb(degenerate_inputs(kind, &mut rng, full), &mut rng);
            let p = Pred(kind);
            let a = adaptive_sign(&p, &v).unwrap();
            assert_eq!(a.sign, ExactRing::sign(&exact_value(&p, &v)));
            seen[a.rung as usize - 1] = true;
        }
    }
    // Rung 4 on a guard trip.
    let a = adaptive_sign(
        &Pred(Kind::Orient2),
        &[0.0, 0.0, 1e200, 1e200, 2e200, 2e200],
    )
    .unwrap();
    assert_eq!(a.rung, Rung::Rational);
    seen[3] = true;
    assert_eq!(seen, [true; 4], "rungs reached: {seen:?}");
}

#[test]
fn magnitude_guards_escalate_to_the_rational_rung() {
    let p = Pred(Kind::Orient2);
    // Product operands beyond ~1e140 -> rungs 1..3 refuse.
    let huge = [0.0, 0.0, 1e200, 2e200, 3e200, -1e200];
    for rung in [Rung::Bound, Rung::DoubleDouble, Rung::Expansion] {
        assert_eq!(eval_rung(&p, &huge, rung).unwrap(), None, "{rung:?}");
    }
    let a = adaptive_sign(&p, &huge).unwrap();
    assert_eq!((a.sign, a.rung), (Sign::Negative, Rung::Rational));

    // Sums beyond ~1e290 -> refuse (and 1.7e308 + 1.7e308 would overflow).
    let s = CancelSum(4);
    let big = [1.7e308, 1.7e308, -1.7e308, -1.7e308];
    for rung in [Rung::Bound, Rung::DoubleDouble, Rung::Expansion] {
        assert_eq!(eval_rung(&s, &big, rung).unwrap(), None, "{rung:?}");
    }
    assert_eq!(adaptive_sign(&s, &big).unwrap().sign, Sign::Zero);

    // Products below 2^-969 -> refuse; the true product is strictly positive
    // and a naive EFT would report (0, 0).
    let tiny = [0.0, 0.0, 1e-200, 0.0, 0.0, 1e-200];
    for rung in [Rung::Bound, Rung::DoubleDouble, Rung::Expansion] {
        assert_eq!(eval_rung(&p, &tiny, rung).unwrap(), None, "{rung:?}");
    }
    let a = adaptive_sign(&p, &tiny).unwrap();
    assert_eq!((a.sign, a.rung), (Sign::Positive, Rung::Rational));
    assert!(a.margin >= 0.0);
}

#[test]
fn expansion_capacity_overflow_escalates() {
    // Directly: three non-overlapping components do not fit in two slots.
    let a = StackExpansion::<2>::from_f64(1.0);
    let b = StackExpansion::<2>::from_f64(2f64.powi(-60));
    let c = StackExpansion::<2>::from_f64(2f64.powi(-120));
    let ok = a.add(&b);
    assert!(!ok.must_escalate());
    assert_eq!(ok.components(), &[2f64.powi(-60), 1.0]);
    let over = ok.add(&c);
    assert!(over.must_escalate());
    assert_eq!(over.sign(), None);
    // Flagged is sticky through every operation.
    assert!(over.add(&a).must_escalate());
    assert!(over.mul(&a).must_escalate());
    assert!(a.mul(&over).must_escalate());
    assert!(over.neg().must_escalate());
    assert!(over.sub(&over).must_escalate());

    // Through the driver: 35 full-mantissa terms 54 bits apart (none merge),
    // then their negations. The peak expansion needs 35 > 32 components
    // although the total is zero.
    let n = 35;
    let term = |i: i32| (1.0 + f64::EPSILON) * 2f64.powi(900 - 54 * i);
    let mut inputs: Vec<f64> = (0..n).map(term).collect();
    inputs.extend((0..n).map(|i| -term(i)));
    let p = CancelSum(2 * n as usize);
    assert_eq!(eval_rung(&p, &inputs, Rung::Expansion).unwrap(), None);
    let r = adaptive_sign(&p, &inputs).unwrap();
    assert_eq!((r.sign, r.rung), (Sign::Zero, Rung::Rational));
    check_all(&p, &inputs, "capacity");
}

#[test]
fn non_finite_inputs_and_wrong_arity_are_typed_errors() {
    let p = Pred(Kind::Orient2);
    assert_eq!(
        adaptive_sign(&p, &[0.0, 0.0, f64::NAN, 1.0, 2.0, 2.0]),
        Err(AdaptiveError::NonFiniteInput)
    );
    assert_eq!(
        adaptive_sign(&p, &[0.0, 0.0, f64::INFINITY, 1.0, 2.0, 2.0]),
        Err(AdaptiveError::NonFiniteInput)
    );
    assert_eq!(
        adaptive_sign(&p, &[0.0; 5]),
        Err(AdaptiveError::Arity {
            expected: 6,
            got: 5
        })
    );
}

#[test]
fn poisoned_values_never_certify() {
    let big = RunningError::lift(1e300);
    assert_eq!(big.sign(), Uncertain::Unknown);
    let z = RunningError::lift(0.0);
    assert_eq!(z.mul(&big).sign(), Uncertain::Unknown);
    let nan = RunningError::lift(f64::NAN);
    assert_eq!(nan.sign(), Uncertain::Unknown);
    assert_eq!(DdBound::lift(1e300).sign(), Uncertain::Unknown);
    assert_eq!(DdBound::lift(f64::INFINITY).sign(), Uncertain::Unknown);
    assert!(Exp32::lift(1e300).must_escalate());
    assert!(Exp32::lift(f64::NAN).must_escalate());
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

fn fingerprint(a: &AdaptiveSign) -> (Sign, Rung, u64) {
    (a.sign, a.rung, a.margin.to_bits())
}

fn corpus() -> Vec<(Kind, Vec<f64>)> {
    let mut rng = seeded(0xC0FFEE);
    let mut out = Vec::new();
    for _ in 0..60 {
        for kind in KINDS {
            let full = rng.below(2) == 0;
            out.push((kind, random_inputs(kind, &mut rng)));
            out.push((
                kind,
                perturb(degenerate_inputs(kind, &mut rng, full), &mut rng),
            ));
            out.push((kind, degenerate_inputs(kind, &mut rng, full)));
        }
    }
    out
}

#[test]
fn repeated_interleaved_and_threaded_calls_agree() {
    let cases = corpus();
    let run = |(k, v): &(Kind, Vec<f64>)| fingerprint(&adaptive_sign(&Pred(*k), v).unwrap());
    let first: Vec<_> = cases.iter().map(run).collect();
    // Repeated.
    let second: Vec<_> = cases.iter().map(run).collect();
    assert_eq!(first, second);
    // Interleaved in a different order, with unrelated calls in between.
    let mut order: Vec<usize> = (0..cases.len()).collect();
    order.reverse();
    for &i in &order {
        let _ = adaptive_sign(
            &Pred(Kind::Quartic),
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        );
        assert_eq!(run(&cases[i]), first[i]);
    }
    // Concurrent.
    std::thread::scope(|s| {
        let hs: Vec<_> = (0..4)
            .map(|_| s.spawn(|| cases.iter().map(run).collect::<Vec<_>>()))
            .collect();
        for h in hs {
            assert_eq!(h.join().unwrap(), first);
        }
    });
}

/// Fixed answers recorded from this implementation. The same file is run with
/// and without `-C target-feature=+fma`; the rungs use the Dekker product and
/// never `mul_add`, so the table must hold in both.
#[test]
fn golden_results_are_build_flag_independent() {
    let g = |kind, v: &[f64]| fingerprint(&adaptive_sign(&Pred(kind), v).unwrap());
    let d = [0.7310585786300049_f64, 0.2689414213699951_f64];
    let results = [
        g(Kind::Orient2, &[0.1, 0.2, 3.3, 0.7, 1.9, 4.4]),
        g(
            Kind::Orient2,
            &[d[0], d[1], 2.0 * d[0], 2.0 * d[1], 4.0 * d[0], 4.0 * d[1]],
        ),
        g(Kind::Orient2, &[0.0, 0.0, 1.0, 1.0, 2.0, 2.0]),
        g(Kind::Orient2, &[0.0, 0.0, 1e200, 2e200, 3e200, -1e200]),
        g(Kind::Orient2, &[0.5, 0.5, 12.0, 12.0, 24.0, nudge(24.0, 3)]),
        g(Kind::Quartic, &[0.1, 0.3, 0.1, 0.3, 0.7, 0.9, 0.2, 0.4]),
    ];
    let golden: [(Sign, Rung, u64); 6] = GOLDEN;
    assert_eq!(results, golden, "recorded: {results:?}");
}

const GOLDEN: [(Sign, Rung, u64); 6] = [
    (Sign::Positive, Rung::Bound, 4623249010470661648),
    (Sign::Zero, Rung::Expansion, 0),
    (Sign::Zero, Rung::Bound, 0),
    (Sign::Negative, Rung::Rational, 9218868437227405311),
    (Sign::Positive, Rung::Bound, 4412823947381309434),
    (Sign::Zero, Rung::Expansion, 0),
];
