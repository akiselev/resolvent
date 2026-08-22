use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlgebraBudget {
    pub max_expression_nodes: usize,
    pub max_polynomial_degree: usize,
    pub max_coefficient_bits: u64,
    pub max_matrix_dimension: usize,
    pub max_root_bisections: usize,
    pub max_lazy_nodes: usize,
}

impl Default for AlgebraBudget {
    fn default() -> Self {
        Self {
            max_expression_nodes: 100_000,
            max_polynomial_degree: 64,
            max_coefficient_bits: 16_384,
            max_matrix_dimension: 64,
            max_root_bisections: 4_096,
            max_lazy_nodes: 1_000_000,
        }
    }
}

/// Deterministic counter shared by the internal stages of one budgeted
/// operation. Charge before doing the corresponding exact arithmetic so an
/// exhausted budget cannot merely report after expensive work has happened.
#[derive(Debug)]
pub(crate) struct AlgebraWork {
    used: usize,
    limit: usize,
    operation: &'static str,
}

impl AlgebraWork {
    pub(crate) fn new(budget: AlgebraBudget, operation: &'static str) -> Self {
        Self {
            used: 0,
            limit: budget.max_expression_nodes,
            operation,
        }
    }

    pub(crate) fn spend(&mut self, amount: usize) -> Result<(), AlgebraError> {
        self.used = self
            .used
            .checked_add(amount)
            .ok_or(AlgebraError::BudgetExceeded {
                operation: self.operation,
                limit: self.limit,
            })?;
        if self.used > self.limit {
            return Err(AlgebraError::BudgetExceeded {
                operation: self.operation,
                limit: self.limit,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AlgebraError {
    #[error("algebra budget exceeded while {operation}: limit {limit}")]
    BudgetExceeded {
        operation: &'static str,
        limit: usize,
    },
    #[error("unsupported symbolic derivative for function `{0}`")]
    UnsupportedFunction(String),
    #[error("exact evaluation requires a value for symbol `{0}`")]
    MissingSymbol(String),
    #[error("exact evaluation is indeterminate for function `{0}`")]
    IndeterminateFunction(String),
    #[error("division by exact zero")]
    DivisionByZero,
    #[error("division by the zero polynomial")]
    DivisionByZeroPolynomial,
    #[error("resultant matrix dimension {actual} exceeds limit {limit}")]
    ResultantDimension { actual: usize, limit: usize },
    #[error("matrix dimension {actual} exceeds limit {limit}")]
    MatrixDimension { actual: usize, limit: usize },
    #[error("polynomial degree {actual} exceeds limit {limit}")]
    PolynomialDegree { actual: usize, limit: usize },
    #[error("coefficient bit size {actual} exceeds limit {limit}")]
    CoefficientBits { actual: u64, limit: u64 },
    #[error("root isolation requires a nonzero polynomial")]
    ZeroPolynomialRoots,
    #[error("failed to serialize algebra receipt payload: {0}")]
    Serialization(String),
    #[error("invalid {operation} shape: {details}")]
    Shape {
        operation: &'static str,
        details: String,
    },
    #[error("invalid interval bounds")]
    InvalidInterval,
    #[error("root refinement width must be strictly positive")]
    NonPositiveRefinementWidth,
    #[error("negative square-root radicand")]
    NegativeRadicand,
}
