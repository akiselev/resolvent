//! Dense exact rational linear algebra, sized at runtime.
//!
//! Textbook Gaussian elimination over [`Rational`], no pivoting strategy
//! beyond "first nonzero" and no blocking: the consumers are 3x3 and 4x4
//! forms (quadric coefficient matrices and their conic restrictions), where
//! the cost is entirely in the coefficient growth rather than in the flop
//! count.
//!
//! The load-bearing routine is [`Mat::congruence_diag`] — Lagrange congruence
//! diagonalisation of a symmetric form. It yields **rank and Sylvester
//! inertia together**, over ℚ, with no eigenvalue computation and no algebraic
//! numbers, which is what lets a real classification of a pencil of quadrics
//! be exact and cheap at the same time.

use crate::exact::{ExactField, ExactRing, Rational, RingOps};
use crate::uncertain::Sign;

fn is0(x: &Rational) -> bool {
    x.sign() == Sign::Zero
}

/// A dense `rows x cols` exact rational matrix, row-major.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mat {
    /// Row count.
    pub rows: usize,
    /// Column count.
    pub cols: usize,
    /// Entries, row-major (`a[i*cols + j]`).
    pub a: Vec<Rational>,
}

impl Mat {
    /// The `rows x cols` zero matrix.
    pub fn zeros(rows: usize, cols: usize) -> Mat {
        Mat {
            rows,
            cols,
            a: vec![Rational::zero(); rows * cols],
        }
    }

    /// The `n x n` identity.
    pub fn ident(n: usize) -> Mat {
        let mut m = Mat::zeros(n, n);
        for i in 0..n {
            m.set(i, i, Rational::one());
        }
        m
    }

