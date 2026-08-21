use crate::{AlgebraBudget, AlgebraError, Rational, Sign};
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Polynomial {
    coefficients: Vec<Rational>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootInterval {
    pub lower: Rational,
    pub upper: Rational,
}

impl Polynomial {
    pub fn new(mut coefficients: Vec<Rational>) -> Self {
        while coefficients.last().is_some_and(Zero::is_zero) {
            coefficients.pop();
        }
        Self { coefficients }
    }
    pub fn zero() -> Self {
        Self::new(Vec::new())
    }
    pub fn one() -> Self {
        Self::new(vec![Rational::one()])
    }
    pub fn variable() -> Self {
        Self::new(vec![Rational::zero(), Rational::one()])
    }
    pub fn coefficients(&self) -> &[Rational] {
        &self.coefficients
    }
    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }
    pub fn degree(&self) -> Option<usize> {
        self.coefficients.len().checked_sub(1)
    }
    pub fn leading_coefficient(&self) -> Option<&Rational> {
        self.coefficients.last()
    }
    pub fn evaluate(&self, value: &Rational) -> Rational {
        self.coefficients
            .iter()
            .rev()
            .fold(Rational::zero(), |acc, coefficient| {
                acc * value + coefficient
            })
    }
    pub fn add(&self, rhs: &Self) -> Self {
        let mut out = vec![Rational::zero(); self.coefficients.len().max(rhs.coefficients.len())];
        for (index, coefficient) in self.coefficients.iter().enumerate() {
            out[index] += coefficient;
        }
        for (index, coefficient) in rhs.coefficients.iter().enumerate() {
            out[index] += coefficient;
        }
        Self::new(out)
    }
    pub fn negated(&self) -> Self {
        Self::new(self.coefficients.iter().map(std::ops::Neg::neg).collect())
    }
    pub fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.negated())
    }
    pub fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }
        let mut out = vec![Rational::zero(); self.coefficients.len() + rhs.coefficients.len() - 1];
        for (left_degree, left) in self.coefficients.iter().enumerate() {
            for (right_degree, right) in rhs.coefficients.iter().enumerate() {
                out[left_degree + right_degree] += left * right;
            }
        }
        Self::new(out)
    }
    pub fn derivative(&self) -> Self {
        Self::new(
            self.coefficients
                .iter()
                .enumerate()
                .skip(1)
                .map(|(degree, coefficient)| coefficient * Rational::from_integer(degree.into()))
                .collect(),
        )
    }
    pub fn div_rem(&self, divisor: &Self) -> Result<(Self, Self), AlgebraError> {
        if divisor.is_zero() {
            return Err(AlgebraError::DivisionByZeroPolynomial);
        }
        let mut remainder = self.clone();
        let divisor_degree = divisor.degree().expect("nonzero divisor");
        let divisor_lead = divisor
            .leading_coefficient()
            .expect("nonzero divisor")
            .clone();
        let quotient_size = self
            .degree()
            .and_then(|degree| degree.checked_sub(divisor_degree))
            .map_or(0, |degree| degree + 1);
        let mut quotient = vec![Rational::zero(); quotient_size];
        while remainder
            .degree()
            .is_some_and(|degree| degree >= divisor_degree)
        {
            let degree = remainder.degree().expect("checked") - divisor_degree;
            let factor =
                remainder.leading_coefficient().expect("nonzero remainder") / &divisor_lead;
            quotient[degree] = factor.clone();
            let mut term = vec![Rational::zero(); degree];
            term.extend(
                divisor
                    .coefficients
                    .iter()
                    .map(|coefficient| coefficient * &factor),
            );
            remainder = remainder.sub(&Self::new(term));
        }
        Ok((Self::new(quotient), remainder))
    }
    pub fn gcd(&self, rhs: &Self) -> Result<Self, AlgebraError> {
        let (mut left, mut right) = (self.clone(), rhs.clone());
        while !right.is_zero() {
            let (_, remainder) = left.div_rem(&right)?;
            left = right;
            right = remainder;
        }
        Ok(left.monic())
    }
    pub fn monic(&self) -> Self {
        let Some(lead) = self.leading_coefficient() else {
            return Self::zero();
        };
        Self::new(
            self.coefficients
                .iter()
                .map(|coefficient| coefficient / lead)
                .collect(),
        )
    }
    pub fn resultant(&self, rhs: &Self, budget: AlgebraBudget) -> Result<Rational, AlgebraError> {
        if self.is_zero() || rhs.is_zero() {
            return Ok(Rational::zero());
        }
        let (m, n) = (
            self.degree().expect("nonzero"),
            rhs.degree().expect("nonzero"),
        );
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
        let left = self.coefficients.iter().rev().cloned().collect::<Vec<_>>();
        let right = rhs.coefficients.iter().rev().cloned().collect::<Vec<_>>();
        for row in 0..n {
            matrix[row][row..row + left.len()].clone_from_slice(&left);
        }
        for row in 0..m {
            matrix[n + row][row..row + right.len()].clone_from_slice(&right);
        }
        Ok(determinant(matrix))
    }
    pub fn isolate_real_roots(
        &self,
        budget: AlgebraBudget,
    ) -> Result<Vec<RootInterval>, AlgebraError> {
        if self.is_zero() {
            return Err(AlgebraError::ZeroPolynomialRoots);
        }
        if self.degree().is_none_or(|degree| degree == 0) {
            return Ok(Vec::new());
        }
        let square_free = self.div_rem(&self.gcd(&self.derivative())?)?.0;
        let sturm = sturm_sequence(&square_free)?;
        let bound = root_bound(&square_free);
        let mut out = Vec::new();
        let mut work = vec![(-bound.clone(), bound)];
        let mut bisections = 0;
        while let Some((lower, upper)) = work.pop() {
            let count = root_count(&sturm, &lower, &upper);
            if count == 0 {
                continue;
            }
            if count == 1 {
                out.push(RootInterval { lower, upper });
                continue;
            }
            bisections += 1;
            if bisections > budget.max_root_bisections {
                return Err(AlgebraError::BudgetExceeded {
                    operation: "isolating real roots",
                    limit: budget.max_root_bisections,
                });
            }
            let midpoint = (&lower + &upper) / Rational::from_integer(2.into());
            work.push((midpoint.clone(), upper));
            work.push((lower, midpoint));
        }
        out.sort_by(|left, right| left.lower.cmp(&right.lower));
        Ok(out)
    }
}

