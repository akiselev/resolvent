//! Proof-of-concept for the `Scalar` seam, end to end.
//!
//! ONE generic scalar-elliptic kernel — 1-D Poisson `−c u'' = f` on `[0,1]`,
//! `u(0)=u(1)=0`, P1 (linear) finite elements — is assembled and solved by
//! Gaussian elimination written a **single time** over `S: Scalar`, then run
//! with BOTH scalar tiers:
//!
//! * `S = f64`   → the fast tier; matches the reference float result.
//! * `S = Real`  → the certified-exact tier; the residual is provably zero.
//!
//! The `∫ c u' v'` weak form is the 1-D instance of the scalar-elliptic kernel
//! the ecosystem re-derives across electrostatics / magnetostatics / steady
//! heat (`residua::assemble_stiffness_coeff`); this PoC shows that one seam
//! carries such a kernel across both tiers without forking the code.
//!
//! ## Why the answer exposes the difference
//!
//! For constant `c, f` the P1 Galerkin solution is *nodally exact*:
//! `u(x_i) = (f / 2c) · x_i · (1 − x_i)`. With `c = f = 1` and `n = 3`
//! elements the interior nodal values are `u₁ = u₂ = 1/9` — a **non-dyadic**
//! rational, so `f64` cannot even represent it, while `Real` carries it exactly
//! and certifies the residual as identically zero.

use resolvent::{Rational, Real, Scalar, Sign};

// The generic scalar-elliptic kernel is written ONCE over `S: Scalar` in the
// shared `generic_elliptic` module and instantiated here at `f64`/`Real` (and,
// in `dual_exact_poc.rs`, at `Dual<f64>`/`Dual<Real>`).
mod generic_elliptic;
use generic_elliptic::{assemble_and_solve, maxnorm};

type R = Real<Rational>;

// ===========================================================================
// (a) f64 tier: matches the reference float result.
// ===========================================================================

#[test]
fn f64_tier_matches_reference() {
    let (u, residual) = assemble_and_solve::<f64>(3, 1.0, 1.0);
    assert_eq!(u.len(), 2);

    // Reference: the analytic nodal values 1/9, computed independently.
    let reference = 1.0f64 / 9.0;
    for &ui in &u {
        assert!(
            (ui - reference).abs() < 1e-12,
            "f64 solution matches float reference"
        );
    }

    // The residual is *small* — but only approximately zero (float roundoff).
    let r = maxnorm(&residual);
    assert!(r < 1e-12, "f64 residual is small: {r}");

    // f64 cannot even REPRESENT the exact answer 1/9: this is the gap the
    // exact tier closes.
    assert_ne!(
        Rational::from_f64(reference).unwrap(),
        Rational::from_ratio(1, 9),
        "1/9 is non-dyadic — f64 only rounds it"
    );

    println!("f64  interior u = {u:?}   residual max-norm = {r:e}");
}

// ===========================================================================
// (b) Real tier: a certified-exact result from the SAME generic code.
// ===========================================================================

#[test]
fn real_tier_is_certified_exact() {
    let one = <R as Scalar>::one();
    let (u, residual) = assemble_and_solve::<R>(3, one.clone(), one);
    assert_eq!(u.len(), 2);

    // The solution is EXACTLY 1/9 at every interior node (certified equality —
    // Real's `==` forces the exact rung when the interval filter cannot decide).
    let exact_ninth = <R as Scalar>::from_ratio(1, 9);
    for ui in &u {
        assert_eq!(*ui, exact_ninth, "Real solution is exactly 1/9");
    }

    // The residual is CERTIFIED identically zero: every component's exact sign
    // is Zero — not merely small, provably nothing.
    for ri in &residual {
        assert_eq!(
            ri.sign(),
            Sign::Zero,
            "Real residual is certified-exact zero"
        );
    }
    assert_eq!(maxnorm(&residual).sign(), Sign::Zero);

    // And the two tiers agree numerically (one generic codebase).
    for ui in &u {
        assert!((ui.to_f64() - 1.0 / 9.0).abs() < 1e-12);
    }

    let approx: Vec<f64> = u.iter().map(Scalar::to_f64).collect();
    println!("Real interior u ≈ {approx:?}   (exact 1/9, residual sign = Zero)");
}

// ===========================================================================
// Both tiers, side by side: same kernel, numerically agreeing.
// ===========================================================================

#[test]
fn tiers_agree_on_a_larger_mesh() {
    // n = 8 elements (power of two → dyadic coordinates), constant c = 2, f = 3.
    let (u_f64, res_f64) = assemble_and_solve::<f64>(8, 2.0, 3.0);
    let (u_real, res_real) =
        assemble_and_solve::<R>(8, <R as Scalar>::from_i32(2), <R as Scalar>::from_i32(3));

    assert_eq!(u_f64.len(), 7);
    assert_eq!(u_real.len(), 7);

    // The exact tier certifies a zero residual...
    for ri in &res_real {
        assert_eq!(ri.sign(), Sign::Zero);
    }
    // ...the float tier only approximates it...
    assert!(maxnorm(&res_f64) < 1e-12);

    // ...and the two solutions agree node-for-node.
    for (uf, ur) in u_f64.iter().zip(&u_real) {
        assert!((uf - ur.to_f64()).abs() < 1e-12);
    }
}
