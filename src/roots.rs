//! Real-root isolation over ℚ (DESIGN.md §3.6, scheduled for M6): dense
//! rational polynomials, Yun square-free decomposition, Descartes/VCA
//! bisection isolation, and [`RealRoot`] — an exact real algebraic number
//! with interval refinement, total ordering, and sign-at-root evaluation.
//!
//! Scope: the M6 consumer is the degree-≤4 conic/quartic event algebra
//! (ellipse ∩ ellipse resultants and friends). The algorithms are written
//! for arbitrary degree and are exact at every step, but no separation-
//! bound machinery is included — comparisons terminate because distinct
//! algebraic numbers are eventually separated by bisection, and equality
//! is decided algebraically through gcds, never by "small enough".
//!
//! Every decision is exact: `Rational` coefficient arithmetic throughout,
//! sign-change certificates on square-free divisors for membership, and
//! Descartes variation counts only in the directions where they are proof
//! (`0` ⇒ no roots; `1` on a square-free polynomial ⇒ exactly one).

use crate::error::AlgebraWork;
use crate::exact::{ExactField, ExactRing, Rational, RingOps};
use crate::interval::{AtomicInterval, Interval};
use crate::uncertain::{Sign, UOrd, USign, Uncertain};
use crate::{AlgebraBudget, AlgebraError};
use core::cmp::Ordering;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Errors from root isolation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RootError {
    /// The zero polynomial vanishes everywhere — it has no isolated roots.
    ZeroPolynomial,
    /// Root isolation exceeded its explicit bisection budget.
    BudgetExceeded {
        /// Maximum permitted bisections.
        limit: usize,
    },
    /// Polynomial degree exceeds the configured exact-work limit.
    DegreeExceeded {
        /// Observed degree.
        actual: usize,
        /// Maximum permitted degree.
        limit: usize,
    },
    /// A coefficient exceeds the configured bit-size limit.
    CoefficientBitsExceeded {
        /// Observed coefficient bit size.
        actual: u64,
        /// Maximum permitted coefficient bit size.
        limit: u64,
    },
    /// Deterministic algebraic work estimate exceeds the configured limit.
    WorkBudgetExceeded {
        /// Maximum permitted work units.
        limit: usize,
    },
}

impl core::fmt::Display for RootError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RootError::ZeroPolynomial => write!(f, "zero polynomial has no isolated roots"),
            RootError::BudgetExceeded { limit } => {
                write!(f, "root isolation exceeded bisection budget {limit}")
            }
            RootError::DegreeExceeded { actual, limit } => {
                write!(f, "root polynomial degree {actual} exceeds limit {limit}")
            }
            RootError::CoefficientBitsExceeded { actual, limit } => {
                write!(
                    f,
                    "root coefficient bit size {actual} exceeds limit {limit}"
                )
            }
            RootError::WorkBudgetExceeded { limit } => {
                write!(f, "root algebra exceeds deterministic work budget {limit}")
            }
        }
    }
}

impl std::error::Error for RootError {}

/// Validation failure for an immutable algebraic-root certificate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RootCertificateError {
    reason: &'static str,
}

impl core::fmt::Display for RootCertificateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid real-root certificate: {}", self.reason)
    }
}

impl std::error::Error for RootCertificateError {}

/// A dense univariate polynomial over ℚ, coefficients low-to-high with a
/// nonzero leading coefficient (the zero polynomial is the empty list).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QPoly {
    coeffs: Vec<Rational>,
}

const QPOLY_SCHEMA: &str = "resolvent-qpoly/1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QPolyWire {
    schema: String,
    coefficients: Vec<Rational>,
}

impl Serialize for QPoly {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        QPolyWire {
            schema: QPOLY_SCHEMA.into(),
            coefficients: self.coeffs.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QPoly {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = QPolyWire::deserialize(deserializer)?;
        if wire.schema != QPOLY_SCHEMA {
            return Err(serde::de::Error::custom("unsupported QPoly schema"));
        }
        if wire
            .coefficients
            .last()
            .is_some_and(|coefficient| coefficient.sign() == Sign::Zero)
        {
            return Err(serde::de::Error::custom(
                "non-canonical QPoly has a trailing zero coefficient",
            ));
        }
        Ok(QPoly {
            coeffs: wire.coefficients,
        })
    }
}

impl QPoly {
    /// From low-to-high coefficients; trailing zeros are trimmed.
    pub fn new(mut coeffs: Vec<Rational>) -> QPoly {
        while coeffs.last().is_some_and(|c| c.sign() == Sign::Zero) {
            coeffs.pop();
        }
        QPoly { coeffs }
    }

    /// Integer-coefficient convenience (low-to-high).
    pub fn from_i64s(coeffs: &[i64]) -> QPoly {
        QPoly::new(coeffs.iter().map(|&c| Rational::from_i64(c)).collect())
    }

    /// The zero polynomial.
    pub fn zero_poly() -> QPoly {
        QPoly { coeffs: Vec::new() }
    }

    /// Is this the zero polynomial?
    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// Degree; `None` for the zero polynomial.
    pub fn degree(&self) -> Option<usize> {
        self.coeffs.len().checked_sub(1)
    }

    /// Coefficient of `x^i` (zero beyond the degree).
    pub fn coeff(&self, i: usize) -> Rational {
        self.coeffs.get(i).cloned().unwrap_or_else(Rational::zero)
    }

    /// Canonical low-to-high coefficient slice.
    pub fn coefficients(&self) -> &[Rational] {
        &self.coeffs
    }

    /// Exact scalar resultant via a budgeted Sylvester determinant.
    pub fn resultant(&self, rhs: &QPoly, budget: AlgebraBudget) -> Result<Rational, AlgebraError> {
        if self.is_zero() || rhs.is_zero() {
            return Ok(Rational::zero());
        }
        let (m, n) = (
            self.degree().expect("nonzero polynomial"),
            rhs.degree().expect("nonzero polynomial"),
        );
        let degree = m.max(n);
        if degree > budget.max_polynomial_degree {
            return Err(AlgebraError::PolynomialDegree {
                actual: degree,
                limit: budget.max_polynomial_degree,
            });
        }
        let coefficient_bits = self
            .coeffs
            .iter()
            .chain(&rhs.coeffs)
            .map(Rational::bit_size)
            .max()
            .unwrap_or(0);
        if coefficient_bits > budget.max_coefficient_bits {
            return Err(AlgebraError::CoefficientBits {
                actual: coefficient_bits,
                limit: budget.max_coefficient_bits,
            });
        }
        let dimension = m + n;
        if dimension > budget.max_matrix_dimension {
            return Err(AlgebraError::ResultantDimension {
                actual: dimension,
                limit: budget.max_matrix_dimension,
            });
        }
        if dimension == 0 {
            return Ok(Rational::one());
        }
        let mut matrix = vec![vec![Rational::zero(); dimension]; dimension];
        let left = self.coeffs.iter().rev().cloned().collect::<Vec<_>>();
        let right = rhs.coeffs.iter().rev().cloned().collect::<Vec<_>>();
        for row in 0..n {
            matrix[row][row..row + left.len()].clone_from_slice(&left);
        }
        for row in 0..m {
            matrix[n + row][row..row + right.len()].clone_from_slice(&right);
        }
        determinant(matrix, budget)
    }

    /// Exact evaluation (Horner).
    pub fn eval(&self, x: &Rational) -> Rational {
        let mut acc = Rational::zero();
        for c in self.coeffs.iter().rev() {
            acc = acc.mul(x).add(c);
        }
        acc
    }

    fn eval_with_meter(
        &self,
        x: &Rational,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<Rational, AlgebraError> {
        self.validate_budget(budget)?;
        check_coefficient_bits(x, budget)?;
        let mut acc = Rational::zero();
        for coefficient in self.coeffs.iter().rev() {
            work.spend(2)?;
            let product = acc.mul(x);
            check_coefficient_bits(&product, budget)?;
            let value = product.add(coefficient);
            check_coefficient_bits(&value, budget)?;
            acc = value;
        }
        Ok(acc)
    }

    /// Exact sign at a rational point.
    pub fn sign_at(&self, x: &Rational) -> Sign {
        self.eval(x).sign()
    }

    /// Outward-correct interval evaluation (Horner) — **the filter rung of
    /// every sign predicate over an algebraic abscissa.**
    ///
    /// `p(x) ∈ p.eval_interval(X)` for every real `x ∈ X`, because each
    /// [`Interval`] operation encloses the exact result on its operands and
    /// each coefficient's [`ExactRing::to_interval`] encloses the
    /// coefficient. So a returned interval that excludes zero is a *proof*
    /// of the sign of `p` at every point of `X`, hence at the root the
    /// caller cares about — no refinement, no gcd, no bignum.
    pub fn eval_interval(&self, x: Interval) -> Interval {
        let mut acc = Interval::point(0.0);
        for c in self.coeffs.iter().rev() {
            acc = acc * x + c.to_interval();
        }
        acc
    }

    /// Formal derivative.
    pub fn derivative(&self) -> QPoly {
        if self.coeffs.len() <= 1 {
            return QPoly::zero_poly();
        }
        QPoly::new(
            self.coeffs
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, c)| c.mul(&Rational::from_i64(i as i64)))
                .collect(),
        )
    }

    fn derivative_with_meter(
        &self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        if self.coeffs.len() <= 1 {
            return Ok(QPoly::zero_poly());
        }
        let mut coefficients = Vec::with_capacity(self.coeffs.len() - 1);
        for (index, coefficient) in self.coeffs.iter().enumerate().skip(1) {
            work.spend(1)?;
            let value = coefficient.mul(&Rational::from_i64(index as i64));
            check_coefficient_bits(&value, budget)?;
            coefficients.push(value);
        }
        Ok(QPoly::new(coefficients))
    }

    /// Polynomial sum.
    pub fn add_poly(&self, rhs: &QPoly) -> QPoly {
        let n = self.coeffs.len().max(rhs.coeffs.len());
        QPoly::new((0..n).map(|i| self.coeff(i).add(&rhs.coeff(i))).collect())
    }

    /// Polynomial difference.
    pub fn sub_poly(&self, rhs: &QPoly) -> QPoly {
        let n = self.coeffs.len().max(rhs.coeffs.len());
        QPoly::new((0..n).map(|i| self.coeff(i).sub(&rhs.coeff(i))).collect())
    }

    /// Polynomial product.
    pub fn mul_poly(&self, rhs: &QPoly) -> QPoly {
        if self.is_zero() || rhs.is_zero() {
            return QPoly::zero_poly();
        }
        let mut out = vec![Rational::zero(); self.coeffs.len() + rhs.coeffs.len() - 1];
        for (i, a) in self.coeffs.iter().enumerate() {
            for (j, b) in rhs.coeffs.iter().enumerate() {
                out[i + j] = out[i + j].add(&a.mul(b));
            }
        }
        QPoly::new(out)
    }

    /// Polynomial product under explicit work, degree, and coefficient limits.
    pub fn mul_poly_with_budget(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
    ) -> Result<QPoly, AlgebraError> {
        let mut work = AlgebraWork::new(budget, "multiplying polynomials");
        self.mul_poly_with_meter(rhs, budget, &mut work)
    }

