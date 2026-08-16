use crate::evidence::{EvidenceItem, EvidenceProfile};
use crate::refinement::ArtifactRef;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationKind {
    ExactIdentity,
    Conservation,
    Symmetry,
    Invariance,
    Positivity,
    Bound,
    ManufacturedSolution,
    ConvergenceOrder,
    AdjointDotProduct,
    ParameterDerivative,
    AssembledVsMatrixFree,
    ReferenceInterpreter,
    DifferentialOracle,
    MeasurementComparison,
    Metamorphic,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub id: String,
    pub kind: ValidationKind,
    pub claim: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_source: Option<String>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    NotRun,
    Passed,
    Failed,
    Inapplicable,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_id: String,
    pub status: CheckStatus,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Portable validator product. Sinbad can execute these checks; Pi Lab can ingest their
/// evidence; Lean can be the source of formal contracts. The bundle itself does not pretend
/// that passing numerical/empirical checks proves the originating theorem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationBundle {
    pub schema_version: String,
    pub subject: ArtifactRef,
    #[serde(default)]
    pub checks: Vec<ValidationCheck>,
    #[serde(default)]
    pub results: Vec<CheckResult>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl ValidationBundle {
    pub fn new(subject: ArtifactRef) -> Self {
        Self {
            schema_version: "resolvent-validation/0.1".into(),
            subject,
            checks: Vec::new(),
            results: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn result_for(&self, id: &str) -> Option<&CheckResult> {
        self.results.iter().find(|r| r.check_id == id)
    }

    pub fn evidence_profile(&self) -> EvidenceProfile {
        let mut all = Vec::new();
        for result in &self.results {
            all.extend(result.evidence.iter().cloned());
        }
        EvidenceProfile::from_items(&all)
    }

    pub fn has_failure(&self) -> bool {
        self.results.iter().any(|r| r.status == CheckStatus::Failed)
    }
}
