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

use crate::exact::Rational;
use crate::ratmat::Mat;
use crate::roots::QPoly;

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
        let n = m1.rows;
        let mut a = Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                a.push(QPoly::new(vec![m0.get(i, j).clone(), m1.get(i, j).clone()]));
            }
        }
        PolyMat { n, a }
    }

    /// Entry `(i, j)`.
    pub fn get(&self, i: usize, j: usize) -> &QPoly {
        &self.a[i * self.n + j]
    }

    /// Determinant of the submatrix on `rows x cols`, by Laplace expansion.
    /// The empty minor is `1`.
    pub fn minor(&self, rows: &[usize], cols: &[usize]) -> QPoly {
        let k = rows.len();
        debug_assert_eq!(k, cols.len());
        if k == 0 {
            return QPoly::new(vec![Rational::one()]);
        }
        if k == 1 {
            return self.get(rows[0], cols[0]).clone();
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
            let term = e.mul_poly(&self.minor(&rows[1..], &sub_cols));
            acc = if j % 2 == 0 {
                acc.add_poly(&term)
            } else {
                acc.sub_poly(&term)
            };
        }
        acc
    }

    /// The full determinant, `det(λ·m1 + m0)`.
    pub fn det(&self) -> QPoly {
        let all: Vec<usize> = (0..self.n).collect();
        self.minor(&all, &all)
    }

    /// The `k`-th **determinantal divisor** `D_k`: the monic gcd of all
    /// `k x k` minors. `D_0 = 1`; `D_k = 0` iff every `k x k` minor vanishes.
    pub fn determinantal_divisor(&self, k: usize) -> QPoly {
        let one = QPoly::new(vec![Rational::one()]);
        if k == 0 {
            return one;
        }
        let combos = combinations(self.n, k);
        let mut g = QPoly::zero_poly();
        for rows in &combos {
            for cols in &combos {
                let m = self.minor(rows, cols);
                if m.is_zero() {
                    continue;
                }
                g = g.gcd(&m);
                if g.degree() == Some(0) {
                    return one; // a unit gcd cannot shrink further
                }
            }
        }
        g.monic()
    }

    /// The **invariant factors** `i_1 | i_2 | … | i_n` of the pencil over ℚ\[λ\],
    /// as `i_k = D_k / D_{k−1}`. Their prime-power parts are the elementary
    /// divisors, which *are* the Segre data.
    pub fn invariant_factors(&self) -> Vec<QPoly> {
        let mut ds = Vec::with_capacity(self.n + 1);
        for k in 0..=self.n {
            ds.push(self.determinantal_divisor(k));
        }
        let mut out = Vec::with_capacity(self.n);
        for k in 1..=self.n {
            if ds[k].is_zero() {
                out.push(QPoly::zero_poly());
                continue;
            }
            match ds[k].div_rem(&ds[k - 1]) {
                Some((q, rm)) => {
                    debug_assert!(rm.is_zero(), "D_(k−1) must divide D_k");
                    out.push(q.monic());
                }
                None => out.push(QPoly::zero_poly()),
            }
        }
        out
    }

    /// The largest coefficient bit size anywhere in the matrix — the
    /// coefficient-growth metric.
    pub fn max_bits(&self) -> u64 {
        self.a.iter().map(poly_bits).max().unwrap_or(1)
    }
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
