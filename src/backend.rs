use crate::Context;
use crate::discrete::DiscreteProgram;
use crate::id::{Digest, OperatorId};
use crate::operator::OperatorProgram;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Realization {
    Reference,
    Assembled,
    PartialAssembly,
    MatrixFree,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub realization: Realization,
    #[serde(default)]
    pub fuse: bool,
    #[serde(default)]
    pub vectorize: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}
impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            realization: Realization::Reference,
            fuse: false,
            vectorize: false,
            target: None,
        }
    }
}

/// Backend-neutral executable packet. It still references mathematical expressions by ID;
/// an execution backend must receive the frozen Context whose digest is recorded here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub schema: String,
    pub operator_id: OperatorId,
    pub operator: OperatorProgram,
    pub programs: Vec<DiscreteProgram>,
    pub context_digest: Digest,
    pub policy: ExecutionPolicy,
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("operator id {0} is absent from context")]
    MissingOperator(u32),
    #[error("discrete program id {0} is absent from context")]
    MissingProgram(u32),
    #[error("could not hash compiler context: {0}")]
    Hash(String),
    #[error("backend `{backend}` rejected plan: {message}")]
    Rejected { backend: String, message: String },
}

pub fn build_execution_plan(
    ctx: &Context,
    operator_id: OperatorId,
    policy: ExecutionPolicy,
) -> Result<ExecutionPlan, BackendError> {
    let operator = ctx
        .operator(operator_id)
        .cloned()
        .ok_or(BackendError::MissingOperator(operator_id.0))?;
    let mut programs = vec![];
    for block in &operator.blocks {
        programs.push(
            ctx.discrete(block.program)
                .cloned()
                .ok_or(BackendError::MissingProgram(block.program.0))?,
        )
    }
    let context_digest = ctx
        .rooted_artifact_ref(crate::ArtifactKind::OperatorProgram, &operator)
        .map_err(|e| BackendError::Hash(e.to_string()))?
        .digest;
    Ok(ExecutionPlan {
        schema: "resolvent-execution/0.1".into(),
        operator_id,
        operator,
        programs,
        context_digest,
        policy,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub assembled: bool,
    pub matrix_free: bool,
    pub jvp: bool,
    pub vjp: bool,
    pub targets: Vec<String>,
}

pub trait ExecutionBackend {
    type Executable;
    fn name(&self) -> &str;
    fn capabilities(&self) -> BackendCapabilities;
    fn compile(&self, plan: &ExecutionPlan) -> Result<Self::Executable, BackendError>;
}

/// Useful for tests, caching layers and adapters that only need a validated frozen plan.
pub struct IdentityBackend;
impl ExecutionBackend for IdentityBackend {
    type Executable = ExecutionPlan;
    fn name(&self) -> &str {
        "identity"
    }
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            assembled: true,
            matrix_free: true,
            jvp: true,
            vjp: true,
            targets: vec!["portable".into()],
        }
    }
    fn compile(&self, plan: &ExecutionPlan) -> Result<Self::Executable, BackendError> {
        Ok(plan.clone())
    }
}