    /// From equal-length rows. Empty input gives the 0x0 matrix.
    pub fn from_rows(rows: &[Vec<Rational>]) -> Mat {
        let nr = rows.len();
        let nc = rows.first().map_or(0, Vec::len);
        let mut m = Mat::zeros(nr, nc);
        for (i, row) in rows.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                m.set(i, j, v.clone());
            }
        }
        m
    }

    /// The `n x n` matrix whose columns are `cols` (each of length `n`).
    pub fn from_cols(n: usize, cols: &[Vec<Rational>]) -> Mat {
        let mut m = Mat::zeros(n, cols.len());
        for (j, c) in cols.iter().enumerate() {
            for (i, x) in c.iter().enumerate() {
                m.set(i, j, x.clone());
            }
        }
        m
    }

    /// Entry `(i, j)`.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> &Rational {
        &self.a[i * self.cols + j]
    }

    /// Set entry `(i, j)`.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, v: Rational) {
        self.a[i * self.cols + j] = v;
    }

    /// Transpose.
    pub fn transpose(&self) -> Mat {
        let mut m = Mat::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                m.set(j, i, self.get(i, j).clone());
            }
        }
        m
    }

    /// Matrix product. Panics on a shape mismatch.
    pub fn matmul(&self, rhs: &Mat) -> Mat {
        assert_eq!(self.cols, rhs.rows, "matmul shape mismatch");
        let mut m = Mat::zeros(self.rows, rhs.cols);
        for i in 0..self.rows {
            for j in 0..rhs.cols {
                let mut acc = Rational::zero();
                for k in 0..self.cols {
                    acc = acc.add(&self.get(i, k).mul(rhs.get(k, j)));
                }
                m.set(i, j, acc);
            }
        }
        m
    }

    /// Scalar multiple.
    pub fn scale(&self, s: &Rational) -> Mat {
        let mut m = self.clone();
        for v in &mut m.a {
            *v = v.mul(s);
        }
        m
    }

    /// Entry-wise sum (shapes must match).
    pub fn add_mat(&self, rhs: &Mat) -> Mat {
        let mut m = self.clone();
        for (v, w) in m.a.iter_mut().zip(rhs.a.iter()) {
            *v = v.add(w);
        }
        m
    }

    /// Is every entry exactly zero?
    pub fn is_zero(&self) -> bool {
        self.a.iter().all(is0)
    }

    /// Column `j` as a vector.
    pub fn col(&self, j: usize) -> Vec<Rational> {
        (0..self.rows).map(|i| self.get(i, j).clone()).collect()
    }

    /// Row `i` as a vector.
    pub fn row(&self, i: usize) -> Vec<Rational> {
        (0..self.cols).map(|j| self.get(i, j).clone()).collect()
    }

    /// Determinant by Gaussian elimination. Panics on a non-square matrix.
    pub fn det(&self) -> Rational {
        assert_eq!(self.rows, self.cols, "det of a non-square matrix");
        let n = self.rows;
        let mut m = self.clone();
        let mut det = Rational::one();
        for k in 0..n {
            let Some(p) = (k..n).find(|&i| !is0(m.get(i, k))) else {
                return Rational::zero();
            };
            if p != k {
                for j in 0..n {
                    let t = m.get(k, j).clone();
                    m.set(k, j, m.get(p, j).clone());
                    m.set(p, j, t);
                }
                det = det.neg();
            }
            let d = m.get(k, k).clone();
            det = det.mul(&d);
            for i in (k + 1)..n {
                let f = m.get(i, k).div(&d);
                if is0(&f) {
                    continue;
                }
                for j in k..n {
                    let v = m.get(i, j).sub(&f.mul(m.get(k, j)));
                    m.set(i, j, v);
                }
            }
        }
        det
    }

    /// Row-reduced echelon form, plus the pivot column indices.
    pub fn rref(&self) -> (Mat, Vec<usize>) {
        let mut m = self.clone();
        let mut pivots = Vec::new();
        let mut row = 0usize;
        for col in 0..m.cols {
            if row >= m.rows {
                break;
            }
            let Some(p) = (row..m.rows).find(|&i| !is0(m.get(i, col))) else {
                continue;
            };
            if p != row {
                for j in 0..m.cols {
                    let t = m.get(row, j).clone();
                    m.set(row, j, m.get(p, j).clone());
                    m.set(p, j, t);
                }
            }
            let d = m.get(row, col).clone();
            for j in 0..m.cols {
                let v = m.get(row, j).div(&d);
                m.set(row, j, v);
            }
            for i in 0..m.rows {
                if i == row {
                    continue;
                }
                let f = m.get(i, col).clone();
                if is0(&f) {
                    continue;
                }
                for j in 0..m.cols {
                    let v = m.get(i, j).sub(&f.mul(m.get(row, j)));
                    m.set(i, j, v);
                }
            }
            pivots.push(col);
            row += 1;
        }
        (m, pivots)
    }

    /// Exact rank.
    pub fn rank(&self) -> usize {
        self.rref().1.len()
    }

    /// A basis of the right null space, as column vectors.
    pub fn kernel(&self) -> Vec<Vec<Rational>> {
        let (m, pivots) = self.rref();
        let free: Vec<usize> = (0..self.cols).filter(|c| !pivots.contains(c)).collect();
        let mut basis = Vec::new();
        for &f in &free {
            let mut v = vec![Rational::zero(); self.cols];
            v[f] = Rational::one();
            for (pi, &pc) in pivots.iter().enumerate() {
                v[pc] = m.get(pi, f).neg();
            }
            basis.push(v);
        }
        basis
    }

    /// The exact inverse, or `None` when singular.
    pub fn inverse(&self) -> Option<Mat> {
        let n = self.rows;
        if n != self.cols {
            return None;
        }
        let mut aug = Mat::zeros(n, 2 * n);
        for i in 0..n {
            for j in 0..n {
                aug.set(i, j, self.get(i, j).clone());
            }
            aug.set(i, n + i, Rational::one());
        }
        let (red, piv) = aug.rref();
        if piv != (0..n).collect::<Vec<_>>() {
            return None;
        }
        let mut inv = Mat::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                inv.set(i, j, red.get(i, n + j).clone());
            }
        }
        Some(inv)
    }

    /// Congruence diagonalisation of a symmetric matrix (Lagrange): returns
    /// `(d, T)` with `Tᵀ·self·T = diag(d)` and `T` invertible over ℚ.
    ///
    /// The workhorse: rank AND Sylvester inertia fall out of `d` with no
    /// eigenvalue computation and no algebraic numbers.
    pub fn congruence_diag(&self) -> (Vec<Rational>, Mat) {
        assert_eq!(self.rows, self.cols, "congruence of a non-square matrix");
        let n = self.rows;
        let mut a = self.clone();
        let mut t = Mat::ident(n);

        // col_i += c·col_k and row_i += c·row_k, i.e. A ← EᵀAE, T ← T·E.
        fn col_add(a: &mut Mat, t: &mut Mat, i: usize, k: usize, c: &Rational) {
            let n = a.rows;
            for row in 0..n {
                let v = a.get(row, i).add(&c.mul(a.get(row, k)));
                a.set(row, i, v);
            }
            for col in 0..n {
                let v = a.get(i, col).add(&c.mul(a.get(k, col)));
                a.set(i, col, v);
            }
            for row in 0..n {
                let v = t.get(row, i).add(&c.mul(t.get(row, k)));
                t.set(row, i, v);
            }
        }
        fn swap(a: &mut Mat, t: &mut Mat, i: usize, j: usize) {
            let n = a.rows;
            for col in 0..n {
                let v = a.get(i, col).clone();
                a.set(i, col, a.get(j, col).clone());
                a.set(j, col, v);
            }
            for row in 0..n {
                let v = a.get(row, i).clone();
                a.set(row, i, a.get(row, j).clone());
                a.set(row, j, v);
            }
            for row in 0..n {
                let v = t.get(row, i).clone();
                t.set(row, i, t.get(row, j).clone());
                t.set(row, j, v);
            }
        }

        for k in 0..n {
            if is0(a.get(k, k)) {
                if let Some(j) = ((k + 1)..n).find(|&j| !is0(a.get(j, j))) {
                    swap(&mut a, &mut t, k, j);
                } else {
                    // Every remaining diagonal entry vanishes: manufacture one
                    // from an off-diagonal (a[i][i] becomes 2·a[i][j] ≠ 0).
                    let mut found = None;
                    'outer: for i in k..n {
                        for j in (i + 1)..n {
                            if !is0(a.get(i, j)) {
                                found = Some((i, j));
                                break 'outer;
                            }
                        }
                    }
                    match found {
                        None => break, // the remaining block is identically zero
                        Some((i, j)) => {
                            col_add(&mut a, &mut t, i, j, &Rational::one());
                            if i != k {
                                swap(&mut a, &mut t, k, i);
                            }
                        }
                    }
                }
            }
            let d = a.get(k, k).clone();
            for i in (k + 1)..n {
                if is0(a.get(i, k)) {
                    continue;
                }
                let c = a.get(i, k).div(&d).neg();
                col_add(&mut a, &mut t, i, k, &c);
            }
        }
        let d: Vec<Rational> = (0..n).map(|i| a.get(i, i).clone()).collect();
        (d, t)
    }

    /// `(n₊, n₋, n₀)` — the Sylvester inertia of a symmetric matrix.
    pub fn inertia(&self) -> (usize, usize, usize) {
        let (d, _) = self.congruence_diag();
        let mut counts = (0, 0, 0);
        for v in &d {
            match v.sign() {
                Sign::Positive => counts.0 += 1,
                Sign::Negative => counts.1 += 1,
                Sign::Zero => counts.2 += 1,
            }
        }
        counts
    }

    /// Are two matrices proportional — i.e. the same projective quadric?
    /// `false` when either is zero and the other is not.
    pub fn proportional_to(&self, other: &Mat) -> bool {
        let mut ratio: Option<Rational> = None;
        for (x, y) in self.a.iter().zip(other.a.iter()) {
            match (is0(x), is0(y)) {
                (true, true) => continue,
                (true, false) | (false, true) => return false,
                (false, false) => {
                    let k = y.div(x);
                    match &ratio {
                        None => ratio = Some(k),
                        Some(k0) if *k0 == k => {}
                        _ => return false,
                    }
                }
            }
        }
        ratio.is_some()
    }
}

