//! A genuinely hard predicate, decided at three certified tiers: does a
//! sphere and a cylinder intersect in a smooth quartic curve?
//!
//! The full answer (smooth transversal quartic / tangent / reducible /
//! degenerate) requires classifying every root of the pencil
//! `det(Q_sphere + λ·Q_cylinder)`, an expensive general-purpose computation.
//! But the *generic* case — by far the most common one in practice — has a
//! shortcut: the pencil determinant for a sphere/cylinder pair reduces to a
//! closed-form quartic in `λ`, and the intersection is a smooth transversal
//! quartic **iff that quartic is square-free iff its discriminant is
//! nonzero**. So instead of classifying roots, decide one polynomial sign.
//!
//! That sign is still a real predicate — the discriminant of a general
//! quartic is a degree-4-in-the-coefficients polynomial with 16 terms — so
//! it gets the same certify-or-escalate treatment as any other predicate in
//! this crate: try a fast [`Interval`] evaluation first, and only pay for
//! exact [`Rational`] arithmetic when the interval can't already prove the
//! sign. When the *exact* discriminant is genuinely zero, that's not a
//! failure — it's a real non-generic configuration (tangency, coincident
//! axes, ...) that needs a different, more careful classifier than "one
//! polynomial sign," which is out of scope for this example.
//!
//! Run: `cargo run -p resolvent --example sphere_cylinder_classification`

use resolvent::exact::{ExactRing, RingOps};
use resolvent::{Interval, Rational, Sign};

#[derive(Clone, Copy)]
struct Sphere {
    center: [f64; 3],
    radius: f64,
}

#[derive(Clone, Copy)]
struct Cylinder {
    /// A point on the axis.
    origin: [f64; 3],
    /// The axis direction. Need not be unit length — the formula below is
    /// scale-invariant in it.
    axis: [f64; 3],
    radius: f64,
}

fn dot<F: RingOps>(u: &[F; 3], v: &[F; 3]) -> F {
    u[0].mul(&v[0]).add(&u[1].mul(&v[1])).add(&u[2].mul(&v[2]))
}

fn sq<F: RingOps>(x: &F) -> F {
    x.mul(x)
}

/// The finite characteristic-polynomial coefficients `[c0..c4]` of
/// `det(Q_sphere + lambda * Q_cylinder)`. One generic-`F` program; the tiers
/// below differ only in which `F` they instantiate it at.
///
/// This closed form exists because, for a sphere `Q1` (rank 4) and a
/// cylinder `Q2` (rank 3), the pencil determinant is a bordered determinant
/// of a rank-1 update of a scalar matrix, which factors as
/// `(1 + s*lambda) * (k0 + k1*lambda + k2*lambda^2)` — a degree-2 polynomial
/// times a fixed linear factor, giving a degree-4 result with `c4` always 0
/// (the cylinder's rank-3 signature is a root at infinity).
fn char_poly_coeffs<F: RingOps + Clone>(
    c: &[F; 3],
    o: &[F; 3],
    a: &[F; 3],
    r_sph: &F,
    r_cyl: &F,
) -> [F; 5] {
    let d: [F; 3] = [o[0].sub(&c[0]), o[1].sub(&c[1]), o[2].sub(&c[2])];
    let s = dot(a, a);
    let dd = dot(&d, &d);
    let ad = dot(a, &d);
    let r2_sph = sq(r_sph);
    let r2_cyl = sq(r_cyl);
    let zero = F::from_i32(0);

    let k0 = zero.sub(&r2_sph);
    let k1 = s.mul(&dd.sub(&r2_sph).sub(&r2_cyl)).sub(&sq(&ad));
    let k2 = zero.sub(&r2_cyl.mul(&sq(&s)));

    [
        k0.clone(),
        k1.add(&s.mul(&k0)),
        k2.add(&s.mul(&k1)),
        s.mul(&k2),
        F::from_i32(0),
    ]
}

/// The discriminant of the quartic `c4*x^4 + c3*x^3 + c2*x^2 + c1*x + c0`:
/// the standard 16-term closed form. Nonzero iff the quartic is square-free.
fn quartic_discriminant<F: RingOps + Clone>(coeffs: &[F; 5]) -> F {
    let (a, b, c, d, e) = (&coeffs[4], &coeffs[3], &coeffs[2], &coeffs[1], &coeffs[0]);
    let i = F::from_i32;
    let (a2, a3) = (sq(a), sq(a).mul(a));
    let (b2, b3, b4) = (sq(b), sq(b).mul(b), sq(&sq(b)));
    let (c2, c3, c4) = (sq(c), sq(c).mul(c), sq(&sq(c)));
    let (d2, d3, d4) = (sq(d), sq(d).mul(d), sq(&sq(d)));
    let (e2, e3) = (sq(e), sq(e).mul(e));

    let terms: [F; 16] = [
        i(256).mul(&a3).mul(&e3),
        i(-192).mul(&a2).mul(b).mul(d).mul(&e2),
        i(-128).mul(&a2).mul(&c2).mul(&e2),
        i(144).mul(&a2).mul(c).mul(&d2).mul(e),
        i(-27).mul(&a2).mul(&d4),
        i(144).mul(a).mul(&b2).mul(c).mul(&e2),
        i(-6).mul(a).mul(&b2).mul(&d2).mul(e),
        i(-80).mul(a).mul(b).mul(&c2).mul(d).mul(e),
        i(18).mul(a).mul(b).mul(c).mul(&d3),
        i(16).mul(a).mul(&c4).mul(e),
        i(-4).mul(a).mul(&c3).mul(&d2),
        i(-27).mul(&b4).mul(&e2),
        i(18).mul(&b3).mul(c).mul(d).mul(e),
        i(-4).mul(&b3).mul(&d3),
        i(-4).mul(&b2).mul(&c3).mul(e),
        b2.mul(&c2).mul(&d2),
    ];
    terms
        .into_iter()
        .reduce(|acc, t| acc.add(&t))
        .unwrap_or_else(|| F::from_i32(0))
}

