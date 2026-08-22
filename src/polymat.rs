//! Matrices of univariate rational polynomials, and the determinantal data a
//! **matrix pencil** is classified by.
//!
//! For two symmetric rational matrices `A`, `B`, the pencil `λA + B` is
//! completely classified over ℂ by its **elementary divisors**, and those are
//! read off the determinantal divisors `D_k` — the gcd of all `k x k` minors —
//! entirely inside ℚ\[λ\]. No eigenvalues, no algebraic numbers, no per-input
//! case analysis: [`PolyMat::invariant_factors`] is the whole computation.
//!
//! Laplace expansion is used for the minors. That is exponential in the block
//! size and deliberate: the consumers are 3x3 and 4x4, where it beats fraction-
//! free elimination on coefficient growth, which is the real cost here.

use crate::error::AlgebraWork;
use crate::exact::Rational;
use crate::ratmat::Mat;
use crate::roots::QPoly;
use crate::{AlgebraBudget, AlgebraError};

/// A dense `n x n` matrix of [`QPoly`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolyMat {
    /// Side length.
    pub n: usize,
    /// Entries, row-major.
    pub a: Vec<QPoly>,
}

impl PolyMat {
    /// The pencil `λ·m1 + m0`, entry-wise. Both matrices must be `n x n`.
    pub fn pencil(m1: &Mat, m0: &Mat) -> PolyMat {
        Self::pencil_with_budget(m1, m0, AlgebraBudget::default())
            .expect("PolyMat::pencil requires equal square matrices within the default budget")
    }