/// The bilinear form `vᵀ M w`.
pub fn bilinear(v: &[Rational], m: &Mat, w: &[Rational]) -> Rational {
    assert_eq!(v.len(), m.rows, "left vector dimension mismatch");
    assert_eq!(w.len(), m.cols, "right vector dimension mismatch");
    let mut acc = Rational::zero();
    for (i, vi) in v.iter().enumerate() {
        let mut inner = Rational::zero();
        for (j, wj) in w.iter().enumerate() {
            inner = inner.add(&m.get(i, j).mul(wj));
        }
        acc = acc.add(&vi.mul(&inner));
    }
    acc
}

/// Extend the nonzero vector `v` to an invertible `n x n` matrix whose **last**
/// column is `v`. Used to move a singular point of a quadric to `e_n`.
pub fn basis_with_last(v: &[Rational]) -> Mat {
    let n = v.len();
    let mut cols: Vec<Vec<Rational>> = Vec::new();
    for i in 0..n {
        if cols.len() == n - 1 {
            break;
        }
        let mut e = vec![Rational::zero(); n];
        e[i] = Rational::one();
        let mut trial = cols.clone();
        trial.push(e.clone());
        trial.push(v.to_vec());
        if Mat::from_cols(n, &trial).rank() == trial.len() {
            cols.push(e);
        }
    }
    cols.push(v.to_vec());
    Mat::from_cols(n, &cols)
}

