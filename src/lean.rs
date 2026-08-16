use crate::evidence::FormalGrade;
use crate::id::Digest;
use crate::refinement::ArtifactRef;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Reference to the elaborated, kernel-visible Lean declaration. Surface syntax (including
/// Ferris–Howard) is deliberately absent from this contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanDeclaration {
    pub module: String,
    pub name: String,
    pub statement_digest: Digest,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
}

/// Receipt connecting a kernel-visible declaration to a Resolvent deep-IR artifact.
/// A producer may emit an unchecked receipt for exploration; `KernelProved` requires a
/// named Lean theorem relating the declaration's semantics to the reified artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReificationReceipt {
    pub schema_version: String,
    pub declaration: LeanDeclaration,
    pub artifact: ArtifactRef,
    pub grade: FormalGrade,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soundness_theorem: Option<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
}

impl ReificationReceipt {
    pub fn validate(&self) -> Result<(), LeanBridgeError> {
        if self.grade == FormalGrade::KernelProved
            && self.soundness_theorem.as_deref().is_none_or(str::is_empty)
        {
            return Err(LeanBridgeError::MissingSoundnessTheorem);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeanExportManifest {
    pub schema_version: String,
    #[serde(default)]
    pub receipts: Vec<ReificationReceipt>,
    #[serde(default)]
    pub checker_axiom_whitelist: Vec<String>,
}

impl LeanExportManifest {
    pub fn validate(&self) -> Result<(), LeanBridgeError> {
        for receipt in &self.receipts {
            receipt.validate()?;
            if receipt.grade == FormalGrade::KernelProved {
                for axiom in &receipt.declaration.axioms {
                    if !self.checker_axiom_whitelist.contains(axiom) {
                        return Err(LeanBridgeError::UnexpectedAxiom(axiom.clone()));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LeanBridgeError {
    #[error("kernel-proved reification is missing a named soundness theorem")]
    MissingSoundnessTheorem,
    #[error("kernel-proved declaration depends on non-whitelisted axiom {0}")]
    UnexpectedAxiom(String),
}