    /// Construct a polynomial pencil under explicit shape/resource limits.
    pub fn pencil_with_budget(
        m1: &Mat,
        m0: &Mat,
        budget: AlgebraBudget,
    ) -> Result<PolyMat, AlgebraError> {
        if m1.rows != m1.cols
            || m0.rows != m0.cols
            || (m1.rows, m1.cols) != (m0.rows, m0.cols)
            || m1.a.len() != m1.rows.saturating_mul(m1.cols)
            || m0.a.len() != m0.rows.saturating_mul(m0.cols)
        {
            return Err(AlgebraError::Shape {
                operation: "constructing a polynomial pencil",
                details: "matrices must be well-formed, square, and equal-sized".into(),
            });
        }
        let n = m1.rows;
        if n > budget.max_matrix_dimension {
            return Err(AlgebraError::MatrixDimension {
                actual: n,
                limit: budget.max_matrix_dimension,
            });
        }
        let mut a = Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                a.push(QPoly::new(vec![m0.get(i, j).clone(), m1.get(i, j).clone()]));
            }
        }
        let result = PolyMat { n, a };
        result.validate_budget(budget)?;
        Ok(result)
    }

    /// Entry `(i, j)`.
    pub fn get(&self, i: usize, j: usize) -> &QPoly {
        &self.a[i * self.n + j]
    }

    /// Determinant of the submatrix on `rows x cols`, by Laplace expansion.
    /// The empty minor is `1`.
    pub fn minor(&self, rows: &[usize], cols: &[usize]) -> QPoly {
        self.minor_with_budget(rows, cols, AlgebraBudget::default())
            .expect("minor indices/shape/work must fit the default budget")
    }

    /// Determinant of a submatrix under explicit work and growth limits.
    pub fn minor_with_budget(
        &self,
        rows: &[usize],
        cols: &[usize],
        budget: AlgebraBudget,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        if rows.len() != cols.len()
            || rows.iter().any(|&index| index >= self.n)
            || cols.iter().any(|&index| index >= self.n)
        {
            return Err(AlgebraError::Shape {
                operation: "computing a polynomial minor",
                details: "row/column selections must be equal-sized and in range".into(),
            });
        }
        let mut work = AlgebraWork::new(budget, "computing a polynomial matrix minor");
        self.minor_inner(rows, cols, budget, &mut work)
    }

    fn minor_inner(
        &self,
        rows: &[usize],
        cols: &[usize],
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        work.spend(1)?;
        let k = rows.len();
        if k == 0 {
            return Ok(QPoly::new(vec![Rational::one()]));
        }
        if k == 1 {
            return Ok(self.get(rows[0], cols[0]).clone());
        }
        let mut acc = QPoly::zero_poly();
        for (j, &c) in cols.iter().enumerate() {
            let e = self.get(rows[0], c);
            if e.is_zero() {
                continue;
            }
            let sub_cols: Vec<usize> = cols
                .iter()
                .enumerate()
                .filter(|(jj, _)| *jj != j)
                .map(|(_, &cc)| cc)
                .collect();
            let sub = self.minor_inner(&rows[1..], &sub_cols, budget, work)?;
            let term = e.mul_poly_with_meter(&sub, budget, work)?;
            acc = if j % 2 == 0 {
                acc.add_poly_with_meter(&term, budget, work)?
            } else {
                acc.sub_poly_with_meter(&term, budget, work)?
            };
            acc.validate_budget(budget)?;
        }
        Ok(acc)
    }

    /// The full determinant, `det(λ·m1 + m0)`.
    pub fn det(&self) -> QPoly {
        self.det_with_budget(AlgebraBudget::default())
            .expect("polynomial determinant must fit the default budget")
    }

    /// Full determinant under explicit work/growth limits.
    pub fn det_with_budget(&self, budget: AlgebraBudget) -> Result<QPoly, AlgebraError> {
        let all: Vec<usize> = (0..self.n).collect();
        self.minor_with_budget(&all, &all, budget)
    }

    /// The `k`-th **determinantal divisor** `D_k`: the monic gcd of all
    /// `k x k` minors. `D_0 = 1`; `D_k = 0` iff every `k x k` minor vanishes.
    pub fn determinantal_divisor(&self, k: usize) -> QPoly {
        self.determinantal_divisor_with_budget(k, AlgebraBudget::default())
            .expect("determinantal divisor must fit the default budget")
    }

    /// Determinantal divisor under explicit combination/work/growth limits.
    pub fn determinantal_divisor_with_budget(
        &self,
        k: usize,
        budget: AlgebraBudget,
    ) -> Result<QPoly, AlgebraError> {
        let mut work = AlgebraWork::new(budget, "computing a polynomial determinantal divisor");
        self.determinantal_divisor_with_meter(k, budget, &mut work)
    }

    fn determinantal_divisor_with_meter(
        &self,
        k: usize,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        let one = QPoly::new(vec![Rational::one()]);
        if k == 0 {
            return Ok(one);
        }
        if k > self.n {
            return Ok(QPoly::zero_poly());
        }
        let combinations_count = binomial_capped(self.n, k, budget.max_expression_nodes);
        if combinations_count
            .checked_mul(combinations_count)
            .is_none_or(|work| work > budget.max_expression_nodes)
        {
            return Err(AlgebraError::BudgetExceeded {
                operation: "enumerating polynomial minors",
                limit: budget.max_expression_nodes,
            });
        }
        work.spend(combinations_count)?;
        let combos = combinations(self.n, k);
        let mut g = QPoly::zero_poly();
        for rows in &combos {
            for cols in &combos {
                let m = self.minor_inner(rows, cols, budget, work)?;
                if m.is_zero() {
                    continue;
                }
                g = g.gcd_with_meter(&m, budget, work)?;
                if g.degree() == Some(0) {
                    return Ok(one); // a unit gcd cannot shrink further
                }
            }
        }
        // Every nonzero update is `gcd_with_budget`, which already returns a
        // monic polynomial. Avoid a second, unmetered coefficient division.
        g.validate_budget(budget)?;
        Ok(g)
    }

    /// The **invariant factors** `i_1 | i_2 | … | i_n` of the pencil over ℚ\[λ\],
    /// as `i_k = D_k / D_{k−1}`. Their prime-power parts are the elementary
    /// divisors, which *are* the Segre data.
    pub fn invariant_factors(&self) -> Vec<QPoly> {
        self.invariant_factors_with_budget(AlgebraBudget::default())
            .expect("invariant factors must fit the default budget")
    }

    /// Invariant factors under explicit combination/work/growth limits.
    pub fn invariant_factors_with_budget(
        &self,
        budget: AlgebraBudget,
    ) -> Result<Vec<QPoly>, AlgebraError> {
        self.validate_budget(budget)?;
        let mut work = AlgebraWork::new(budget, "computing polynomial invariant factors");
        let mut ds = Vec::with_capacity(self.n + 1);
        for k in 0..=self.n {
            ds.push(self.determinantal_divisor_with_meter(k, budget, &mut work)?);
        }
        let mut out = Vec::with_capacity(self.n);
        for k in 1..=self.n {
            if ds[k].is_zero() {
                out.push(QPoly::zero_poly());
                continue;
            }
            match ds[k].divrem_with_meter(&ds[k - 1], budget, &mut work) {
                Ok((q, rm)) => {
                    debug_assert!(rm.is_zero(), "D_(k−1) must divide D_k");
                    q.validate_budget(budget)?;
                    // Both determinantal divisors are monic, hence their exact
                    // quotient is monic without a second normalization pass.
                    out.push(q);
                }
                Err(AlgebraError::DivisionByZeroPolynomial) => {
                    out.push(QPoly::zero_poly());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(out)
    }

    /// The largest coefficient bit size anywhere in the matrix — the
    /// coefficient-growth metric.
    pub fn max_bits(&self) -> u64 {
        self.a.iter().map(poly_bits).max().unwrap_or(1)
    }

    fn validate_budget(&self, budget: AlgebraBudget) -> Result<(), AlgebraError> {
        if self.n > budget.max_matrix_dimension {
            return Err(AlgebraError::MatrixDimension {
                actual: self.n,
                limit: budget.max_matrix_dimension,
            });
        }
        if self.n.checked_mul(self.n) != Some(self.a.len()) {
            return Err(AlgebraError::Shape {
                operation: "validating a polynomial matrix",
                details: format!(
                    "{}x{} matrix stores {} entries",
                    self.n,
                    self.n,
                    self.a.len()
                ),
            });
        }
        for polynomial in &self.a {
            polynomial.validate_budget(budget)?;
        }
        Ok(())
    }
}

fn binomial_capped(n: usize, k: usize, cap: usize) -> usize {
    let k = k.min(n - k);
    let mut value = 1usize;
    for i in 0..k {
        let Some(product) = value.checked_mul(n - i) else {
            return cap.saturating_add(1);
        };
        value = product / (i + 1);
        if value > cap {
            return cap.saturating_add(1);
        }
    }
    value
}

/// The largest coefficient bit size in a polynomial (`1` for zero).
pub fn poly_bits(p: &QPoly) -> u64 {
    match p.degree() {
        None => 1,
        Some(d) => (0..=d).map(|i| p.coeff(i).bit_size()).max().unwrap_or(1),
    }
}

/// All `k`-subsets of `0..n`, ascending.
pub fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    fn go(start: usize, n: usize, k: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for i in start..n {
            cur.push(i);
            go(i + 1, n, k, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    go(0, n, k, &mut Vec::new(), &mut out);
    out
}

/// All `k`-subsets under explicit dimension and output-count limits.
pub fn combinations_with_budget(
    n: usize,
    k: usize,
    budget: AlgebraBudget,
) -> Result<Vec<Vec<usize>>, AlgebraError> {
    if n > budget.max_matrix_dimension {
        return Err(AlgebraError::MatrixDimension {
            actual: n,
            limit: budget.max_matrix_dimension,
        });
    }
    if k > n {
        return Ok(Vec::new());
    }
    if binomial_capped(n, k, budget.max_expression_nodes) > budget.max_expression_nodes {
        return Err(AlgebraError::BudgetExceeded {
            operation: "enumerating index combinations",
            limit: budget.max_expression_nodes,
        });
    }
    Ok(combinations(n, k))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(i: i64) -> Rational {
        Rational::from_i64(i)
    }

    #[test]
    fn pencil_determinant_and_invariant_factors() {
        // λI + diag(0, −1): det = λ(λ − 1), invariant factors 1, λ(λ−1)?
        // D_1 = gcd(λ, λ−1) = 1, D_2 = λ(λ−1), so i_1 = 1, i_2 = λ² − λ.
        let b = Mat::from_rows(&[vec![r(0), r(0)], vec![r(0), r(-1)]]);
        let pm = PolyMat::pencil(&Mat::ident(2), &b);
        assert_eq!(pm.det(), QPoly::from_i64s(&[0, -1, 1]));
        let inv = pm.invariant_factors();
        assert_eq!(inv[0], QPoly::from_i64s(&[1]));
        assert_eq!(inv[1], QPoly::from_i64s(&[0, -1, 1]));
    }

    #[test]
    fn a_repeated_eigenvalue_shows_up_in_d1() {
        // λI − I: det = (λ−1)², and D_1 = λ−1 because every entry carries it.
        // Two 1x1 Jordan blocks ⇒ invariant factors (λ−1), (λ−1).
        let pm = PolyMat::pencil(&Mat::ident(2), &Mat::ident(2).scale(&r(-1)));
        let inv = pm.invariant_factors();
        assert_eq!(inv[0], QPoly::from_i64s(&[-1, 1]));
        assert_eq!(inv[1], QPoly::from_i64s(&[-1, 1]));

        // One 2x2 Jordan block: D_1 = 1, so the single invariant factor is
        // (λ−1)² — this is exactly the distinction a (multiplicity, rank) pair
        // cannot make.
        let j = Mat::from_rows(&[vec![r(-1), r(1)], vec![r(0), r(-1)]]);
        let pm = PolyMat::pencil(&Mat::ident(2), &j);
        let inv = pm.invariant_factors();
        assert_eq!(inv[0], QPoly::from_i64s(&[1]));
        assert_eq!(inv[1], QPoly::from_i64s(&[1, -2, 1]));
    }

    #[test]
    fn singular_pencil_has_a_zero_determinant() {
        let z = Mat::zeros(2, 2);
        let pm = PolyMat::pencil(&z, &z);
        assert!(pm.det().is_zero());
        assert!(pm.determinantal_divisor(1).is_zero());
        assert!(pm.invariant_factors().iter().all(QPoly::is_zero));
    }

    #[test]
    fn combinations_are_the_k_subsets() {
        assert_eq!(combinations(3, 2), vec![vec![0, 1], vec![0, 2], vec![1, 2]]);
        assert_eq!(combinations(3, 0), vec![Vec::<usize>::new()]);
        assert_eq!(combinations(2, 3).len(), 0);
    }
}