/// Are two homogeneous coordinate vectors the same projective point?
pub fn same_projective_point(a: &[Rational], b: &[Rational]) -> bool {
    let mut ratio: Option<Rational> = None;
    for (x, y) in a.iter().zip(b.iter()) {
        match (is0(x), is0(y)) {
            (true, true) => continue,
            (true, false) | (false, true) => return false,
            (false, false) => {
                let k = y.div(x);
                match &ratio {
                    None => ratio = Some(k),
                    Some(k0) if *k0 == k => {}
                    _ => return false,
                }
            }
        }
    }
    ratio.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(i: i64) -> Rational {
        Rational::from_i64(i)
    }

    #[test]
    fn det_rank_kernel_and_inverse() {
        let m = Mat::from_rows(&[
            vec![r(2), r(0), r(0)],
            vec![r(0), r(3), r(0)],
            vec![r(0), r(0), r(5)],
        ]);
        assert_eq!(m.det(), r(30));
        assert_eq!(m.rank(), 3);
        assert!(m.kernel().is_empty());
        let inv = m.inverse().expect("nonsingular");
        assert_eq!(inv.matmul(&m), Mat::ident(3));

        let sing = Mat::from_rows(&[
            vec![r(1), r(2), r(3)],
            vec![r(2), r(4), r(6)],
            vec![r(0), r(0), r(0)],
        ]);
        assert_eq!(sing.det(), r(0));
        assert_eq!(sing.rank(), 1);
        assert_eq!(sing.kernel().len(), 2);
        assert!(sing.inverse().is_none());
    }

    #[test]
    fn congruence_diagonalises_and_reports_inertia() {
        // A form with a zero diagonal — the branch that must manufacture a
        // pivot from an off-diagonal entry. xy has signature (1,1).
        let m = Mat::from_rows(&[vec![r(0), r(1)], vec![r(1), r(0)]]);
        let (d, t) = m.congruence_diag();
        let got = t.transpose().matmul(&m).matmul(&t);
        for (i, di) in d.iter().enumerate().take(2) {
            for j in 0..2 {
                let want = if i == j { di.clone() } else { r(0) };
                assert_eq!(*got.get(i, j), want);
            }
        }
        assert_eq!(m.inertia(), (1, 1, 0));
        assert_eq!(Mat::ident(4).inertia(), (4, 0, 0));
        assert_eq!(Mat::zeros(3, 3).inertia(), (0, 0, 3));
    }

    #[test]
    fn basis_extension_and_projective_helpers() {
        let v = vec![r(0), r(2), r(0)];
        let b = basis_with_last(&v);
        assert_eq!(b.rank(), 3);
        assert_eq!(b.col(2), v);
        assert!(same_projective_point(&[r(1), r(2)], &[r(3), r(6)]));
        assert!(!same_projective_point(&[r(1), r(2)], &[r(3), r(7)]));
        assert!(!same_projective_point(&[r(1), r(0)], &[r(1), r(1)]));
        assert!(Mat::ident(2).proportional_to(&Mat::ident(2).scale(&r(7))));
        assert!(!Mat::ident(2).proportional_to(&Mat::zeros(2, 2)));
    }

    #[test]
    fn bilinear_matches_by_hand() {
        let m = Mat::from_rows(&[vec![r(1), r(2)], vec![r(2), r(3)]]);
        // (1,1)ᵀ M (1,1) = 1 + 2 + 2 + 3
        assert_eq!(bilinear(&[r(1), r(1)], &m, &[r(1), r(1)]), r(8));
    }
}