fn determinant(mut matrix: Vec<Vec<Rational>>) -> Rational {
    let mut determinant = Rational::one();
    for column in 0..matrix.len() {
        let Some(pivot) = (column..matrix.len()).find(|row| !matrix[*row][column].is_zero()) else {
            return Rational::zero();
        };
        if pivot != column {
            matrix.swap(pivot, column);
            determinant = -determinant;
        }
        let pivot_value = matrix[column][column].clone();
        determinant *= &pivot_value;
        let pivot_row = matrix[column].clone();
        for row in matrix.iter_mut().skip(column + 1) {
            let factor = row[column].clone() / &pivot_value;
            for (entry, pivot_entry) in row.iter_mut().zip(&pivot_row).skip(column) {
                let reduction = &factor * pivot_entry;
                *entry -= reduction;
            }
        }
    }
    determinant
}

fn sturm_sequence(polynomial: &Polynomial) -> Result<Vec<Polynomial>, AlgebraError> {
    let mut sequence = vec![polynomial.clone(), polynomial.derivative()];
    loop {
        let length = sequence.len();
        if sequence[length - 1].is_zero() {
            break;
        }
        let (_, remainder) = sequence[length - 2].div_rem(&sequence[length - 1])?;
        if remainder.is_zero() {
            break;
        }
        sequence.push(remainder.negated());
    }
    Ok(sequence)
}

fn variations(sequence: &[Polynomial], point: &Rational) -> usize {
    let mut previous = None;
    let mut changes = 0;
    for polynomial in sequence {
        let sign = Sign::of(&polynomial.evaluate(point));
        if sign == Sign::Zero {
            continue;
        }
        if previous.is_some_and(|previous| previous != sign) {
            changes += 1;
        }
        previous = Some(sign);
    }
    changes
}

fn root_count(sequence: &[Polynomial], lower: &Rational, upper: &Rational) -> usize {
    variations(sequence, lower).saturating_sub(variations(sequence, upper))
}

fn root_bound(polynomial: &Polynomial) -> Rational {
    let lead = polynomial
        .leading_coefficient()
        .expect("nonconstant polynomial")
        .abs();
    polynomial
        .coefficients
        .iter()
        .take(polynomial.coefficients.len() - 1)
        .map(|coefficient| coefficient.abs() / &lead)
        .max()
        .unwrap_or_else(Rational::zero)
        + Rational::one()
}
