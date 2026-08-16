use crate::refinement::{ArtifactKind, ArtifactRef, RefinementError, RefinementRecord, RefinementRelation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dialect {
    Expr,
    System,
    Form,
    Discrete,
    Operator,
    Executable,
}

/// MLIR-style legality boundary. A pass may preserve higher-level dialects deliberately;
/// lowering is progressive rather than an instruction to erase semantic structure early.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalityTarget {
    #[serde(default)]
    pub legal: BTreeSet<Dialect>,
}

impl LegalityTarget {
    pub fn allows(&self, dialect: Dialect) -> bool {
        self.legal.contains(&dialect)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Note,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassContract {
    pub name: String,
    pub input: Dialect,
    pub output: Dialect,
    pub relation: RefinementRelation,
    pub legality: LegalityTarget,
}

/// A lowering never returns just `T`. The receipt travels with the output from the moment
/// it is created, preventing later code from having to reconstruct why two artifacts are
/// supposed to represent the same scientific object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lowering<T> {
    pub output: T,
    pub refinement: RefinementRecord,
    #[serde(default)]
    pub diagnostics: Vec<CompilerDiagnostic>,
}

impl<T> Lowering<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Lowering<U> {
        Lowering {
            output: f(self.output),
            refinement: self.refinement,
            diagnostics: self.diagnostics,
        }
    }
}

/// General pass seam. Concrete passes are free to use exact algebra, structural graph
/// algorithms, variational calculus or discretization machinery, but they must emit the
/// semantic relation they claim.
pub trait LoweringPass<S, T> {
    fn contract(&self) -> PassContract;
    fn lower(&self, source: &S) -> Result<Lowering<T>, CompileError>;
}

/// Utility for identity/compatibility migrations: wrap two serializable representations
/// in a declared refinement without inventing a second provenance format.
pub fn declare_refinement<S: Serialize, T: Serialize>(
    source_kind: ArtifactKind,
    source: &S,
    target_kind: ArtifactKind,
    target: &T,
    relation: RefinementRelation,
) -> Result<RefinementRecord, CompileError> {
    let source = ArtifactRef::of(source_kind, source)?;
    let target = ArtifactRef::of(target_kind, target)?;
    Ok(RefinementRecord::new(source, target, relation))
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Refinement(#[from] RefinementError),
    #[error("lowering failed: {0}")]
    Lowering(String),
}
