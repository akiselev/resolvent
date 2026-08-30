use crate::{AlgebraBudget, AlgebraError, ExactRing, Rational, RingOps, Sign};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Rational(Rational),
    Symbol(String),
    Add(Vec<Self>),
    Mul(Vec<Self>),
    Pow { base: Box<Self>, exponent: i32 },
    Function { name: String, args: Vec<Self> },
}

impl Expr {
    pub fn integer(value: i64) -> Self {
        Self::Rational(Rational::from_i64(value))
    }
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
    }
    pub fn add(terms: impl IntoIterator<Item = Self>) -> Self {
        Self::Add(terms.into_iter().collect())
    }
    pub fn mul(factors: impl IntoIterator<Item = Self>) -> Self {
        Self::Mul(factors.into_iter().collect())
    }
    pub fn pow(self, exponent: i32) -> Self {
        Self::Pow {
            base: Box::new(self),
            exponent,
        }
    }
    pub fn function(name: impl Into<String>, args: impl IntoIterator<Item = Self>) -> Self {
        Self::Function {
            name: name.into(),
            args: args.into_iter().collect(),
        }
    }
    pub fn canonicalize(&self, budget: AlgebraBudget) -> Result<Self, AlgebraError> {
        let mut visited = 0;
        canonicalize(self, budget, &mut visited)
    }
    pub fn differentiate(
        &self,
        variable: &str,
        budget: AlgebraBudget,
    ) -> Result<Self, AlgebraError> {
        let mut visited = 0;
        let derivative = differentiate(self, variable, budget.max_expression_nodes, &mut visited)?;
        derivative.canonicalize(budget)
    }
    pub fn evaluate(
        &self,
        environment: &BTreeMap<String, Rational>,
    ) -> Result<Rational, AlgebraError> {
        match self {
            Self::Rational(value) => Ok(value.clone()),
            Self::Symbol(name) => environment
                .get(name)
                .cloned()
                .ok_or_else(|| AlgebraError::MissingSymbol(name.clone())),
            Self::Add(terms) => terms.iter().try_fold(Rational::zero(), |sum, term| {
                Ok(sum.add(&term.evaluate(environment)?))
            }),
            Self::Mul(factors) => factors.iter().try_fold(Rational::one(), |product, factor| {
                Ok(product.mul(&factor.evaluate(environment)?))
            }),
            Self::Pow { base, exponent } => {
                let value = base.evaluate(environment)?;
                value
                    .checked_pow(*exponent)
                    .ok_or(AlgebraError::DivisionByZero)
            }
            Self::Function { name, .. } => Err(AlgebraError::IndeterminateFunction(name.clone())),
        }
    }
    pub fn exact_sign(
        &self,
        environment: &BTreeMap<String, Rational>,
    ) -> Result<Sign, AlgebraError> {
        self.evaluate(environment).map(|value| value.sign())
    }
}

fn spend(visited: &mut usize, limit: usize, operation: &'static str) -> Result<(), AlgebraError> {
    *visited += 1;
    if *visited > limit {
        Err(AlgebraError::BudgetExceeded { operation, limit })
    } else {
        Ok(())
    }
}

fn canonicalize(
    expr: &Expr,
    budget: AlgebraBudget,
    visited: &mut usize,
) -> Result<Expr, AlgebraError> {
    let limit = budget.max_expression_nodes;
    spend(visited, limit, "canonicalizing expression")?;
    Ok(match expr {
        Expr::Rational(_) | Expr::Symbol(_) => expr.clone(),
        Expr::Add(terms) => {
            let mut flat = Vec::new();
            let mut constant = Rational::zero();
            for term in terms {
                match canonicalize(term, budget, visited)? {
                    Expr::Add(nested) => {
                        for value in nested {
                            if let Expr::Rational(value) = value {
                                constant = constant.add(&value);
                            } else {
                                flat.push(value);
                            }
                        }
                    }
                    Expr::Rational(value) => constant = constant.add(&value),
                    value => flat.push(value),
                }
            }
            if !constant.is_zero() {
                flat.push(Expr::Rational(constant));
            }
            flat.sort_by_key(canonical_key);
            match flat.len() {
                0 => Expr::integer(0),
                1 => flat.pop().expect("length checked"),
                _ => Expr::Add(flat),
            }
        }
        Expr::Mul(factors) => {
            let mut flat = Vec::new();
            let mut constant = Rational::one();
            for factor in factors {
                match canonicalize(factor, budget, visited)? {
                    Expr::Mul(nested) => {
                        for value in nested {
                            if let Expr::Rational(value) = value {
                                constant = constant.mul(&value);
                            } else {
                                flat.push(value);
                            }
                        }
                    }
                    Expr::Rational(value) => constant = constant.mul(&value),
                    value => flat.push(value),
                }
            }
            if constant.is_zero() {
                Expr::integer(0)
            } else {
                if !constant.is_one() {
                    flat.push(Expr::Rational(constant));
                }
                flat.sort_by_key(canonical_key);
                match flat.len() {
                    0 => Expr::integer(1),
                    1 => flat.pop().expect("length checked"),
                    _ => Expr::Mul(flat),
                }
            }
        }
        Expr::Pow { base, exponent } => {
            let base = canonicalize(base, budget, visited)?;
            match exponent {
                0 => Expr::integer(1),
                1 => base,
                _ => fold_literal_power(base, *exponent, budget)?,
            }
        }
        Expr::Function { name, args } => Expr::Function {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| canonicalize(arg, budget, visited))
                .collect::<Result<_, _>>()?,
        },
    })
}

