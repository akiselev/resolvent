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

use crate::AlgebraError;
use crate::error::AlgebraWork;
use crate::exact::{ExactField, ExactRing, Rational, RingOps};
use crate::uncertain::Sign;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

const MAT_SCHEMA: &str = "resolvent-rational-matrix/1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatWire {
    schema: String,
    rows: usize,
    cols: usize,
    entries: Vec<Rational>,
}

impl Serialize for Mat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validate("serializing a matrix")
            .map_err(serde::ser::Error::custom)?;
        MatWire {
            schema: MAT_SCHEMA.into(),
            rows: self.rows,
            cols: self.cols,
            entries: self.a.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = MatWire::deserialize(deserializer)?;
        if wire.schema != MAT_SCHEMA {
            return Err(serde::de::Error::custom(
                "unsupported rational-matrix schema",
            ));
        }
        let matrix = Mat {
            rows: wire.rows,
            cols: wire.cols,
            a: wire.entries,
        };
        matrix
            .validate("deserializing a matrix")
            .map_err(serde::de::Error::custom)?;
        Ok(matrix)
    }
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
        Self::try_from_rows(rows).expect("Mat::from_rows: ragged rows")
    }

    /// From equal-length rows, rejecting ragged input.
    pub fn try_from_rows(rows: &[Vec<Rational>]) -> Result<Mat, AlgebraError> {
        let nr = rows.len();
        let nc = rows.first().map_or(0, Vec::len);
        if rows.iter().any(|row| row.len() != nc) {
            return Err(AlgebraError::Shape {
                operation: "constructing a matrix from rows",
                details: "rows have different lengths".into(),
            });
        }
        let mut m = Mat::zeros(nr, nc);
        for (i, row) in rows.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                m.set(i, j, v.clone());
            }
        }
        Ok(m)
    }

    /// The `n x n` matrix whose columns are `cols` (each of length `n`).
    pub fn from_cols(n: usize, cols: &[Vec<Rational>]) -> Mat {
        Self::try_from_cols(n, cols).expect("Mat::from_cols: wrong column length")
    }

    /// From columns of length `n`, rejecting malformed input.
    pub fn try_from_cols(n: usize, cols: &[Vec<Rational>]) -> Result<Mat, AlgebraError> {
        if cols.iter().any(|column| column.len() != n) {
            return Err(AlgebraError::Shape {
                operation: "constructing a matrix from columns",
                details: format!("a column does not have declared length {n}"),
            });
        }
        let mut m = Mat::zeros(n, cols.len());
        for (j, c) in cols.iter().enumerate() {
            for (i, x) in c.iter().enumerate() {
                m.set(i, j, x.clone());
            }
        }
        Ok(m)
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
        self.checked_matmul(rhs).expect("matmul shape mismatch")
    }

    /// Matrix product, rejecting malformed matrices and shape mismatches.
    pub fn checked_matmul(&self, rhs: &Mat) -> Result<Mat, AlgebraError> {
        self.checked_matmul_with_budget(
            rhs,
            crate::AlgebraBudget {
                max_expression_nodes: usize::MAX,
                max_matrix_dimension: usize::MAX,
                max_coefficient_bits: u64::MAX,
                ..crate::AlgebraBudget::default()
            },
        )
    }

    /// Matrix product under explicit dimension, work, and coefficient limits.
    pub fn checked_matmul_with_budget(
        &self,
        rhs: &Mat,
        budget: crate::AlgebraBudget,
    ) -> Result<Mat, AlgebraError> {
        self.validate("matrix multiplication")?;
        rhs.validate("matrix multiplication")?;
        self.validate_budget("matrix multiplication", budget)?;
        rhs.validate_budget("matrix multiplication", budget)?;
        if self.cols != rhs.rows {
            return Err(AlgebraError::Shape {
                operation: "matrix multiplication",
                details: format!("{} columns != {} rows", self.cols, rhs.rows),
            });
        }
        let output_dimension = self.rows.max(rhs.cols);
        if output_dimension > budget.max_matrix_dimension {
            return Err(AlgebraError::MatrixDimension {
                actual: output_dimension,
                limit: budget.max_matrix_dimension,
            });
        }
        let mut work = AlgebraWork::new(budget, "multiplying matrices");
        let mut m = Mat::zeros(self.rows, rhs.cols);
        for i in 0..self.rows {
            for j in 0..rhs.cols {
                let mut acc = Rational::zero();
                for k in 0..self.cols {
                    work.spend(2)?;
                    let product = self.get(i, k).mul(rhs.get(k, j));
                    check_bits(&product, budget)?;
                    let value = acc.add(&product);
                    check_bits(&value, budget)?;
                    acc = value;
                }
                m.set(i, j, acc);
            }
        }
        Ok(m)
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
        self.checked_add_mat(rhs)
            .expect("matrix addition shape mismatch")
    }

    /// Entry-wise sum, rejecting malformed matrices and shape mismatches.
    pub fn checked_add_mat(&self, rhs: &Mat) -> Result<Mat, AlgebraError> {
        self.validate("matrix addition")?;
        rhs.validate("matrix addition")?;
        if (self.rows, self.cols) != (rhs.rows, rhs.cols) {
            return Err(AlgebraError::Shape {
                operation: "matrix addition",
                details: format!("{}x{} != {}x{}", self.rows, self.cols, rhs.rows, rhs.cols),
            });
        }
        let mut m = self.clone();
        for (v, w) in m.a.iter_mut().zip(rhs.a.iter()) {
            *v = v.add(w);
        }
        Ok(m)
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
        self.checked_det()
            .expect("det of a malformed/non-square matrix")
    }

    /// Determinant, rejecting malformed and non-square matrices.
    pub fn checked_det(&self) -> Result<Rational, AlgebraError> {
        self.checked_det_with_budget(crate::AlgebraBudget::default())
    }

    /// Determinant under explicit dimension and coefficient-growth limits.
    pub fn checked_det_with_budget(
        &self,
        budget: crate::AlgebraBudget,
    ) -> Result<Rational, AlgebraError> {
        self.validate("matrix determinant")?;
        self.validate_budget("matrix determinant", budget)?;
        if self.rows != self.cols {
            return Err(AlgebraError::Shape {
                operation: "matrix determinant",
                details: format!("{}x{} is not square", self.rows, self.cols),
            });
        }
        let n = self.rows;
        let mut work = AlgebraWork::new(budget, "computing a matrix determinant");
        let mut m = self.clone();
        let mut det = Rational::one();
        for k in 0..n {
            let mut pivot = None;
            for i in k..n {
                work.spend(1)?;
                if !is0(m.get(i, k)) {
                    pivot = Some(i);
                    break;
                }
            }
            let Some(p) = pivot else {
                return Ok(Rational::zero());
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
            work.spend(1)?;
            det = det.mul(&d);
            check_bits(&det, budget)?;
            for i in (k + 1)..n {
                work.spend(1)?;
                let f = m.get(i, k).div(&d);
                check_bits(&f, budget)?;
                if is0(&f) {
                    continue;
                }
                for j in k..n {
                    work.spend(2)?;
                    let product = f.mul(m.get(k, j));
                    check_bits(&product, budget)?;
                    let v = m.get(i, j).sub(&product);
                    check_bits(&v, budget)?;
                    m.set(i, j, v);
                }
            }
        }
        Ok(det)
    }

    /// Row-reduced echelon form, plus the pivot column indices.
    pub fn rref(&self) -> (Mat, Vec<usize>) {
        self.rref_with_budget(crate::AlgebraBudget {
            max_matrix_dimension: usize::MAX,
            max_coefficient_bits: u64::MAX,
            max_expression_nodes: usize::MAX,
            ..crate::AlgebraBudget::default()
        })
        .expect("unbounded RREF only requires a well-formed matrix")
    }

    /// Row-reduced echelon form under explicit dimension/coefficient limits.
    pub fn rref_with_budget(
        &self,
        budget: crate::AlgebraBudget,
    ) -> Result<(Mat, Vec<usize>), AlgebraError> {
        self.validate("matrix RREF")?;
        self.validate_budget("matrix RREF", budget)?;
        let mut work = AlgebraWork::new(budget, "computing matrix RREF");
        let mut m = self.clone();
        let mut pivots = Vec::new();
        let mut row = 0usize;
        for col in 0..m.cols {
            if row >= m.rows {
                break;
            }
            let mut pivot = None;
            for i in row..m.rows {
                work.spend(1)?;
                if !is0(m.get(i, col)) {
                    pivot = Some(i);
                    break;
                }
            }
            let Some(p) = pivot else {
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
                work.spend(1)?;
                let v = m.get(row, j).div(&d);
                check_bits(&v, budget)?;
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
                    work.spend(2)?;
                    let product = f.mul(m.get(row, j));
                    check_bits(&product, budget)?;
                    let v = m.get(i, j).sub(&product);
                    check_bits(&v, budget)?;
                    m.set(i, j, v);
                }
            }
            pivots.push(col);
            row += 1;
        }
        Ok((m, pivots))
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
        self.checked_congruence_diag()
            .expect("congruence of a malformed/non-square matrix")
    }

    /// Congruence diagonalisation, rejecting malformed/non-square matrices.
    pub fn checked_congruence_diag(&self) -> Result<(Vec<Rational>, Mat), AlgebraError> {
        self.validate("matrix congruence")?;
        if self.rows != self.cols {
            return Err(AlgebraError::Shape {
                operation: "matrix congruence",
                details: format!("{}x{} is not square", self.rows, self.cols),
            });
        }
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
        Ok((d, t))
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
        if self.validate("matrix proportionality").is_err()
            || other.validate("matrix proportionality").is_err()
            || (self.rows, self.cols) != (other.rows, other.cols)
        {
            return false;
        }
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

    fn validate(&self, operation: &'static str) -> Result<(), AlgebraError> {
        if self.rows.checked_mul(self.cols) != Some(self.a.len()) {
            return Err(AlgebraError::Shape {
                operation,
                details: format!(
                    "{}x{} declares {} entries but stores {}",
                    self.rows,
                    self.cols,
                    self.rows.saturating_mul(self.cols),
                    self.a.len()
                ),
            });
        }
        Ok(())
    }

    fn validate_budget(
        &self,
        operation: &'static str,
        budget: crate::AlgebraBudget,
    ) -> Result<(), AlgebraError> {
        let dimension = self.rows.max(self.cols);
        if dimension > budget.max_matrix_dimension {
            return Err(AlgebraError::MatrixDimension {
                actual: dimension,
                limit: budget.max_matrix_dimension,
            });
        }
        for value in &self.a {
            check_bits(value, budget).map_err(|_| AlgebraError::CoefficientBits {
                actual: value.bit_size(),
                limit: budget.max_coefficient_bits,
            })?;
        }
        let _ = operation;
        Ok(())
    }
}