fn f64s_to_interval(v: &[f64; 3]) -> [Interval; 3] {
    v.map(Interval::point)
}

/// Exact ingress for this demo's inputs.
///
/// `Rational::from_f64` admits exactly the finite doubles; every value this
/// example feeds it is a finite literal from the corpus below, so the
/// `expect` cannot fire.
#[allow(clippy::expect_used)] // demo inputs are finite by inspection
fn q(x: f64) -> Rational {
    Rational::from_f64(x).expect("sphere_cylinder: inputs are finite doubles")
}

fn f64s_to_rational(v: &[f64; 3]) -> [Rational; 3] {
    v.map(q)
}

/// `Some(true)`/`Some(false)`: certified smooth / certified non-generic, at
/// the cheap interval rung. `None`: the interval straddles zero, escalate.
fn classify_interval(sphere: &Sphere, cylinder: &Cylinder) -> Option<bool> {
    let c = f64s_to_interval(&sphere.center);
    let o = f64s_to_interval(&cylinder.origin);
    let a = f64s_to_interval(&cylinder.axis);
    let coeffs = char_poly_coeffs(
        &c,
        &o,
        &a,
        &Interval::point(sphere.radius),
        &Interval::point(cylinder.radius),
    );
    let disc = quartic_discriminant(&coeffs);
    if disc.contains_zero() {
        None
    } else {
        Some(true)
    }
}

/// The exact rung: the true sign of the discriminant, with no chance of a
/// false "straddle."
fn classify_exact(sphere: &Sphere, cylinder: &Cylinder) -> Sign {
    let c = f64s_to_rational(&sphere.center);
    let o = f64s_to_rational(&cylinder.origin);
    let a = f64s_to_rational(&cylinder.axis);
    let coeffs = char_poly_coeffs(&c, &o, &a, &q(sphere.radius), &q(cylinder.radius));
    quartic_discriminant(&coeffs).sign()
}

fn classify(sphere: &Sphere, cylinder: &Cylinder) -> (&'static str, &'static str) {
    match classify_interval(sphere, cylinder) {
        Some(true) => ("smooth transversal quartic", "interval (cheap)"),
        _ => match classify_exact(sphere, cylinder) {
            Sign::Zero => (
                "non-generic (tangent / coincident-axis / degenerate) \u{2014} needs a real classifier, not just a sign",
                "exact (escalated)",
            ),
            _ => ("smooth transversal quartic", "exact (escalated)"),
        },
    }
}

fn main() {
    let cases = [
        (
            "off-axis sphere pierced by a thinner cylinder",
            Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 2.0,
            },
            Cylinder {
                origin: [0.5, 0.0, 0.0],
                axis: [0.0, 0.0, 1.0],
                radius: 1.0,
            },
        ),
        (
            "coaxial, equal radius (touches along a tangent circle)",
            Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
            },
            Cylinder {
                origin: [0.0, 0.0, 0.0],
                axis: [0.0, 0.0, 1.0],
                radius: 1.0,
            },
        ),
    ];

    for (label, sphere, cylinder) in &cases {
        let (verdict, tier) = classify(sphere, cylinder);
        println!("{label}:\n  verdict: {verdict}\n  decided at: {tier}\n");
    }

    // Self-check against known-good coefficients (sphere r=2 at the origin,
    // cylinder r=1, axis z, at (0.5, 0, 0)): a hand-derivable closed form in
    // the cylinder frame.
    let coeffs = char_poly_coeffs(
        &f64s_to_rational(&[0.0, 0.0, 0.0]),
        &f64s_to_rational(&[0.5, 0.0, 0.0]),
        &f64s_to_rational(&[0.0, 0.0, 1.0]),
        &q(2.0),
        &q(1.0),
    );
    let expect = [-4.0, -8.75, -5.75, -1.0, 0.0];
    for (k, e) in expect.iter().enumerate() {
        assert_eq!(coeffs[k], q(*e), "coefficient {k}");
    }
    println!("self-check: characteristic-polynomial coefficients match the known closed form.");
}