    pub(crate) fn mul_poly_with_meter(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        rhs.validate_budget(budget)?;
        if self.is_zero() || rhs.is_zero() {
            return Ok(QPoly::zero_poly());
        }
        let degree = self
            .degree()
            .and_then(|left| rhs.degree().and_then(|right| left.checked_add(right)))
            .ok_or(AlgebraError::PolynomialDegree {
                actual: usize::MAX,
                limit: budget.max_polynomial_degree,
            })?;
        if degree > budget.max_polynomial_degree {
            return Err(AlgebraError::PolynomialDegree {
                actual: degree,
                limit: budget.max_polynomial_degree,
            });
        }
        let mut out = vec![Rational::zero(); degree + 1];
        for (i, a) in self.coeffs.iter().enumerate() {
            for (j, b) in rhs.coeffs.iter().enumerate() {
                work.spend(2)?;
                let product = a.mul(b);
                check_coefficient_bits(&product, budget)?;
                let value = out[i + j].add(&product);
                check_coefficient_bits(&value, budget)?;
                out[i + j] = value;
            }
        }
        Ok(QPoly::new(out))
    }

    /// Scalar multiple.
    pub fn scale(&self, s: &Rational) -> QPoly {
        QPoly::new(self.coeffs.iter().map(|c| c.mul(s)).collect())
    }

    pub(crate) fn add_poly_with_meter(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        rhs.validate_budget(budget)?;
        let length = self.coeffs.len().max(rhs.coeffs.len());
        let mut coefficients = Vec::with_capacity(length);
        for index in 0..length {
            work.spend(1)?;
            let value = self.coeff(index).add(&rhs.coeff(index));
            check_coefficient_bits(&value, budget)?;
            coefficients.push(value);
        }
        Ok(QPoly::new(coefficients))
    }

    pub(crate) fn sub_poly_with_meter(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        rhs.validate_budget(budget)?;
        let length = self.coeffs.len().max(rhs.coeffs.len());
        let mut coefficients = Vec::with_capacity(length);
        for index in 0..length {
            work.spend(1)?;
            let value = self.coeff(index).sub(&rhs.coeff(index));
            check_coefficient_bits(&value, budget)?;
            coefficients.push(value);
        }
        Ok(QPoly::new(coefficients))
    }

    /// Euclidean division: `(quotient, remainder)` with
    /// `self = q·rhs + r`, `deg r < deg rhs`. Panics on zero divisor.
    fn divrem(&self, rhs: &QPoly) -> (QPoly, QPoly) {
        assert!(!rhs.is_zero(), "division by the zero polynomial");
        if self.coeffs.len() < rhs.coeffs.len() {
            return (QPoly::zero_poly(), self.clone());
        }
        let dr = rhs.coeffs.len() - 1;
        // `coeff` is total (zero beyond the degree), and `QPoly::new` trims
        // trailing zeros, so `coeff(dr)` *is* the non-zero leading
        // coefficient — no fallible accessor needed.
        let lead = rhs.coeff(dr);
        let mut rem = self.coeffs.clone();
        let qlen = rem.len() - dr;
        let mut quot = vec![Rational::zero(); qlen];
        for k in (0..qlen).rev() {
            let c = rem[k + dr].clone(); // coefficient of x^{k+dr}
            if c.sign() == Sign::Zero {
                continue;
            }
            let q = c.div(&lead);
            for (j, b) in rhs.coeffs.iter().enumerate() {
                rem[k + j] = rem[k + j].sub(&q.mul(b));
            }
            quot[k] = q;
        }
        rem.truncate(dr);
        (QPoly::new(quot), QPoly::new(rem))
    }