fn check_bits(value: &Rational, budget: crate::AlgebraBudget) -> Result<(), AlgebraError> {
    let actual = value.bit_size();
    if actual > budget.max_coefficient_bits {
        Err(AlgebraError::CoefficientBits {
            actual,
            limit: budget.max_coefficient_bits,
        })
    } else {
        Ok(())
    }
}

/// The bilinear form `vᵀ M w`.
pub fn bilinear(v: &[Rational], m: &Mat, w: &[Rational]) -> Rational {
    checked_bilinear(v, m, w).expect("bilinear form dimension mismatch")
}

/// Bilinear form `vᵀ M w`, rejecting malformed inputs and dimension mismatches.
pub fn checked_bilinear(v: &[Rational], m: &Mat, w: &[Rational]) -> Result<Rational, AlgebraError> {
    m.validate("bilinear form")?;
    if v.len() != m.rows || w.len() != m.cols {
        return Err(AlgebraError::Shape {
            operation: "bilinear form",
            details: format!(
                "vectors have lengths {} and {} for a {}x{} matrix",
                v.len(),
                w.len(),
                m.rows,
                m.cols
            ),
        });
    }
    let mut acc = Rational::zero();
    for (i, vi) in v.iter().enumerate() {
        let mut inner = Rational::zero();
        for (j, wj) in w.iter().enumerate() {
            inner = inner.add(&m.get(i, j).mul(wj));
        }
        acc = acc.add(&vi.mul(&inner));
    }
    Ok(acc)
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
    if a.len() != b.len() {
        return false;
    }
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