/// Folds `Pow` of an exact literal base with a nonzero, non-unit integer
/// exponent (`exponent == 0` and `exponent == 1` are handled by the caller)
/// down to an exact literal, for example `5^2` to `25` and `(1/2)^-1` to `2`.
///
/// Any other base — a symbol, a compound expression, or a function call — is
/// left as a structural `Pow` node: only *exact literal* bases fold.
///
/// `0^0` folds to the literal `1`, matching the existing convention in
/// [`Expr::evaluate`] (`Rational::checked_pow` already treats a zero exponent
/// as the empty product). A zero base raised to a negative exponent is a
/// genuine division by exact zero and refuses with
/// [`AlgebraError::DivisionByZero`], the same error `evaluate` reports for
/// the identical construct.
///
/// The resulting magnitude is charged against `budget.max_coefficient_bits`:
/// a conservative pre-check (exact exponentiation can only shrink a
/// rational's bit size via reduction, never grow it beyond
/// `|exponent| * base.bit_size()`) rejects runaway exponents before doing the
/// expensive exact exponentiation, and a post-check catches any remaining
/// case.
fn fold_literal_power(
    base: Expr,
    exponent: i32,
    budget: AlgebraBudget,
) -> Result<Expr, AlgebraError> {
    let Expr::Rational(value) = &base else {
        return Ok(Expr::Pow {
            base: Box::new(base),
            exponent,
        });
    };
    let estimated_bits = value
        .bit_size()
        .saturating_mul(u64::from(exponent.unsigned_abs()));
    if estimated_bits > budget.max_coefficient_bits {
        return Err(AlgebraError::CoefficientBits {
            actual: estimated_bits,
            limit: budget.max_coefficient_bits,
        });
    }
    let folded = value
        .checked_pow(exponent)
        .ok_or(AlgebraError::DivisionByZero)?;
    let actual_bits = folded.bit_size();
    if actual_bits > budget.max_coefficient_bits {
        return Err(AlgebraError::CoefficientBits {
            actual: actual_bits,
            limit: budget.max_coefficient_bits,
        });
    }
    Ok(Expr::Rational(folded))
}

fn canonical_key(expr: &Expr) -> String {
    serde_json::to_string(expr).expect("expression serialization is infallible")
}

fn differentiate(
    expr: &Expr,
    variable: &str,
    limit: usize,
    visited: &mut usize,
) -> Result<Expr, AlgebraError> {
    spend(visited, limit, "differentiating expression")?;
    Ok(match expr {
        Expr::Rational(_) => Expr::integer(0),
        Expr::Symbol(name) => Expr::integer(i64::from(name == variable)),
        Expr::Add(terms) => Expr::add(
            terms
                .iter()
                .map(|term| differentiate(term, variable, limit, visited))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Expr::Mul(factors) => {
            let mut terms = Vec::with_capacity(factors.len());
            for index in 0..factors.len() {
                let mut product = factors.clone();
                product[index] = differentiate(&factors[index], variable, limit, visited)?;
                terms.push(Expr::mul(product));
            }
            Expr::add(terms)
        }
        Expr::Pow { base, exponent } => Expr::mul([
            Expr::integer(i64::from(*exponent)),
            base.as_ref().clone().pow(exponent - 1),
            differentiate(base, variable, limit, visited)?,
        ]),
        Expr::Function { name, args } if args.len() == 1 => {
            let arg = args[0].clone();
            let derivative = differentiate(&arg, variable, limit, visited)?;
            let outer = match name.as_str() {
                "sin" => Expr::function("cos", [arg]),
                "cos" => Expr::mul([Expr::integer(-1), Expr::function("sin", [arg])]),
                "exp" => Expr::function("exp", [arg]),
                "log" | "ln" => arg.pow(-1),
                "sqrt" => Expr::mul([
                    Expr::Rational(Rational::from_ratio(1, 2)),
                    Expr::function("sqrt", [arg]).pow(-1),
                ]),
                _ => return Err(AlgebraError::UnsupportedFunction(name.clone())),
            };
            Expr::mul([outer, derivative])
        }
        Expr::Function { name, .. } => {
            return Err(AlgebraError::UnsupportedFunction(name.clone()));
        }
    })
}
