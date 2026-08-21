//! `f64` orientation tests can report "exactly collinear" for points that are
//! not collinear at all. This runs one such case — found by brute-force
//! search over random point triples, not hand-picked — through plain `f64`
//! arithmetic and through `resolvent::Rational`, and prints both answers.
//!
//! Run: `cargo run -p resolvent --example why_floats_lie`

use resolvent::Rational;
use resolvent::exact::{ExactRing, RingOps};

/// The textbook orientation predicate: sign of the cross product
/// `(b - a) x (c - a)`. Positive = c is left of the line a->b, negative =
/// right, zero = exactly collinear.
fn orient_f64(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
}

/// Exact ingress for this demo's coordinates.
///
/// `Rational::from_f64` admits exactly the finite doubles, and every value
/// this example feeds it is a finite literal from `main` — so the `expect`
/// cannot fire. It stays visible rather than becoming a silent default
/// because "`f64` → exact is a *checked* conversion" is half the point being
/// demonstrated.
#[allow(clippy::expect_used)] // demo literals are finite by inspection
fn q(x: f64) -> Rational {
    Rational::from_f64(x).expect("why_floats_lie: coordinates are finite doubles")
}

fn orient_exact(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> Rational {
    let (ax, ay, bx, by, cx, cy) = (q(ax), q(ay), q(bx), q(by), q(cx), q(cy));
    let dx = bx.sub(&ax);
    let dy = by.sub(&ay);
    let px = cx.sub(&ax);
    let py = cy.sub(&ay);
    dx.mul(&py).sub(&dy.mul(&px))
}

fn main() {
    // a, b, c: c was constructed to lie exactly on the line through a and b
    // in real-number math (c = a + t*(b - a) for a random t), then rounded to
    // the nearest f64. Rounding is all it takes.
    let (ax, ay) = (-3089173.0, 6656906.0);
    let (bx, by) = (7841570.0, 4347616.0);
    let (cx, cy) = (5406514.99152003, 4862059.362224572);

    let f = orient_f64(ax, ay, bx, by, cx, cy);
    let exact = orient_exact(ax, ay, bx, by, cx, cy);

    println!("a = ({ax}, {ay})");
    println!("b = ({bx}, {by})");
    println!("c = ({cx}, {cy})");
    println!();
    println!("f64 cross product:    {f}");
    println!(
        "exact (Rational):     {exact:?}  (sign: {:?})",
        exact.sign()
    );
    println!();
    if f == 0.0 {
        println!(
            "f64 says c is exactly on the line a-b. It is not — the exact \
             computation says otherwise. An arrangement built on the f64 \
             answer would treat this as a degenerate collinear case (merge \
             an edge, drop a vertex, skip a split) instead of the ordinary \
             non-degenerate crossing it actually is."
        );
    }
}