    pub(crate) fn divrem_with_meter(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<(QPoly, QPoly), AlgebraError> {
        if rhs.is_zero() {
            return Err(AlgebraError::DivisionByZeroPolynomial);
        }
        self.validate_budget(budget)?;
        rhs.validate_budget(budget)?;
        if self.coeffs.len() < rhs.coeffs.len() {
            return Ok((QPoly::zero_poly(), self.clone()));
        }
        let divisor_degree = rhs.coeffs.len() - 1;
        let lead = rhs.coeff(divisor_degree);
        let mut remainder = self.coeffs.clone();
        let quotient_len = remainder.len() - divisor_degree;
        let mut quotient = vec![Rational::zero(); quotient_len];
        for k in (0..quotient_len).rev() {
            let coefficient = remainder[k + divisor_degree].clone();
            if coefficient.sign() == Sign::Zero {
                continue;
            }
            work.spend(1)?;
            let q = coefficient.div(&lead);
            check_coefficient_bits(&q, budget)?;
            for (j, divisor_coefficient) in rhs.coeffs.iter().enumerate() {
                // One multiplication and one subtraction. Charge them before
                // either arbitrary-precision operation is executed.
                work.spend(2)?;
                let product = q.mul(divisor_coefficient);
                check_coefficient_bits(&product, budget)?;
                let value = remainder[k + j].sub(&product);
                check_coefficient_bits(&value, budget)?;
                remainder[k + j] = value;
            }
            quotient[k] = q;
        }
        remainder.truncate(divisor_degree);
        let quotient = QPoly::new(quotient);
        let remainder = QPoly::new(remainder);
        quotient.validate_budget(budget)?;
        remainder.validate_budget(budget)?;
        Ok((quotient, remainder))
    }

    /// Euclidean division `self = q·rhs + r` with `deg r < deg rhs`.
    /// `None` iff `rhs` is the zero polynomial — the public, total form of the
    /// internal `divrem`.
    pub fn div_rem(&self, rhs: &QPoly) -> Option<(QPoly, QPoly)> {
        (!rhs.is_zero()).then(|| self.divrem(rhs))
    }

    /// Euclidean division under explicit work, degree, and coefficient limits.
    pub fn div_rem_with_budget(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
    ) -> Result<(QPoly, QPoly), AlgebraError> {
        let mut work = AlgebraWork::new(budget, "dividing polynomials");
        self.divrem_with_meter(rhs, budget, &mut work)
    }

    /// Does `self` divide `rhs` exactly? The zero polynomial divides only `0`.
    pub fn divides(&self, rhs: &QPoly) -> bool {
        if self.is_zero() {
            return rhs.is_zero();
        }
        rhs.divrem(self).1.is_zero()
    }

    /// The monic associate `self / lead(self)`; the zero polynomial is fixed.
    pub fn monic(&self) -> QPoly {
        let Some(d) = self.degree() else {
            return self.clone();
        };
        self.scale(&Rational::one().div(&self.coeff(d)))
    }

    /// The exact polynomial square root: `Some(g)` with `g² = self`, when one
    /// exists over ℚ. Exact — a `Some` is a proof, and `0` is a square.
    ///
    /// Used to detect that a carrier's radicand is a perfect square, i.e. that
    /// the two sheets of `c ± s·√r` are separately **rational** and the
    /// quadratic extension collapses.
    pub fn sqrt_exact(&self) -> Option<QPoly> {
        let Some(d) = self.degree() else {
            return Some(QPoly::zero_poly()); // 0 = 0²
        };
        if d % 2 != 0 {
            return None;
        }
        if d == 0 {
            return self.coeff(0).sqrt_exact().map(|c| QPoly::new(vec![c]));
        }
        // For p = c·g² with g squarefree-or-not, gcd(p, p') carries g exactly
        // when deg gcd = d/2; then match leading coefficients.
        let g = self.gcd(&self.derivative());
        if g.degree() != Some(d / 2) {
            return None;
        }
        let gl = g.coeff(d / 2);
        let c = self.coeff(d).div(&gl.mul(&gl));
        let cand = g.scale(&c.sqrt_exact()?);
        cand.mul_poly(&cand)
            .sub_poly(self)
            .is_zero()
            .then_some(cand)
    }

    /// Monic gcd (Euclid). `gcd(0, q) = monic q`.
    pub fn gcd(&self, rhs: &QPoly) -> QPoly {
        let mut a = self.clone();
        let mut b = rhs.clone();
        while !b.is_zero() {
            let (_, r) = a.divrem(&b);
            a = b;
            b = r;
        }
        // `QPoly::new` trims trailing zeros, so `last()` is `None` exactly
        // when `a` is the zero polynomial — which `gcd(0, 0) = 0` returns
        // unchanged. One pattern replaces the `is_zero`-then-unwrap pair.
        let Some(lead) = a.coeffs.last().cloned() else {
            return a;
        };
        a.scale(&Rational::one().div(&lead))
    }

    /// Monic GCD under explicit Euclidean-work and coefficient limits.
    pub fn gcd_with_budget(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
    ) -> Result<QPoly, AlgebraError> {
        let mut work = AlgebraWork::new(budget, "computing a polynomial GCD");
        self.gcd_with_meter(rhs, budget, &mut work)
    }

    pub(crate) fn gcd_with_meter(
        &self,
        rhs: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        rhs.validate_budget(budget)?;
        let mut a = self.clone();
        let mut b = rhs.clone();
        while !b.is_zero() {
            let (_, remainder) = a.divrem_with_meter(&b, budget, work)?;
            a = b;
            b = remainder;
        }
        let Some(lead) = a.coeffs.last().cloned() else {
            return Ok(a);
        };
        let mut coefficients = Vec::with_capacity(a.coeffs.len());
        for coefficient in &a.coeffs {
            work.spend(1)?;
            let value = coefficient.div(&lead);
            check_coefficient_bits(&value, budget)?;
            coefficients.push(value);
        }
        let result = QPoly::new(coefficients);
        result.validate_budget(budget)?;
        Ok(result)
    }

    pub(crate) fn validate_budget(&self, budget: AlgebraBudget) -> Result<(), AlgebraError> {
        let degree = self.degree().unwrap_or(0);
        if degree > budget.max_polynomial_degree {
            return Err(AlgebraError::PolynomialDegree {
                actual: degree,
                limit: budget.max_polynomial_degree,
            });
        }
        for coefficient in &self.coeffs {
            check_coefficient_bits(coefficient, budget)?;
        }
        Ok(())
    }

    /// The square-free part `self / gcd(self, self')` (same root set,
    /// every root simple).
    pub fn square_free_part(&self) -> QPoly {
        let g = self.gcd(&self.derivative());
        if g.degree() == Some(0) {
            return self.clone();
        }
        let (q, r) = self.divrem(&g);
        debug_assert!(r.is_zero(), "gcd divides");
        q
    }

    fn square_free_part_with_meter(
        &self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        let derivative = self.derivative_with_meter(budget, work)?;
        let gcd = self.gcd_with_meter(&derivative, budget, work)?;
        if gcd.degree() == Some(0) {
            return Ok(self.clone());
        }
        let (quotient, remainder) = self.divrem_with_meter(&gcd, budget, work)?;
        debug_assert!(remainder.is_zero(), "gcd divides");
        Ok(quotient)
    }

    /// Yun's square-free decomposition: pairs `(multiplicity, factor)`
    /// with each factor square-free, pairwise coprime, and
    /// `self = lead · ∏ factor_i^i`. Constant factors are omitted.
    pub fn square_free_decomposition(&self) -> Vec<(u32, QPoly)> {
        if self.degree().unwrap_or(0) == 0 {
            return Vec::new();
        }
        let d = self.gcd(&self.derivative());
        if d.degree() == Some(0) {
            return vec![(1, self.clone())];
        }
        let (mut w, r) = self.divrem(&d);
        debug_assert!(r.is_zero());
        let (mut y, r) = self.derivative().divrem(&d);
        debug_assert!(r.is_zero());
        let mut z = y.sub_poly(&w.derivative());
        let mut out = Vec::new();
        let mut i = 1u32;
        while w.degree().unwrap_or(0) > 0 {
            let a = w.gcd(&z);
            if a.degree().unwrap_or(0) > 0 {
                out.push((i, a.clone()));
            }
            let (w2, r) = w.divrem(&a);
            debug_assert!(r.is_zero());
            w = w2;
            let (y2, r) = z.divrem(&a);
            debug_assert!(r.is_zero());
            y = y2;
            z = y.sub_poly(&w.derivative());
            i += 1;
        }
        out
    }

    fn square_free_decomposition_with_meter(
        &self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<Vec<(u32, QPoly)>, AlgebraError> {
        if self.degree().unwrap_or(0) == 0 {
            return Ok(Vec::new());
        }
        let derivative = self.derivative_with_meter(budget, work)?;
        let divisor = self.gcd_with_meter(&derivative, budget, work)?;
        if divisor.degree() == Some(0) {
            return Ok(vec![(1, self.clone())]);
        }
        let (mut square_free, remainder) = self.divrem_with_meter(&divisor, budget, work)?;
        debug_assert!(remainder.is_zero());
        let (mut derivative_quotient, remainder) =
            derivative.divrem_with_meter(&divisor, budget, work)?;
        debug_assert!(remainder.is_zero());
        let square_free_derivative = square_free.derivative_with_meter(budget, work)?;
        let mut residual =
            derivative_quotient.sub_poly_with_meter(&square_free_derivative, budget, work)?;
        let mut output = Vec::new();
        let mut multiplicity = 1u32;
        while square_free.degree().unwrap_or(0) > 0 {
            let factor = square_free.gcd_with_meter(&residual, budget, work)?;
            if factor.degree().unwrap_or(0) > 0 {
                output.push((multiplicity, factor.clone()));
            }
            let (next_square_free, remainder) =
                square_free.divrem_with_meter(&factor, budget, work)?;
            debug_assert!(remainder.is_zero());
            square_free = next_square_free;
            let (next_derivative_quotient, remainder) =
                residual.divrem_with_meter(&factor, budget, work)?;
            debug_assert!(remainder.is_zero());
            derivative_quotient = next_derivative_quotient;
            let derivative = square_free.derivative_with_meter(budget, work)?;
            residual = derivative_quotient.sub_poly_with_meter(&derivative, budget, work)?;
            multiplicity += 1;
        }
        Ok(output)
    }

    /// `p(a + b·x)` — affine composition.
    fn compose_affine(&self, a: &Rational, b: &Rational) -> QPoly {
        let mut acc = QPoly::zero_poly();
        let affine = QPoly::new(vec![a.clone(), b.clone()]);
        for c in self.coeffs.iter().rev() {
            acc = acc.mul_poly(&affine).add_poly(&QPoly::new(vec![c.clone()]));
        }
        acc
    }

    fn compose_affine_with_meter(
        &self,
        a: &Rational,
        b: &Rational,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<QPoly, AlgebraError> {
        self.validate_budget(budget)?;
        check_coefficient_bits(a, budget)?;
        check_coefficient_bits(b, budget)?;
        let affine = QPoly::new(vec![a.clone(), b.clone()]);
        let mut accumulator = QPoly::zero_poly();
        for coefficient in self.coeffs.iter().rev() {
            accumulator = accumulator.mul_poly_with_meter(&affine, budget, work)?;
            let mut coefficients = accumulator.coeffs;
            if coefficients.is_empty() {
                coefficients.push(coefficient.clone());
            } else {
                work.spend(1)?;
                let value = coefficients[0].add(coefficient);
                check_coefficient_bits(&value, budget)?;
                coefficients[0] = value;
            }
            accumulator = QPoly::new(coefficients);
            accumulator.validate_budget(budget)?;
        }
        Ok(accumulator)
    }

    /// `x^n · p(1/x)` — coefficient reversal (degree n = deg p).
    fn reverse(&self) -> QPoly {
        let mut c = self.coeffs.clone();
        c.reverse();
        QPoly::new(c)
    }

    /// Number of sign variations in the coefficient list (zeros skipped).
    fn variations(&self) -> usize {
        let mut count = 0;
        let mut last: Option<Sign> = None;
        for c in &self.coeffs {
            let s = c.sign();
            if s == Sign::Zero {
                continue;
            }
            if let Some(l) = last
                && l != s
            {
                count += 1;
            }
            last = Some(s);
        }
        count
    }

    /// Descartes bound on the number of roots in the OPEN interval
    /// `(lo, hi)`: `0` is proof of none; `1` on a square-free polynomial
    /// is proof of exactly one. Roots AT the endpoints are excluded.
    fn descartes_in(&self, lo: &Rational, hi: &Rational) -> usize {
        // Map (lo,hi) → (0,1): q(t) = p(lo + (hi−lo)t), then (0,1) → (0,∞)
        // via t = s/(1+s):  r(s) = (1+s)^n q(s/(1+s)) = rev(q(1−y))(1+s).
        let q = self.compose_affine(lo, &hi.sub(lo));
        let q1 = q.compose_affine(&Rational::one(), &Rational::from_i64(-1)); // q(1−y)
        let mut r = q1
            .reverse()
            .compose_affine(&Rational::one(), &Rational::one());
        // Strip s = 0 roots (they are roots at t = 0, i.e. at `lo`).
        let strip = r
            .coeffs
            .iter()
            .take_while(|c| c.sign() == Sign::Zero)
            .count();
        if strip > 0 {
            r = QPoly::new(r.coeffs[strip..].to_vec());
        }
        r.variations()
    }

    fn descartes_in_with_meter(
        &self,
        lo: &Rational,
        hi: &Rational,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<usize, AlgebraError> {
        work.spend(1)?;
        let span = hi.sub(lo);
        check_coefficient_bits(&span, budget)?;
        let q = self.compose_affine_with_meter(lo, &span, budget, work)?;
        let q1 =
            q.compose_affine_with_meter(&Rational::one(), &Rational::from_i64(-1), budget, work)?;
        work.spend(q1.coeffs.len())?;
        let reversed = q1.reverse();
        let mut transformed =
            reversed.compose_affine_with_meter(&Rational::one(), &Rational::one(), budget, work)?;
        let strip = transformed
            .coeffs
            .iter()
            .take_while(|coefficient| coefficient.sign() == Sign::Zero)
            .count();
        if strip > 0 {
            transformed = QPoly::new(transformed.coeffs[strip..].to_vec());
        }
        work.spend(transformed.coeffs.len())?;
        Ok(transformed.variations())
    }

    /// `1/L²`, where `L` is the leading coefficient of the integer-cleared
    /// polynomial. By the rational-root theorem the denominator `b` of any
    /// rational root divides `L`, and two distinct rationals with denominators
    /// `≤ L` differ by at least `1/L²` — so an interval narrower than this
    /// contains at most one candidate and, once that candidate is rejected,
    /// provably no rational root at all. `None` for the zero polynomial.
    fn rational_root_separation(&self) -> Option<Rational> {
        use dashu::integer::{IBig, UBig};
        let d = self.degree()?;
        // lcm of the denominators, then L = lead·lcm.
        let mut lcm = UBig::ONE;
        for i in 0..=d {
            let den = self.coeff(i).as_rbig().denominator().clone();
            let g = gcd_ubig(&lcm, &den);
            lcm = &lcm / g * den;
        }
        let lead = self.coeff(d);
        let rb = lead.as_rbig();
        let scale: IBig = IBig::from(lcm) / IBig::from(rb.denominator().clone());
        let l = rb.numerator() * scale;
        let l: UBig = UBig::try_from(if l.sign() == dashu::base::Sign::Negative {
            -l
        } else {
            l
        })
        .ok()?;
        if l == UBig::ZERO {
            return None;
        }
        Some(Rational::from(dashu::rational::RBig::from_parts(
            IBig::ONE,
            &l * &l,
        )))
    }

    /// Budgeted Cauchy root bound: every real root lies strictly inside
    /// `(-B, B)`. `None` only for the zero polynomial.
    fn cauchy_bound_with_meter(
        &self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<Option<Rational>, AlgebraError> {
        self.validate_budget(budget)?;
        let Some(lead) = self.coeffs.last() else {
            return Ok(None);
        };
        let mut maximum = Rational::zero();
        for coefficient in &self.coeffs[..self.coeffs.len() - 1] {
            work.spend(1)?;
            let ratio = coefficient.div(lead);
            check_coefficient_bits(&ratio, budget)?;
            let absolute = if ratio.sign() == Sign::Negative {
                ratio.neg()
            } else {
                ratio
            };
            if absolute > maximum {
                maximum = absolute;
            }
        }
        work.spend(1)?;
        let bound = Rational::one().add(&maximum);
        check_coefficient_bits(&bound, budget)?;
        Ok(Some(bound))
    }
}

fn determinant(
    mut matrix: Vec<Vec<Rational>>,
    budget: AlgebraBudget,
) -> Result<Rational, AlgebraError> {
    let mut work = AlgebraWork::new(budget, "computing a resultant determinant");
    let mut determinant = Rational::one();
    for column in 0..matrix.len() {
        let mut pivot = None;
        for (row, entries) in matrix.iter().enumerate().skip(column) {
            work.spend(1)?;
            if entries[column].sign() != Sign::Zero {
                pivot = Some(row);
                break;
            }
        }
        let Some(pivot) = pivot else {
            return Ok(Rational::zero());
        };
        if pivot != column {
            matrix.swap(pivot, column);
            determinant = determinant.neg();
        }
        let pivot_value = matrix[column][column].clone();
        work.spend(1)?;
        determinant = determinant.mul(&pivot_value);
        check_coefficient_bits(&determinant, budget)?;
        let pivot_row = matrix[column].clone();
        for row in matrix.iter_mut().skip(column + 1) {
            work.spend(1)?;
            let factor = row[column].div(&pivot_value);
            check_coefficient_bits(&factor, budget)?;
            for (entry, pivot_entry) in row.iter_mut().zip(&pivot_row).skip(column) {
                work.spend(2)?;
                let product = factor.mul(pivot_entry);
                check_coefficient_bits(&product, budget)?;
                *entry = entry.sub(&product);
                check_coefficient_bits(entry, budget)?;
            }
        }
    }
    Ok(determinant)
}

fn check_coefficient_bits(value: &Rational, budget: AlgebraBudget) -> Result<(), AlgebraError> {
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

fn map_root_algebra_error(error: AlgebraError, budget: AlgebraBudget) -> RootError {
    match error {
        AlgebraError::CoefficientBits { actual, limit } => {
            RootError::CoefficientBitsExceeded { actual, limit }
        }
        AlgebraError::PolynomialDegree { actual, limit } => {
            RootError::DegreeExceeded { actual, limit }
        }
        _ => RootError::WorkBudgetExceeded {
            limit: budget.max_expression_nodes,
        },
    }
}

fn spend_root_bisection(count: &mut usize, budget: AlgebraBudget) -> Result<(), RootError> {
    if *count >= budget.max_root_bisections {
        return Err(RootError::BudgetExceeded {
            limit: budget.max_root_bisections,
        });
    }
    *count += 1;
    Ok(())
}

/// How many extra bisections [`RealRoot::collapse_if_rational`] will spend
/// looking for an exact rational root before giving up.
///
/// Every step halves the interval, so this separates rational roots whose
/// denominator is up to roughly `2^24` on an O(1)-wide isolating interval —
/// far past anything a CAD coefficient produces — while keeping the cost of
/// the common irrational case to a bounded number of exact evaluations.
/// Overshooting the budget loses nothing but exactness of the *report*: the
/// interval still isolates the root.
pub const MAX_RATIONAL_COLLAPSE_STEPS: u32 = 48;

fn gcd_ubig(a: &dashu::integer::UBig, b: &dashu::integer::UBig) -> dashu::integer::UBig {
    use dashu::integer::UBig;
    let (mut x, mut y) = (a.clone(), b.clone());
    while y != UBig::ZERO {
        let t = &x % &y;
        x = y;
        y = t;
    }
    if x == UBig::ZERO { UBig::ONE } else { x }
}

/// `⌊x⌋`.
fn floor_int(x: &Rational) -> dashu::integer::IBig {
    use dashu::integer::IBig;
    let rb = x.as_rbig();
    let n = rb.numerator().clone();
    let d = IBig::from(rb.denominator().clone());
    let q = &n / &d;
    let r = n - &q * d;
    if r.sign() == dashu::base::Sign::Negative && r != IBig::ZERO {
        q - IBig::ONE
    } else {
        q
    }
}

fn from_int(i: dashu::integer::IBig) -> Rational {
    Rational::from(dashu::rational::RBig::from_parts(
        i,
        dashu::integer::UBig::ONE,
    ))
}

/// The rational of **least denominator** strictly inside `(lo, hi)`, by the
/// Stern–Brocot / continued-fraction descent. `None` when the interval is
/// empty. No factoring and no search: the answer is unique and is reached in
/// one continued-fraction expansion.
pub fn simplest_rational_in(lo: &Rational, hi: &Rational) -> Option<Rational> {
    if lo >= hi {
        return None;
    }
    let zero = Rational::zero();
    if *lo < zero && *hi > zero {
        return Some(zero);
    }
    Some(if *hi <= zero {
        simplest_nonneg(&hi.neg(), &lo.neg()).neg()
    } else {
        simplest_nonneg(lo, hi)
    })
}

fn simplest_rational_in_with_meter(
    lo: &Rational,
    hi: &Rational,
    budget: AlgebraBudget,
    work: &mut AlgebraWork,
) -> Result<Option<Rational>, AlgebraError> {
    check_coefficient_bits(lo, budget)?;
    check_coefficient_bits(hi, budget)?;
    work.spend(1)?;
    if lo >= hi {
        return Ok(None);
    }
    let zero = Rational::zero();
    if *lo < zero && *hi > zero {
        return Ok(Some(zero));
    }
    if *hi <= zero {
        work.spend(2)?;
        let value = simplest_nonneg_with_meter(&hi.neg(), &lo.neg(), budget, work)?.neg();
        check_coefficient_bits(&value, budget)?;
        Ok(Some(value))
    } else {
        Ok(Some(simplest_nonneg_with_meter(lo, hi, budget, work)?))
    }
}

fn simplest_nonneg_with_meter(
    lo: &Rational,
    hi: &Rational,
    budget: AlgebraBudget,
    work: &mut AlgebraWork,
) -> Result<Rational, AlgebraError> {
    use dashu::integer::IBig;
    work.spend(3)?;
    let floor = floor_int(lo);
    let floor_rational = from_int(floor.clone());
    let next = from_int(floor + IBig::ONE);
    check_coefficient_bits(&floor_rational, budget)?;
    check_coefficient_bits(&next, budget)?;
    if next < *hi {
        return Ok(next);
    }
    work.spend(1)?;
    let hi_fraction = hi.sub(&floor_rational);
    check_coefficient_bits(&hi_fraction, budget)?;
    let one = Rational::one();
    if *lo == floor_rational {
        work.spend(3)?;
        let reciprocal = one.div(&hi_fraction);
        check_coefficient_bits(&reciprocal, budget)?;
        let denominator = from_int(floor_int(&reciprocal) + IBig::ONE);
        let fraction = one.div(&denominator);
        check_coefficient_bits(&fraction, budget)?;
        let result = floor_rational.add(&fraction);
        check_coefficient_bits(&result, budget)?;
        return Ok(result);
    }
    work.spend(3)?;
    let lower_fraction = lo.sub(&floor_rational);
    check_coefficient_bits(&lower_fraction, budget)?;
    let inverted_hi = one.div(&hi_fraction);
    let inverted_lo = one.div(&lower_fraction);
    check_coefficient_bits(&inverted_hi, budget)?;
    check_coefficient_bits(&inverted_lo, budget)?;
    let inner = simplest_nonneg_with_meter(&inverted_hi, &inverted_lo, budget, work)?;
    work.spend(2)?;
    let reciprocal = one.div(&inner);
    check_coefficient_bits(&reciprocal, budget)?;
    let result = floor_rational.add(&reciprocal);
    check_coefficient_bits(&result, budget)?;
    Ok(result)
}

/// `simplest_rational_in` restricted to `0 ≤ lo < hi`.
fn simplest_nonneg(lo: &Rational, hi: &Rational) -> Rational {
    use dashu::integer::IBig;
    let fl = floor_int(lo);
    let flr = from_int(fl.clone());
    let next = from_int(fl + IBig::ONE);
    if next < *hi {
        return next; // an integer sits strictly inside
    }
    // Every x in (lo,hi) now shares the floor `fl`; write x = fl + 1/y.
    let one = Rational::one();
    let hi_f = hi.sub(&flr);
    if *lo == flr {
        // y ranges over (1/(hi−fl), ∞): take the least integer above it.
        let y = floor_int(&one.div(&hi_f)) + IBig::ONE;
        return flr.add(&one.div(&from_int(y)));
    }
    let y = simplest_nonneg(&one.div(&hi_f), &one.div(&lo.sub(&flr)));
    flr.add(&one.div(&y))
}

/// An isolated real root of a rational polynomial: an exact real
/// algebraic number. Either exactly rational (`lo == hi`) or the unique
/// root of the (square-free) `poly` in the open interval `(lo, hi)`,
/// with `poly(lo) ≠ 0 ≠ poly(hi)`.
///
/// Refining methods take `&mut self`: the interval shrinks, the number
/// never changes.
///
/// # The `f64` enclosure is readable through `&self`
///
/// [`RealRoot::to_interval`] fuses *readout* with *refinement*, so it needs
/// `&mut self` and a consumer that only has a shared reference cannot get an
/// enclosure at all — which is why the chart geometry used to reach for a
/// mutex on the sweep's inner loop. [`RealRoot::enclosure`] separates the
/// two: it reads a monotonically-tightened [`AtomicInterval`] maintained
/// alongside `(lo, hi)`, never locks and never blocks, and is what every
/// `try_*` filter below runs on.
#[derive(Clone, Debug)]
pub struct RealRoot {
    poly: QPoly,
    lo: Rational,
    hi: Rational,
    multiplicity: u32,
    /// `f64` shadow of `[lo, hi]`, refined in lock-step with it.
    ///
    /// Kept in sync by [`RealRoot::sync_enclosure`], which is called from
    /// every place `lo`/`hi` change. It is *derived* state — the rationals
    /// are the truth — so a clone that starts from a stale shadow would
    /// still be sound; syncing eagerly is what makes the filter useful.
    iv: AtomicInterval,
}

/// Immutable, canonical projection of a certified real algebraic root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RealRootCertificate {
    /// Resolvent-owned wire schema identifier.
    pub schema: String,
    /// Square-free polynomial defining the root.
    pub polynomial: QPoly,
    /// Exact lower isolating bound.
    pub lower: Rational,
    /// Exact upper isolating bound.
    pub upper: Rational,
    /// Multiplicity in the polynomial originally isolated.
    pub multiplicity: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RealRootCertificateWire {
    schema: String,
    polynomial: QPoly,
    lower: Rational,
    upper: Rational,
    multiplicity: u32,
}

impl<'de> Deserialize<'de> for RealRootCertificate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = RealRootCertificateWire::deserialize(deserializer)?;
        let certificate = RealRootCertificate {
            schema: wire.schema,
            polynomial: wire.polynomial,
            lower: wire.lower,
            upper: wire.upper,
            multiplicity: wire.multiplicity,
        };
        validate_certificate_envelope(&certificate, AlgebraBudget::default())
            .map_err(serde::de::Error::custom)?;
        Ok(certificate)
    }
}

fn validate_certificate_envelope(
    certificate: &RealRootCertificate,
    budget: AlgebraBudget,
) -> Result<(), RootCertificateError> {
    if certificate.schema != "resolvent-real-root-certificate/1" {
        return Err(RootCertificateError {
            reason: "unsupported certificate schema",
        });
    }
    if certificate.polynomial.is_zero() {
        return Err(RootCertificateError {
            reason: "zero defining polynomial",
        });
    }
    let degree = certificate.polynomial.degree().unwrap_or(0);
    if degree > budget.max_polynomial_degree {
        return Err(RootCertificateError {
            reason: "defining polynomial exceeds degree budget",
        });
    }
    if degree
        .checked_mul(degree)
        .and_then(|work| work.checked_mul(degree.max(1)))
        .is_none_or(|work| work > budget.max_expression_nodes)
    {
        return Err(RootCertificateError {
            reason: "certificate validation exceeds work budget",
        });
    }
    if certificate
        .polynomial
        .coefficients()
        .iter()
        .any(|coefficient| coefficient.bit_size() > budget.max_coefficient_bits)
    {
        return Err(RootCertificateError {
            reason: "defining polynomial exceeds coefficient budget",
        });
    }
    let bound_bits = certificate
        .lower
        .bit_size()
        .max(certificate.upper.bit_size());
    if bound_bits > budget.max_coefficient_bits {
        return Err(RootCertificateError {
            reason: "isolating bound exceeds coefficient budget",
        });
    }
    if certificate.multiplicity == 0 {
        return Err(RootCertificateError {
            reason: "zero multiplicity",
        });
    }
    if certificate.lower > certificate.upper {
        return Err(RootCertificateError {
            reason: "reversed isolating bounds",
        });
    }
    Ok(())
}

/// Isolate all real roots of `p`, in ascending order, with their
/// multiplicities in `p`. Errors on the zero polynomial; constants have
/// no roots.
pub fn isolate_roots(p: &QPoly) -> Result<Vec<RealRoot>, RootError> {
    isolate_roots_with_budget(p, AlgebraBudget::default())
}

/// Isolate all real roots under an explicit bisection budget.
pub fn isolate_roots_with_budget(
    p: &QPoly,
    budget: AlgebraBudget,
) -> Result<Vec<RealRoot>, RootError> {
    if p.is_zero() {
        return Err(RootError::ZeroPolynomial);
    }
    if p.degree() == Some(0) {
        return Ok(Vec::new());
    }
    let degree = p.degree().unwrap_or(0);
    if degree > budget.max_polynomial_degree {
        return Err(RootError::DegreeExceeded {
            actual: degree,
            limit: budget.max_polynomial_degree,
        });
    }
    let coefficient_bits = p
        .coefficients()
        .iter()
        .map(Rational::bit_size)
        .max()
        .unwrap_or(0);
    if coefficient_bits > budget.max_coefficient_bits {
        return Err(RootError::CoefficientBitsExceeded {
            actual: coefficient_bits,
            limit: budget.max_coefficient_bits,
        });
    }
    let mut work = AlgebraWork::new(budget, "isolating polynomial roots");
    let decomp = p
        .square_free_decomposition_with_meter(budget, &mut work)
        .map_err(|error| map_root_algebra_error(error, budget))?;
    for (_, factor) in &decomp {
        factor
            .validate_budget(budget)
            .map_err(|error| map_root_algebra_error(error, budget))?;
    }
    let sf = p
        .square_free_part_with_meter(budget, &mut work)
        .map_err(|error| map_root_algebra_error(error, budget))?;
    sf.validate_budget(budget)
        .map_err(|error| map_root_algebra_error(error, budget))?;
    let mut roots: Vec<RealRoot> = Vec::new();
    let mut bisections = 0;
    isolate_square_free(&sf, &mut roots, &mut bisections, budget, &mut work)?;
    // Sort ascending. Every interval comes from one bisection tree over one
    // Cauchy bound (see `isolate_square_free`), so they are pairwise disjoint
    // up to shared endpoints and `(lo, hi)` is a total order on them.
    //
    // Mixing keys — an exact root's value against an isolated root's upper
    // bound — is only valid under that disjointness, and reads as correct
    // right up until it isn't.
    work.spend(roots.len().saturating_mul(roots.len()))
        .map_err(|error| map_root_algebra_error(error, budget))?;
    roots.sort_by(|a, b| a.lo.cmp(&b.lo).then_with(|| a.hi.cmp(&b.hi)));
    // Collapse the roots that are exactly rational. Bisection alone only
    // notices one when a midpoint happens to land on it, which for a Cauchy
    // bound with an odd numerator essentially never happens — so `is_exact()`
    // used to be silently false for rational roots and every consumer that
    // asks "is this pencil parameter rational?" degraded quietly. Collapsing
    // moves each root strictly inside its own interval, so the ascending
    // order established above is preserved.
    for r in &mut roots {
        r.collapse_if_rational_with_meter(budget, &mut work, &mut bisections)?;
    }
    // Multiplicities from the Yun factors: exactly one factor vanishes
    // at each root.
    for r in &mut roots {
        for (m, f) in &decomp {
            if r.is_root_of_with_meter(f, budget, &mut work)
                .map_err(|error| map_root_algebra_error(error, budget))?
            {
                r.multiplicity = *m;
                break;
            }
        }
    }
    Ok(roots)
}

/// VCA bisection on a square-free polynomial; exact rational roots met at
/// midpoints are deflated out and recorded exactly.
fn isolate_square_free(
    p: &QPoly,
    out: &mut Vec<RealRoot>,
    bisections: &mut usize,
    budget: AlgebraBudget,
    work: &mut AlgebraWork,
) -> Result<(), RootError> {
    if p.degree().unwrap_or(0) == 0 {
        return Ok(());
    }
    // Unreachable given the degree check above (the zero polynomial reports
    // `degree() == None`, so `unwrap_or(0)` already returned); spelled as a
    // pattern rather than an unwrap so no panic exists to reason about.
    let Some(bound) = p
        .cauchy_bound_with_meter(budget, work)
        .map_err(|error| map_root_algebra_error(error, budget))?
    else {
        return Ok(());
    };
    let mut stack: Vec<(Rational, Rational)> = vec![(bound.neg(), bound)];
    let half = Rational::from_ratio(1, 2);
    // Deflation is IN PLACE: an exact rational root met at a midpoint is
    // divided out and the SAME bisection continues over the SAME stack.
    //
    // Restarting instead — recursing over the deflated `q`'s own Cauchy
    // bound — is unsound twice over. `q`'s roots are `p`'s roots minus `mid`,
    // which still INCLUDES every root already isolated from an earlier stack
    // entry, so the recursion re-isolates them: duplicates, comparing `Equal`
    // over overlapping intervals that no refinement can separate. And the
    // fresh bound makes those intervals straddle the roots already in `out`,
    // which destroys the pairwise-disjointness the caller's ascending sort
    // depends on.
    //
    // Roots already pushed keep the less-deflated polynomial in `poly`. That
    // stays correct: their interval still isolates exactly one of its roots,
    // so `refine` bisects on the right sign changes.
    let mut p = p.clone();
    while let Some((lo, hi)) = stack.pop() {
        match p
            .descartes_in_with_meter(&lo, &hi, budget, work)
            .map_err(|error| map_root_algebra_error(error, budget))?
        {
            0 => {}
            1 => out.push(RealRoot::from_parts(p.clone(), lo, hi, 1)),
            _ => {
                spend_root_bisection(bisections, budget)?;
                work.spend(2)
                    .map_err(|error| map_root_algebra_error(error, budget))?;
                let mid = lo.add(&hi).mul(&half);
                check_coefficient_bits(&mid, budget)
                    .map_err(|error| map_root_algebra_error(error, budget))?;
                if p.eval_with_meter(&mid, budget, work)
                    .map_err(|error| map_root_algebra_error(error, budget))?
                    .sign()
                    == Sign::Zero
                {
                    // Exact rational root: record and deflate in place.
                    out.push(RealRoot::from_parts(p.clone(), mid.clone(), mid.clone(), 1));
                    let linear = QPoly::new(vec![mid.neg(), Rational::one()]);
                    let (q, r) = p
                        .divrem_with_meter(&linear, budget, work)
                        .map_err(|error| map_root_algebra_error(error, budget))?;
                    debug_assert!(r.is_zero());
                    p = q;
                    if p.degree().unwrap_or(0) == 0 {
                        return Ok(()); // nothing left to find
                    }
                }
                // `mid` lies strictly inside this interval and therefore in no
                // other pending one, so deflating it cannot change any other
                // entry's root count.
                stack.push((lo, mid.clone()));
                stack.push((mid, hi));
            }
        }
    }
    Ok(())
}

/// The `f64` enclosure of `[lo, hi]`, outward-correct on both ends.
fn bounds_interval(lo: &Rational, hi: &Rational) -> Interval {
    Interval::new(lo.to_interval().inf(), hi.to_interval().sup())
}

impl RealRoot {
    /// The one place a `RealRoot` is built: keeps the `f64` shadow in sync
    /// with `(lo, hi)` by construction.
    fn from_parts(poly: QPoly, lo: Rational, hi: Rational, multiplicity: u32) -> RealRoot {
        let iv = AtomicInterval::new(bounds_interval(&lo, &hi));
        RealRoot {
            poly,
            lo,
            hi,
            multiplicity,
            iv,
        }
    }

    /// Project mutable refinement state into a stable immutable certificate.
    pub fn certificate(&self) -> RealRootCertificate {
        RealRootCertificate {
            schema: "resolvent-real-root-certificate/1".into(),
            polynomial: self.poly.clone(),
            lower: self.lo.clone(),
            upper: self.hi.clone(),
            multiplicity: self.multiplicity,
        }
    }

    /// Validate and restore an immutable root certificate.
    pub fn from_certificate(
        certificate: RealRootCertificate,
    ) -> Result<RealRoot, RootCertificateError> {
        Self::from_certificate_with_budget(certificate, AlgebraBudget::default())
    }

    /// Validate and restore a certificate under explicit algebra limits.
    pub fn from_certificate_with_budget(
        certificate: RealRootCertificate,
        budget: AlgebraBudget,
    ) -> Result<RealRoot, RootCertificateError> {
        validate_certificate_envelope(&certificate, budget)?;
        let mut work = AlgebraWork::new(budget, "restoring a root certificate");
        let map_algebra_error = |error: AlgebraError| RootCertificateError {
            reason: match error {
                AlgebraError::CoefficientBits { .. } => {
                    "certificate restoration exceeds coefficient budget"
                }
                _ => "certificate restoration exceeds work budget",
            },
        };
        if certificate.lower == certificate.upper {
            if certificate
                .polynomial
                .eval_with_meter(&certificate.lower, budget, &mut work)
                .map_err(map_algebra_error)?
                .sign()
                != Sign::Zero
            {
                return Err(RootCertificateError {
                    reason: "point bound is not a polynomial root",
                });
            }
        } else {
            if certificate
                .polynomial
                .eval_with_meter(&certificate.lower, budget, &mut work)
                .map_err(map_algebra_error)?
                .sign()
                == Sign::Zero
                || certificate
                    .polynomial
                    .eval_with_meter(&certificate.upper, budget, &mut work)
                    .map_err(map_algebra_error)?
                    .sign()
                    == Sign::Zero
            {
                return Err(RootCertificateError {
                    reason: "non-point interval has a root on its boundary",
                });
            }
            if certificate
                .polynomial
                .descartes_in_with_meter(&certificate.lower, &certificate.upper, budget, &mut work)
                .map_err(map_algebra_error)?
                != 1
            {
                return Err(RootCertificateError {
                    reason: "interval does not isolate exactly one root",
                });
            }
        }
        Ok(RealRoot::from_parts(
            certificate.polynomial,
            certificate.lower,
            certificate.upper,
            certificate.multiplicity,
        ))
    }

    /// Re-derive the `f64` shadow after `lo`/`hi` moved.
    ///
    /// Skipped once the shadow is a single `f64`: at that point `lo` and `hi`
    /// bracket one double, so the number is pinned exactly (see
    /// [`RealRoot::enclosure`]) and no further bisection can improve it — the
    /// two `Rational::to_interval` conversions per bisection step would be
    /// pure cost.
    fn sync_enclosure(&mut self) {
        if self.iv.load().is_point() {
            return;
        }
        self.iv.refine(bounds_interval(&self.lo, &self.hi));
    }

    /// The current `f64` enclosure — **no lock, no blocking, no refinement.**
    ///
    /// `ξ ∈ enclosure()` always holds: every bound stored in the shadow comes
    /// from an outward-correct conversion of an isolating bound, and
    /// [`AtomicInterval`] only ever intersects, so no store can exclude ξ.
    ///
    /// A *point* result is exact information, not a rounding artefact: the
    /// shadow is `[fl(lo), fl(hi)]` with `fl(lo) ≤ lo` and `hi ≤ fl(hi)`, so
    /// `fl(lo) == fl(hi) == d` forces `d ≤ lo ≤ ξ ≤ hi ≤ d`, i.e. `ξ = d`.
    /// That is what lets [`Interval::cmp_interval`] certify `Equal` off the
    /// filter rung.
    pub fn enclosure(&self) -> Interval {
        self.iv.load()
    }

    /// Filter rung of [`RealRoot::sign_of`]: the sign of `h` at this root
    /// when interval evaluation over [`RealRoot::enclosure`] already excludes
    /// zero.
    pub fn try_sign_of(&self, h: &QPoly) -> USign {
        if h.is_zero() {
            return Uncertain::Certain(Sign::Zero);
        }
        h.eval_interval(self.enclosure()).sign()
    }

    /// Filter rung of [`RealRoot::cmp_rational`].
    pub fn try_cmp_rational(&self, r: &Rational) -> UOrd {
        self.enclosure().cmp_interval(r.to_interval())
    }

    /// Filter rung of [`RealRoot::cmp_root`]: disjoint enclosures decide the
    /// order outright.
    pub fn try_cmp_root(&self, other: &RealRoot) -> UOrd {
        self.enclosure().cmp_interval(other.enclosure())
    }

    /// An exact rational value as a (degenerate) `RealRoot` — the root of
    /// `x − r` with a point interval. Lets rational and algebraic
    /// coordinates share one type.
    pub fn exact_rational(r: Rational) -> RealRoot {
        let poly = QPoly::new(vec![r.neg(), Rational::one()]);
        RealRoot::from_parts(poly, r.clone(), r, 1)
    }

    /// Is the root exactly rational?
    pub fn is_exact(&self) -> bool {
        self.lo == self.hi
    }

    /// The exact rational value, if the root is rational.
    pub fn value(&self) -> Option<&Rational> {
        self.is_exact().then_some(&self.lo)
    }

    /// Lower bound (equal to the root when exact; otherwise strict).
    pub fn lo(&self) -> &Rational {
        &self.lo
    }

    /// Upper bound.
    pub fn hi(&self) -> &Rational {
        &self.hi
    }

    /// Multiplicity in the polynomial handed to [`isolate_roots`].
    pub fn multiplicity(&self) -> u32 {
        self.multiplicity
    }

    /// The square-free defining polynomial.
    pub fn defining_poly(&self) -> &QPoly {
        &self.poly
    }

    /// One bisection step (no-op on exact roots). The invariant
    /// `poly(lo) ≠ 0 ≠ poly(hi)` is preserved; an exact midpoint hit
    /// collapses the root to a point.
    pub fn refine(&mut self) {
        let budget = AlgebraBudget {
            max_expression_nodes: usize::MAX,
            max_polynomial_degree: usize::MAX,
            max_coefficient_bits: u64::MAX,
            max_root_bisections: usize::MAX,
            ..AlgebraBudget::default()
        };
        let mut work = AlgebraWork::new(budget, "refining a real root");
        self.refine_with_meter(budget, &mut work)
            .expect("unbounded refinement of a valid RealRoot cannot fail");
    }

    fn refine_with_meter(
        &mut self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<(), AlgebraError> {
        if self.is_exact() {
            return Ok(());
        }
        work.spend(2)?;
        let mid = self.lo.add(&self.hi).mul(&Rational::from_ratio(1, 2));
        check_coefficient_bits(&mid, budget)?;
        let mid_sign = self.poly.eval_with_meter(&mid, budget, work)?.sign();
        match mid_sign {
            Sign::Zero => {
                self.lo = mid.clone();
                self.hi = mid;
            }
            _ => {
                // The root is on the side where the sign flips vs lo.
                let lo_sign = self.poly.eval_with_meter(&self.lo, budget, work)?.sign();
                if lo_sign == mid_sign {
                    self.lo = mid;
                } else {
                    self.hi = mid;
                }
            }
        }
        self.sync_enclosure();
        Ok(())
    }

    /// Refine until the interval is narrower than `width` (or exact).
    ///
    /// # Panics
    ///
    /// Panics when `width` is not strictly positive. Untrusted widths and
    /// bounded workloads must use [`RealRoot::refine_to_width_with_budget`].
    pub fn refine_to_width(&mut self, width: &Rational) {
        self.refine_to_width_with_budget(
            width,
            AlgebraBudget {
                max_expression_nodes: usize::MAX,
                max_polynomial_degree: usize::MAX,
                max_coefficient_bits: u64::MAX,
                max_root_bisections: usize::MAX,
                ..AlgebraBudget::default()
            },
        )
        .expect("RealRoot::refine_to_width requires a strictly positive width");
    }

    /// Refine to a strictly positive width under explicit bisection, exact
    /// arithmetic work, degree, and intermediate coefficient limits.
    pub fn refine_to_width_with_budget(
        &mut self,
        width: &Rational,
        budget: AlgebraBudget,
    ) -> Result<(), AlgebraError> {
        if width.sign() != Sign::Positive {
            return Err(AlgebraError::NonPositiveRefinementWidth);
        }
        check_coefficient_bits(width, budget)?;
        self.poly.validate_budget(budget)?;
        let mut work = AlgebraWork::new(budget, "refining a real root");
        let mut bisections = 0usize;
        while !self.is_exact() {
            work.spend(1)?;
            let current_width = self.hi.sub(&self.lo);
            check_coefficient_bits(&current_width, budget)?;
            if current_width < *width {
                break;
            }
            if bisections >= budget.max_root_bisections {
                return Err(AlgebraError::BudgetExceeded {
                    operation: "refining a real root",
                    limit: budget.max_root_bisections,
                });
            }
            self.refine_with_meter(budget, &mut work)?;
            bisections += 1;
        }
        Ok(())
    }

    /// Collapse the isolating interval to an exact rational value when the
    /// root **is** rational; `true` iff the root is now exact.
    ///
    /// Method: the unique rational of least denominator inside the interval
    /// (continued fractions / Stern–Brocot — no factoring, no rational-root
    /// theorem enumeration) is tested by exact evaluation, and the interval is
    /// bisected and retested while a rational root could still be hiding.
    ///
    /// **Sound in one direction only, by construction.** The candidate is
    /// accepted only when `poly(candidate)` is exactly zero, so this can never
    /// fabricate a root; it can only fail to find one. That makes it a strict
    /// improvement over reporting every rational root as an interval.
    ///
    /// Termination: a root `a/b` in lowest terms is the unique least-denominator
    /// rational of any interval around it narrower than `1/b²`, and `b` divides
    /// the leading coefficient of the integer-cleared polynomial, so the search
    /// is bounded. The bound is additionally capped at
    /// [`MAX_RATIONAL_COLLAPSE_STEPS`] bisections to keep the cost of the
    /// (overwhelmingly common) irrational case bounded; a root whose
    /// denominator exceeds what that many bisections can separate is simply
    /// left as an interval.
    pub fn collapse_if_rational(&mut self) -> bool {
        if self.is_exact() {
            return true;
        }
        // b | lead(integer-cleared poly), so 1/lead² is a sufficient width.
        let stop_width = self.poly.rational_root_separation();
        for _ in 0..MAX_RATIONAL_COLLAPSE_STEPS {
            if let Some(c) = simplest_rational_in(&self.lo, &self.hi)
                && self.poly.sign_at(&c) == Sign::Zero
            {
                self.lo = c.clone();
                self.hi = c;
                self.sync_enclosure();
                return true;
            }
            if let Some(w) = &stop_width
                && self.hi.sub(&self.lo) < *w
            {
                return false; // narrower than any rational root could hide in
            }
            self.refine();
            if self.is_exact() {
                return true;
            }
        }
        false
    }

    fn collapse_if_rational_with_meter(
        &mut self,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
        bisections: &mut usize,
    ) -> Result<bool, RootError> {
        if self.is_exact() {
            return Ok(true);
        }
        for _ in 0..MAX_RATIONAL_COLLAPSE_STEPS {
            if let Some(candidate) =
                simplest_rational_in_with_meter(&self.lo, &self.hi, budget, work)
                    .map_err(|error| map_root_algebra_error(error, budget))?
                && self
                    .poly
                    .eval_with_meter(&candidate, budget, work)
                    .map_err(|error| map_root_algebra_error(error, budget))?
                    .sign()
                    == Sign::Zero
            {
                self.lo = candidate.clone();
                self.hi = candidate;
                self.sync_enclosure();
                return Ok(true);
            }
            spend_root_bisection(bisections, budget)?;
            self.refine_with_meter(budget, work)
                .map_err(|error| map_root_algebra_error(error, budget))?;
            if self.is_exact() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Is this root also a root of `h`? Decided algebraically (gcd +
    /// sign-change certificate) — no numeric tolerance anywhere.
    pub fn is_root_of(&self, h: &QPoly) -> bool {
        if h.is_zero() {
            return true;
        }
        if let Some(v) = self.value() {
            return h.sign_at(v) == Sign::Zero;
        }
        let g = self.poly.gcd(h);
        if g.degree().unwrap_or(0) == 0 {
            return false;
        }
        // g | poly ⇒ g has at most one root in (lo,hi) (the candidate α)
        // and g(lo) ≠ 0 ≠ g(hi); a sign change is then exact evidence.
        g.sign_at(&self.lo) != g.sign_at(&self.hi)
    }

    fn is_root_of_with_meter(
        &self,
        polynomial: &QPoly,
        budget: AlgebraBudget,
        work: &mut AlgebraWork,
    ) -> Result<bool, AlgebraError> {
        if polynomial.is_zero() {
            return Ok(true);
        }
        if let Some(value) = self.value() {
            return Ok(polynomial.eval_with_meter(value, budget, work)?.sign() == Sign::Zero);
        }
        let gcd = self.poly.gcd_with_meter(polynomial, budget, work)?;
        if gcd.degree().unwrap_or(0) == 0 {
            return Ok(false);
        }
        let lower_sign = gcd.eval_with_meter(&self.lo, budget, work)?.sign();
        let upper_sign = gcd.eval_with_meter(&self.hi, budget, work)?.sign();
        Ok(lower_sign != upper_sign)
    }

    /// The exact multiplicity of this root in `h`: the least `m` with
    /// `h^(m)(ξ) ≠ 0`. `u32::MAX` for the zero polynomial (every order
    /// vanishes). Every step is a certified exact sign, no tolerance.
    pub fn multiplicity_of(&mut self, h: &QPoly) -> u32 {
        if h.is_zero() {
            return u32::MAX;
        }
        let mut q = h.clone();
        let mut m = 0u32;
        loop {
            if self.sign_of(&q) != Sign::Zero {
                return m;
            }
            if q.is_zero() {
                return u32::MAX;
            }
            q = q.derivative();
            m += 1;
        }
    }

    /// Exact sign of `h` evaluated at this root.
    ///
    /// Certify-or-escalate: [`RealRoot::try_sign_of`] first, and only on
    /// `Unknown` the exact rung below — which starts with a gcd
    /// ([`RealRoot::is_root_of`]) and can bisect, so skipping it is the
    /// single biggest saving in the whole ladder.
    pub fn sign_of(&mut self, h: &QPoly) -> Sign {
        if let Uncertain::Certain(s) = self.try_sign_of(h) {
            crate::metrics::root(true);
            return s;
        }
        crate::metrics::root(false);
        if h.is_zero() {
            return Sign::Zero;
        }
        if let Some(v) = self.value() {
            return h.sign_at(v);
        }
        if self.is_root_of(h) {
            return Sign::Zero;
        }
        // h(α) ≠ 0: shrink until h has provably no root in the interval,
        // where its sign is constant.
        loop {
            if h.descartes_in(&self.lo, &self.hi) == 0 {
                let mid = self.lo.add(&self.hi).mul(&Rational::from_ratio(1, 2));
                let s = h.sign_at(&mid);
                if s != Sign::Zero {
                    return s;
                }
                // mid happens to be a root of h AT the boundary of
                // detectability — refine and retry.
            }
            self.refine();
            if let Some(v) = self.value() {
                return h.sign_at(v);
            }
        }
    }

    /// Compare with an exact rational. Filter rung first
    /// ([`RealRoot::try_cmp_rational`]), then exact bisection.
    pub fn cmp_rational(&mut self, r: &Rational) -> Ordering {
        if let Uncertain::Certain(o) = self.try_cmp_rational(r) {
            crate::metrics::root(true);
            return o;
        }
        crate::metrics::root(false);
        loop {
            if let Some(v) = self.value() {
                return v.cmp(r);
            }
            if *r <= self.lo {
                return Ordering::Greater; // α > lo ≥ r
            }
            if *r >= self.hi {
                return Ordering::Less;
            }
            // r is strictly inside: equal iff r is THE root here.
            if self.poly.sign_at(r) == Sign::Zero {
                return Ordering::Equal;
            }
            self.refine();
        }
    }

    /// Compare two isolated roots (possibly of different polynomials).
    /// Equality is decided algebraically through the gcd; distinct roots
    /// are separated by refinement.
    pub fn cmp_root(&mut self, other: &mut RealRoot) -> Ordering {
        if let Uncertain::Certain(o) = self.try_cmp_root(other) {
            crate::metrics::root(true);
            return o;
        }
        crate::metrics::root(false);
        if let Some(v) = other.value().cloned() {
            return self.cmp_rational(&v);
        }
        if let Some(v) = self.value().cloned() {
            return other.cmp_rational(&v).reverse();
        }
        // Equality test once: a common root inside the overlap equals
        // both isolated roots.
        let g = self.poly.gcd(&other.poly);
        loop {
            if self.hi <= other.lo {
                return Ordering::Less;
            }
            if other.hi <= self.lo {
                return Ordering::Greater;
            }
            let olo = if self.lo > other.lo {
                self.lo.clone()
            } else {
                other.lo.clone()
            };
            let ohi = if self.hi < other.hi {
                self.hi.clone()
            } else {
                other.hi.clone()
            };
            if g.degree().unwrap_or(0) > 0 {
                // g divides both square-free defining polynomials: at most
                // one root of g in the overlap, endpoints nonzero for g on
                // each own interval — but overlap endpoints mix the two, so
                // certify via sign change only when both ends are nonzero.
                let slo = g.sign_at(&olo);
                let shi = g.sign_at(&ohi);
                if slo != Sign::Zero && shi != Sign::Zero && slo != shi {
                    return Ordering::Equal;
                }
            }
            self.refine();
            other.refine();
            if let Some(v) = other.value().cloned() {
                return self.cmp_rational(&v);
            }
            if let Some(v) = self.value().cloned() {
                return other.cmp_rational(&v).reverse();
            }
        }
    }

    /// An outward-correct double enclosure, **refining first**: the readout
    /// that costs work. [`RealRoot::enclosure`] is the same enclosure read
    /// through `&self` without refining, and is what filters use.
    pub fn to_interval(&mut self) -> Interval {
        if let Some(v) = self.value() {
            let iv = v.to_interval();
            self.iv.refine(iv);
            return iv;
        }
        // 2^-60 absolute window (coordinates in cadabra's UV charts are
        // O(1); absolute refinement is the honest simple choice).
        let width = Rational::from_ratio(1, 1 << 60);
        self.refine_to_width(&width);
        let iv = bounds_interval(&self.lo, &self.hi);
        self.iv.refine(iv);
        iv
    }
}

/// Interval enclosure of `√h(x)` over `x ∈ X`, given the caller's promise
/// that `h(ξ) ≥ 0`. `None` when the enclosure of `h` is certainly negative —
/// which means the promise does not hold on this enclosure, so nothing can be
/// certified and the caller must escalate.
fn sqrt_enclosure(h: &QPoly, x: Interval) -> Option<Interval> {
    let hv = h.eval_interval(x);
    (hv.sup() >= 0.0).then(|| hv.sqrt())
}

/// Filter rung of [`sign_radical1`]: interval-evaluate `a + b·√h` over an
/// enclosure of ξ. Certain iff the resulting interval excludes zero.
pub fn try_sign_radical1_at(x: Interval, a: &QPoly, b: &QPoly, h: &QPoly) -> USign {
    let Some(sh) = sqrt_enclosure(h, x) else {
        return Uncertain::Unknown;
    };
    (a.eval_interval(x) + b.eval_interval(x) * sh).sign()
}

/// Filter rung of [`sign_radical2`]: interval-evaluate
/// `a + b·√h1 + c·√h2` over an enclosure of ξ.
pub fn try_sign_radical2_at(
    x: Interval,
    a: &QPoly,
    b: &QPoly,
    h1: &QPoly,
    c: &QPoly,
    h2: &QPoly,
) -> USign {
    let (Some(s1), Some(s2)) = (sqrt_enclosure(h1, x), sqrt_enclosure(h2, x)) else {
        return Uncertain::Unknown;
    };
    (a.eval_interval(x) + b.eval_interval(x) * s1 + c.eval_interval(x) * s2).sign()
}

/// Exact sign of the one-radical expression `a(ξ) + b(ξ)·√h(ξ)` at the
/// isolated root ξ. Pre: `h(ξ) ≥ 0` (caller guarantees the radicand —
/// e.g. a conic discriminant inside the arc's x-range).
///
/// Every conic-arc predicate reduces to this ladder (and its two-radical
/// sibling). [`try_sign_radical1_at`] is the filter rung — one interval
/// evaluation of the *whole* expression; only on `Unknown` does the exact
/// rung run, where signs of `a`, `b`, `h` at ξ decide directly when they
/// agree and otherwise the squared difference `a² − b²·h` settles it — all
/// univariate signs at ξ, each exact via [`RealRoot::sign_of`].
pub fn sign_radical1(xi: &mut RealRoot, a: &QPoly, b: &QPoly, h: &QPoly) -> Sign {
    if let Uncertain::Certain(s) = try_sign_radical1_at(xi.enclosure(), a, b, h) {
        crate::metrics::radical(true);
        return s;
    }
    crate::metrics::radical(false);
    let sh = xi.sign_of(h);
    debug_assert_ne!(sh, Sign::Negative, "radicand must be non-negative at ξ");
    let sb = xi.sign_of(b);
    if sh == Sign::Zero || sb == Sign::Zero {
        return xi.sign_of(a);
    }
    let sa = xi.sign_of(a);
    if sa == Sign::Zero {
        return sb; // sign(b·√h) with √h > 0
    }
    if sa == sb {
        return sa;
    }
    // Opposite signs: |a| vs |b|√h ⇔ sign(a² − b²h) carried by sign(a).
    let a2 = a.mul_poly(a);
    let b2h = b.mul_poly(b).mul_poly(h);
    match xi.sign_of(&a2.sub_poly(&b2h)) {
        Sign::Zero => Sign::Zero,
        Sign::Positive => sa,
        Sign::Negative => sb,
    }
}

/// Exact sign of the two-radical expression
/// `a(ξ) + b(ξ)·√h1(ξ) + c(ξ)·√h2(ξ)` at ξ. Pre: `h1(ξ) ≥ 0`,
/// `h2(ξ) ≥ 0`. One squaring reduces to [`sign_radical1`].
pub fn sign_radical2(
    xi: &mut RealRoot,
    a: &QPoly,
    b: &QPoly,
    h1: &QPoly,
    c: &QPoly,
    h2: &QPoly,
) -> Sign {
    if let Uncertain::Certain(s) = try_sign_radical2_at(xi.enclosure(), a, b, h1, c, h2) {
        crate::metrics::radical(true);
        return s;
    }
    crate::metrics::radical(false);
    if xi.sign_of(c) == Sign::Zero || xi.sign_of(h2) == Sign::Zero {
        return sign_radical1(xi, a, b, h1);
    }
    if xi.sign_of(b) == Sign::Zero || xi.sign_of(h1) == Sign::Zero {
        return sign_radical1(xi, a, c, h2);
    }
    let left = sign_radical1(xi, a, b, h1); // sign(a + b√h1)
    let sc = xi.sign_of(c); // sign(c√h2), √h2 > 0 here
    if left == Sign::Zero {
        return sc;
    }
    if sc == Sign::Zero || left == sc {
        return left;
    }
    // Opposite: |a + b√h1| vs |c|√h2 ⇔ sign((a + b√h1)² − c²h2), itself a
    // one-radical expression: (a² + b²h1 − c²h2) + (2ab)·√h1.
    let alpha = a
        .mul_poly(a)
        .add_poly(&b.mul_poly(b).mul_poly(h1))
        .sub_poly(&c.mul_poly(c).mul_poly(h2));
    let beta = a.mul_poly(b).scale(&Rational::from_i64(2));
    match sign_radical1(xi, &alpha, &beta, h1) {
        Sign::Zero => Sign::Zero,
        Sign::Positive => left,
        Sign::Negative => sc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rat(n: i64, d: i64) -> Rational {
        Rational::from_ratio(n, d)
    }

    #[test]
    fn quadratic_sqrt2() {
        // x² − 2: roots ±√2.
        let p = QPoly::from_i64s(&[-2, 0, 1]);
        let mut roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].multiplicity(), 1);
        let iv = roots[1].to_interval();
        assert!(iv.contains(std::f64::consts::SQRT_2));
        assert!(roots[0].to_interval().sup() < 0.0);
        // sign_of: x − 1 at √2 is positive; x − 2 negative; x² − 2 zero.
        assert_eq!(
            roots[1].sign_of(&QPoly::from_i64s(&[-1, 1])),
            Sign::Positive
        );
        assert_eq!(
            roots[1].sign_of(&QPoly::from_i64s(&[-2, 1])),
            Sign::Negative
        );
        assert_eq!(roots[1].sign_of(&p), Sign::Zero);
    }

    #[test]
    fn quartic_two_radicals() {
        // (x² − 2)(x² − 3): four roots ±√2, ±√3, ascending.
        let p = QPoly::from_i64s(&[6, 0, -5, 0, 1]);
        let mut roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 4);
        for w in [3f64.sqrt(), 2f64.sqrt()] {
            assert!(roots.iter_mut().any(|r| r.to_interval().contains(w)));
        }
        // Ordering is ascending.
        for i in 0..3 {
            let (a, b) = roots.split_at_mut(i + 1);
            assert_eq!(a.last_mut().unwrap().cmp_root(&mut b[0]), Ordering::Less);
        }
    }

    #[test]
    fn multiplicities() {
        // (x − 1)²(x + 2): roots −2 (simple), 1 (double).
        let p = QPoly::from_i64s(&[2, -3, 0, 1]);
        let mut roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].multiplicity(), 1);
        assert_eq!(roots[1].multiplicity(), 2);
        assert_eq!(roots[0].cmp_rational(&rat(-2, 1)), Ordering::Equal);
        assert_eq!(roots[1].cmp_rational(&rat(1, 1)), Ordering::Equal);
    }

    #[test]
    fn tangency_style_double_quartic() {
        // (x² − 2)²: ±√2 each with multiplicity 2 (the ellipse-tangency
        // resultant shape).
        let p = QPoly::from_i64s(&[4, 0, -4, 0, 1]);
        let roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().all(|r| r.multiplicity() == 2));
    }

    #[test]
    fn equality_across_polynomials() {
        // √2 as a root of x² − 2 and of x⁴ − 4 must compare Equal.
        let mut a = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        let mut b = isolate_roots(&QPoly::from_i64s(&[-4, 0, 0, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(a.cmp_root(&mut b), Ordering::Equal);
        // And √2 ≠ √3, with the right order.
        let mut c = isolate_roots(&QPoly::from_i64s(&[-3, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(a.cmp_root(&mut c), Ordering::Less);
    }

    #[test]
    fn exact_rational_roots() {
        // (2x − 1)(x² − 5): the rational root 1/2 among irrationals.
        let p = QPoly::new(vec![rat(5, 1), rat(-10, 1), rat(-1, 1), rat(2, 1)]);
        let mut roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 3);
        assert!(
            roots
                .iter_mut()
                .any(|r| r.cmp_rational(&rat(1, 2)) == Ordering::Equal)
        );
        let exact: Vec<_> = roots.iter().filter_map(|r| r.value()).collect();
        // Exactly one root is rational and it must be materialized as 1/2.
        assert_eq!(exact, vec![&rat(1, 2)]);
    }

    #[test]
    fn rational_roots_collapse_to_exact_values() {
        // REGRESSION (PLAN.md §9 / M-A¾): `16λ³ + 35λ² + 23λ + 4` has the
        // rational root λ = −1, which bisection alone reported as the interval
        // (−153/128, −255/256) with `is_exact() == false`. The pencil
        // classifier reads `value()` to find rational pencil parameters, so a
        // missed collapse silently costs it every rational member.
        let p = QPoly::from_i64s(&[4, 23, 35, 16]);
        let roots = isolate_roots(&p).unwrap();
        let exact: Vec<&Rational> = roots.iter().filter_map(|r| r.value()).collect();
        assert!(
            exact.contains(&&rat(-1, 1)),
            "λ = −1 must come back exact, got {:?}",
            roots
                .iter()
                .map(|r| (r.is_exact(), r.lo().clone(), r.hi().clone()))
                .collect::<Vec<_>>()
        );
        // The cofactor 16λ² + 19λ + 4 has discriminant 105, so the other two
        // roots are irrational and must stay intervals.
        assert_eq!(exact.len(), 1);
        assert_eq!(roots.len(), 3);

        // Denominators, not just integers: (3x − 2)(5x + 7)(x² − 2).
        let p = QPoly::from_i64s(&[-2, 0, 1])
            .mul_poly(&QPoly::from_i64s(&[-2, 3]))
            .mul_poly(&QPoly::from_i64s(&[7, 5]));
        let roots = isolate_roots(&p).unwrap();
        let exact: Vec<Rational> = roots.iter().filter_map(|r| r.value().cloned()).collect();
        assert_eq!(exact.len(), 2, "2/3 and −7/5 must both collapse");
        assert!(exact.contains(&rat(2, 3)) && exact.contains(&rat(-7, 5)));
        // …and the irrational roots must NOT be claimed exact.
        assert_eq!(roots.iter().filter(|r| !r.is_exact()).count(), 2);
    }

    #[test]
    fn simplest_rational_is_the_least_denominator_one() {
        let s = |a: (i64, i64), b: (i64, i64)| {
            simplest_rational_in(&rat(a.0, a.1), &rat(b.0, b.1)).unwrap()
        };
        assert_eq!(s((3, 10), (6, 10)), rat(1, 2));
        assert_eq!(s((-153, 128), (-255, 256)), rat(-1, 1));
        assert_eq!(s((1, 4), (1, 3)), rat(2, 7));
        assert_eq!(s((-1, 3), (1, 7)), rat(0, 1));
        assert_eq!(s((7, 10), (3, 4)), rat(5, 7));
        // Endpoints are excluded: the answer is strictly inside.
        let v = s((1, 2), (2, 3));
        assert!(v > rat(1, 2) && v < rat(2, 3));
        assert_eq!(v, rat(3, 5));
        // Empty and degenerate intervals have no answer.
        assert!(simplest_rational_in(&rat(1, 2), &rat(1, 2)).is_none());
        assert!(simplest_rational_in(&rat(1, 1), &rat(0, 1)).is_none());
    }

    #[test]
    fn collapse_never_fabricates_a_root() {
        // An irrational root must survive `collapse_if_rational` as an
        // interval — the routine may only ever MISS, never invent.
        let mut r = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert!(!r.is_exact());
        assert!(!r.collapse_if_rational());
        assert!(!r.is_exact());
        assert!(r.to_interval().contains(std::f64::consts::SQRT_2));
    }

    #[test]
    fn polynomial_division_gcd_and_square_root() {
        let x2m2 = QPoly::from_i64s(&[-2, 0, 1]);
        let x2m3 = QPoly::from_i64s(&[-3, 0, 1]);
        let p = x2m2.mul_poly(&x2m3);
        let (q, r) = p.div_rem(&x2m2).unwrap();
        assert!(r.is_zero());
        assert_eq!(q, x2m3);
        assert!(x2m2.divides(&p));
        assert!(!x2m2.divides(&QPoly::from_i64s(&[1, 1])));
        assert!(p.div_rem(&QPoly::zero_poly()).is_none());
        assert_eq!(x2m2.gcd(&p), x2m2);
        assert_eq!(QPoly::from_i64s(&[-4, 0, 2]).monic(), x2m2);

        // Perfect squares: (x²−2)² is one, (x²−2)(x²−3) is not.
        assert_eq!(x2m2.mul_poly(&x2m2).sqrt_exact(), Some(x2m2.clone()));
        assert_eq!(p.sqrt_exact(), None);
        assert_eq!(
            QPoly::from_i64s(&[9]).sqrt_exact(),
            Some(QPoly::from_i64s(&[3]))
        );
        assert_eq!(QPoly::from_i64s(&[2]).sqrt_exact(), None);
        assert_eq!(QPoly::zero_poly().sqrt_exact(), Some(QPoly::zero_poly()));
        // 4(x−1)² = (2x−2)²: the constant factor must be a rational square too.
        let sq = QPoly::from_i64s(&[-1, 1]);
        assert_eq!(
            sq.mul_poly(&sq).scale(&rat(4, 1)).sqrt_exact(),
            Some(QPoly::from_i64s(&[-2, 2]))
        );
        assert_eq!(sq.mul_poly(&sq).scale(&rat(2, 1)).sqrt_exact(), None);
    }

    #[test]
    fn multiplicity_of_at_a_root() {
        // (x−1)³(x²−2): ξ = √2 is simple in the product, and 1 is a triple
        // root of the first factor.
        let cube = QPoly::from_i64s(&[-1, 1]);
        let cube = cube.mul_poly(&cube).mul_poly(&QPoly::from_i64s(&[-1, 1]));
        let mut sqrt2 = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(sqrt2.multiplicity_of(&cube), 0);
        assert_eq!(sqrt2.multiplicity_of(&QPoly::from_i64s(&[-2, 0, 1])), 1);
        let mut one = RealRoot::exact_rational(rat(1, 1));
        assert_eq!(one.multiplicity_of(&cube), 3);
        assert_eq!(one.multiplicity_of(&QPoly::zero_poly()), u32::MAX);
    }

    #[test]
    fn constants_and_zero() {
        assert!(matches!(
            isolate_roots(&QPoly::zero_poly()),
            Err(RootError::ZeroPolynomial)
        ));
        assert_eq!(isolate_roots(&QPoly::from_i64s(&[7])).unwrap().len(), 0);
        // Linear: exactly one root, found exactly or isolated.
        let mut r = isolate_roots(&QPoly::from_i64s(&[-3, 2]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(r.cmp_rational(&rat(3, 2)), Ordering::Equal);
    }

    #[test]
    fn cmp_rational_thresholds() {
        let mut sqrt2 = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(sqrt2.cmp_rational(&rat(1, 1)), Ordering::Greater);
        assert_eq!(sqrt2.cmp_rational(&rat(2, 1)), Ordering::Less);
        assert_eq!(
            sqrt2.cmp_rational(&rat(141421356, 100000000)),
            Ordering::Greater
        );
        assert_eq!(
            sqrt2.cmp_rational(&rat(141421357, 100000000)),
            Ordering::Less
        );
    }

    #[test]
    fn radical_sign_ladders() {
        // ξ = √2.
        let mut xi = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        let x = QPoly::from_i64s(&[0, 1]);
        let one = QPoly::from_i64s(&[1]);
        // −ξ + √(ξ²) = 0.
        assert_eq!(
            sign_radical1(&mut xi, &x.scale(&rat(-1, 1)), &one, &x.mul_poly(&x)),
            Sign::Zero
        );
        // −1 + √ξ > 0 (√√2 ≈ 1.19).
        assert_eq!(
            sign_radical1(&mut xi, &QPoly::from_i64s(&[-1]), &one, &x),
            Sign::Positive
        );
        // 2 − √ξ > 0; 1.18 − √ξ < 0.
        assert_eq!(
            sign_radical1(
                &mut xi,
                &QPoly::from_i64s(&[2]),
                &one.scale(&rat(-1, 1)),
                &x
            ),
            Sign::Positive
        );
        assert_eq!(
            sign_radical1(
                &mut xi,
                &QPoly::new(vec![rat(118, 100)]),
                &one.scale(&rat(-1, 1)),
                &x
            ),
            Sign::Negative
        );
        // √ξ − √ξ = 0 (two radicals, same radicand).
        assert_eq!(
            sign_radical2(
                &mut xi,
                &QPoly::zero_poly(),
                &one,
                &x,
                &one.scale(&rat(-1, 1)),
                &x
            ),
            Sign::Zero
        );
        // 1 + √ξ − √(2ξ) > 0 (≈ 0.507).
        assert_eq!(
            sign_radical2(
                &mut xi,
                &one,
                &one,
                &x,
                &one.scale(&rat(-1, 1)),
                &x.scale(&rat(2, 1))
            ),
            Sign::Positive
        );
        // −1 + √ξ − √(2ξ) < 0.
        assert_eq!(
            sign_radical2(
                &mut xi,
                &one.scale(&rat(-1, 1)),
                &one,
                &x,
                &one.scale(&rat(-1, 1)),
                &x.scale(&rat(2, 1))
            ),
            Sign::Negative
        );
    }

    #[test]
    fn exact_rational_realroot() {
        let mut r = RealRoot::exact_rational(rat(3, 2));
        assert_eq!(r.value(), Some(&rat(3, 2)));
        assert_eq!(r.cmp_rational(&rat(3, 2)), Ordering::Equal);
        let mut s = isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(r.cmp_root(&mut s), Ordering::Greater); // 1.5 > √2
    }

    #[test]
    fn close_roots_separate() {
        // (x² − 2)(100x − 141)(100x − 142): rationals hugging √2.
        let sqrt2 = QPoly::from_i64s(&[-2, 0, 1]);
        let p = sqrt2
            .mul_poly(&QPoly::from_i64s(&[-141, 100]))
            .mul_poly(&QPoly::from_i64s(&[-142, 100]));
        let mut roots = isolate_roots(&p).unwrap();
        assert_eq!(roots.len(), 4);
        for i in 0..roots.len() - 1 {
            let (a, b) = roots.split_at_mut(i + 1);
            assert_eq!(a.last_mut().unwrap().cmp_root(&mut b[0]), Ordering::Less);
        }
    }
}
