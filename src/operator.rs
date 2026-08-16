use crate::id::DiscreteProgramId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorBlockKind {
    Residual,
    Mass,
    Damping,
    Stiffness,
    Constraint,
    Event,
    Objective,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorBlock {
    pub name: String,
    pub kind: OperatorBlockKind,
    pub program: DiscreteProgramId,
    #[serde(default)]
    pub row_variables: Vec<String>,
    #[serde(default)]
    pub column_variables: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivativeCapability {
    AnalyticJacobian,
    Jvp,
    Vjp,
    ParameterDerivative,
    HessianVector,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorProperty {
    Symmetric,
    Hermitian,
    PositiveDefinite,
    PositiveSemidefinite,
    Conservative { quantity: String },
    Nullspace { description: String },
    GaugeFreedom { description: String },
    SaddlePoint,
    UnitsConsistent,
    Custom { name: String, statement: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparsityContract {
    pub rows: usize,
    pub cols: usize,
    #[serde(default)]
    pub block_pattern: Vec<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Common discrete/semi-discrete mathematical program consumed by numerical algorithms.
/// Solver policy does not live here; neither does machine instruction scheduling.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorProgram {
    pub name: String,
    #[serde(default)]
    pub blocks: Vec<OperatorBlock>,
    #[serde(default)]
    pub derivatives: Vec<DerivativeCapability>,
    #[serde(default)]
    pub properties: Vec<OperatorProperty>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparsity: Option<SparsityContract>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}
