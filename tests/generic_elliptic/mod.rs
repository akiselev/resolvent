//! The generic scalar-elliptic kernel — written **once** over `S: Scalar`, then
//! instantiated by the sibling PoCs at every tier of the seam:
//!
//! * `exact_generic_poc.rs` runs it at `f64` and `Real`,
//! * `dual_exact_poc.rs`    runs it at `Dual<f64>` and `Dual<Real>`.
//!
//! ONE codebase, four instantiations. This is a shared test-support module (a
//! `tests/<dir>/mod.rs`, not compiled as its own test binary), so `dead_code` is
//! allowed — not every consumer exercises every helper.
#![allow(dead_code)]

use resolvent::Scalar;

/// Assemble the interior stiffness matrix `K` and load vector `b` for
/// `−c u'' = f`, `u(0)=u(1)=0`, on a uniform P1 mesh of `n` elements.
///
/// Element stiffness (length `h = 1/n`): `(c/h)·[[1,−1],[−1,1]]`. Consistent P1
/// load for constant source `f`: `b_i = f·h` at each interior node. Dirichlet
/// endpoints are condensed out, leaving the `n−1` interior DOFs.
pub fn assemble<S: Scalar>(n: usize, c: S, f: S) -> (Vec<Vec<S>>, Vec<S>) {
    let h = S::one() / S::from_i32(n as i32);
    let k = c / h.clone(); // c/h
    let two_k = k.clone() + k.clone();
    let neg_k = S::zero() - k.clone();

    let m = n - 1; // interior DOFs
    let mut a = vec![vec![S::zero(); m]; m];
    for i in 0..m {
        a[i][i] = two_k.clone();
        if i + 1 < m {
            a[i][i + 1] = neg_k.clone();
            a[i + 1][i] = neg_k.clone();
        }
    }
    let b = vec![f * h; m];
    (a, b)
}

/// Dense Gaussian elimination, no pivoting (the interior `K` is SPD /
/// diagonally dominant, so the diagonal pivots are safely nonzero). Solves
/// `A x = b`, returning `x`. Exact-closed: only `+ − × ÷` are used.
pub fn gaussian_solve<S: Scalar>(mut a: Vec<Vec<S>>, mut b: Vec<S>) -> Vec<S> {
    let m = b.len();
    // Forward elimination.
    for p in 0..m {
        let piv = a[p][p].clone();
        // Split off the pivot row so the target rows below it borrow disjointly.
        let (top, bottom) = a.split_at_mut(p + 1);
        let row_p = &top[p];
        for (off, row_i) in bottom.iter_mut().enumerate() {
            let i = p + 1 + off;
            let factor = row_i[p].clone() / piv.clone();
            for (aij, apj) in row_i.iter_mut().zip(row_p).skip(p) {
                *aij = aij.clone() - factor.clone() * apj.clone();
            }
            b[i] = b[i].clone() - factor.clone() * b[p].clone();
        }
    }
    // Back substitution.
    let mut x = vec![S::zero(); m];
    for i in (0..m).rev() {
        let mut acc = b[i].clone();
        for j in (i + 1)..m {
            acc = acc - a[i][j].clone() * x[j].clone();
        }
        x[i] = acc / a[i][i].clone();
    }
    x
}

/// `A·x` (dense), generic.
pub fn matvec<S: Scalar>(a: &[Vec<S>], x: &[S]) -> Vec<S> {
    a.iter()
        .map(|row| {
            let mut acc = S::zero();
            for (aij, xj) in row.iter().zip(x) {
                acc = acc + aij.clone() * xj.clone();
            }
            acc
        })
        .collect()
}

/// `max_i |v_i|` — exercises the comparison + `abs` surface of the seam.
pub fn maxnorm<S: Scalar>(v: &[S]) -> S {
    let mut m = S::zero();
    for x in v {
        let a = x.abs();
        if a > m {
            m = a;
        }
    }
    m
}

/// The whole pipeline: assemble, solve, return `(solution, residual = A·u − b)`.
pub fn assemble_and_solve<S: Scalar>(n: usize, c: S, f: S) -> (Vec<S>, Vec<S>) {
    let (a, b) = assemble(n, c.clone(), f.clone());
    let u = gaussian_solve(a.clone(), b.clone());
    let au = matvec(&a, &u);
    let residual: Vec<S> = au
        .into_iter()
        .zip(&b)
        .map(|(ai, bi)| ai - bi.clone())
        .collect();
    (u, residual)
}
