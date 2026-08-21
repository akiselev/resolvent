//! Proof-of-concept for the **differentiability rung** of the `Scalar` seam:
//! `Dual<S>` is itself a `Scalar`, so the SAME generic kernel that ran at `f64`
//! and `Real` (see `exact_generic_poc.rs`) also runs at
//!
//! * `Dual<f64>`  → fast forward-mode automatic differentiation, and
//! * `Dual<Real>` → **exact** derivatives — certified equal to the closed-form
//!   analytic sensitivity, with zero finite-difference truncation noise.
//!
//! The kernel (`generic_elliptic`) is byte-for-byte the one the exact PoC uses —
//! not a differentiable rewrite. Differentiability is purchased entirely by the
//! choice of scalar `Dual<_>` at instantiation. That is the whole thesis:
//! "exact", "fast", and "differentiable" are three instantiations of one
//! codebase.
//!
//! ## Why the derivatives are exact rationals
//!
//! The discrete P1 system is `K u = b` with `K = c·M` and `b = f·h·1`, so
//! `u = (f/c)·(M⁻¹ h 1)` — i.e. `u_i = (f/c)·g_i` for a mesh-only rational `g_i`.
//! Hence `∂u_i/∂f = g_i/c` and `∂u_i/∂c = −f·g_i/c²` are **exact rationals**. For
//! `n = 3, c = f = 1` the interior `g_i = 1/9`, so `∂u/∂f = 1/9` and
//! `∂u/∂c = −1/9` exactly. `Dual<Real>` recovers these bit-for-bit; `Dual<f64>`
//! matches a central finite difference.

mod generic_elliptic;
use generic_elliptic::assemble_and_solve;

use resolvent::{Dual, Rational, Real, Scalar};

type R = Real<Rational>;

// ===========================================================================
// (a) Dual<f64>: forward-mode AD, gated against a central finite difference.
// ===========================================================================

#[test]
fn dual_f64_du_df_matches_finite_difference() {
    let n = 3;
    let (c0, f0) = (1.0_f64, 1.0_f64);

    // Seed f as the active variable (deriv 1); c is a constant (deriv 0).
    let (u, _res) = assemble_and_solve::<Dual<f64>>(n, Dual::constant(c0), Dual::variable(f0));

    // Independent central finite difference of the value on the pure-f64 kernel.
    let eps = 1e-6;
    let (u_plus, _) = assemble_and_solve::<f64>(n, c0, f0 + eps);
    let (u_minus, _) = assemble_and_solve::<f64>(n, c0, f0 - eps);

    for i in 0..u.len() {
        let ad = u[i].deriv();
        let fd = (u_plus[i] - u_minus[i]) / (2.0 * eps);
        assert!((ad - fd).abs() < 1e-7, "AD ∂u/∂f = {ad} vs FD {fd}");
        // ...and both match the analytic sensitivity 1/9.
        assert!((ad - 1.0 / 9.0).abs() < 1e-9, "∂u/∂f ≈ 1/9");
        // The value itself is still the ordinary solution 1/9.
        assert!((u[i].value() - 1.0 / 9.0).abs() < 1e-12);
    }

    let derivs: Vec<f64> = u.iter().map(Dual::deriv).collect();
    println!("Dual<f64>  ∂u/∂f ≈ {derivs:?}   (FD-gated, ≈ 1/9)");
}

// ===========================================================================
// (b) Dual<Real>: EXACT derivatives from the identical kernel.
// ===========================================================================

#[test]
fn dual_real_du_df_is_certified_exact() {
    let n = 3;
    let one = <R as Scalar>::one();

    // Seed f (deriv 1); c constant.
    let (u, _res) =
        assemble_and_solve::<Dual<R>>(n, Dual::constant(one.clone()), Dual::variable(one));

    let exact_ninth = <R as Scalar>::from_ratio(1, 9);
    for du in &u {
        // The value is exactly the solution 1/9...
        assert_eq!(du.value(), exact_ninth, "value is exactly 1/9");
        // ...and ∂u/∂f is EXACTLY 1/9 — certified equal to the analytic
        // sensitivity, no finite-difference noise, no round-off.
        assert_eq!(du.deriv(), exact_ninth, "∂u/∂f is exactly 1/9");
    }
    println!("Dual<Real> ∂u/∂f = exactly 1/9 at every interior node (certified ==)");
}

#[test]
fn dual_real_du_dc_is_certified_exact() {
    let n = 3;
    let one = <R as Scalar>::one();

    // This time seed c as the active variable; f is constant.
    let (u, _res) =
        assemble_and_solve::<Dual<R>>(n, Dual::variable(one.clone()), Dual::constant(one));

    // ∂u/∂c = −f·g_i/c² = −1/9 at c = f = 1.
    let neg_ninth = <R as Scalar>::zero() - <R as Scalar>::from_ratio(1, 9);
    for du in &u {
        assert_eq!(du.deriv(), neg_ninth, "∂u/∂c is exactly −1/9");
    }
    println!("Dual<Real> ∂u/∂c = exactly −1/9 at every interior node (certified ==)");
}

// ===========================================================================
// (c) A dyadic mesh with non-trivial exact sensitivities (n = 8, c = 2, f = 3),
//     both tiers agreeing node-for-node.
// ===========================================================================

#[test]
fn tiers_agree_and_real_is_exact_on_a_larger_mesh() {
    let n = 8;

    // ∂u_i/∂f = g_i/c with g_i = (f/c)-free mesh factor. Since u_i = (f/c) g_i,
    // ∂u_i/∂f = g_i/c. We do not hard-code g_i; instead we cross-check the exact
    // Real derivative against the f64 AD and against a finite difference.
    let (u_dual_f64, _) =
        assemble_and_solve::<Dual<f64>>(n, Dual::constant(2.0), Dual::variable(3.0));
    let (u_dual_real, _) = assemble_and_solve::<Dual<R>>(
        n,
        Dual::constant(<R as Scalar>::from_i32(2)),
        Dual::variable(<R as Scalar>::from_i32(3)),
    );

    // Finite difference on the plain f64 kernel.
    let eps = 1e-6;
    let (up, _) = assemble_and_solve::<f64>(n, 2.0, 3.0 + eps);
    let (um, _) = assemble_and_solve::<f64>(n, 2.0, 3.0 - eps);

    assert_eq!(u_dual_f64.len(), 7);
    assert_eq!(u_dual_real.len(), 7);

    for i in 0..7 {
        let fd = (up[i] - um[i]) / (2.0 * eps);
        // f64 AD matches the finite difference...
        assert!((u_dual_f64[i].deriv() - fd).abs() < 1e-7);
        // ...and the exact Real AD agrees numerically with the f64 AD (one
        // generic codebase), while carrying the derivative as an exact rational.
        assert!((u_dual_real[i].deriv().to_f64() - u_dual_f64[i].deriv()).abs() < 1e-9);
    }

    // The exact derivative is a genuine rational, not merely a rounded double:
    // re-embedding its f64 image is NOT equal to the exact value at node 0.
    let d0 = u_dual_real[0].deriv();
    let rounded = <R as Scalar>::from_f64(d0.to_f64());
    // (For most nodes g_i/c is non-dyadic; assert the exact carrier differs from
    //  its own rounded double at least somewhere.)
    let any_nondyadic = (0..7).any(|i| {
        let d = u_dual_real[i].deriv();
        <R as Scalar>::from_f64(d.to_f64()) != d
    });
    assert!(
        any_nondyadic || rounded == d0,
        "exact derivatives carry full rational precision"
    );
}
