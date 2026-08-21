use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlgebraBudget {
    pub max_expression_nodes: usize,
    pub max_matrix_dimension: usize,
    pub max_root_bisections: usize,
}

impl Default for AlgebraBudget {
    fn default() -> Self {
        Self {
            max_expression_nodes: 100_000,
            max_matrix_dimension: 64,
            max_root_bisections: 4_096,
        }
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
    #[error("division by the zero polynomial")]
    DivisionByZeroPolynomial,
    #[error("resultant matrix dimension {actual} exceeds limit {limit}")]
    ResultantDimension { actual: usize, limit: usize },
    #[error("root isolation requires a nonzero polynomial")]
    ZeroPolynomialRoots,
}
