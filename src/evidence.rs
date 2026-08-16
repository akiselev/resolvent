use crate::id::{Digest, ObligationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The three evidence dimensions are intentionally orthogonal. A proof can establish a
/// theorem without showing that a physical model describes a device; measurements can fit
/// a model without proving a universal mathematical claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAxis {
    Formal,
    Numerical,
    Empirical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalGrade {
    Unchecked,
    Asserted,
    CertificateChecked,
    KernelProved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericalGrade {
    Untested,
    Replayed,
    DifferentiallyChecked,
    ConvergenceTested,
    ErrorBounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmpiricalGrade {
    NoData,
    Retrospective,
    IndependentlyReplicated,
    Prospective,
}

/// A grade on exactly one axis. This is a sum type rather than a single ordinal on purpose:
/// `KernelProved` and `Prospective` cannot be compared as if one were globally stronger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "axis", content = "grade", rename_all = "snake_case")]
pub enum EvidenceGrade {
    Formal(FormalGrade),
    Numerical(NumericalGrade),
    Empirical(EmpiricalGrade),
}

impl EvidenceGrade {
    pub const fn axis(&self) -> EvidenceAxis {
        match self {
            Self::Formal(_) => EvidenceAxis::Formal,
            Self::Numerical(_) => EvidenceAxis::Numerical,
            Self::Empirical(_) => EvidenceAxis::Empirical,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    pub digest: Digest,
    pub media_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub grade: EvidenceGrade,
    pub claim: String,
    #[serde(default)]
    pub artifacts: Vec<EvidenceArtifact>,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    Open,
    Discharged,
    Waived { reason: String },
    Refuted { reason: String },
}

/// A statement that must be settled before a refinement can claim the relation it names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obligation {
    pub id: ObligationId,
    pub claim: String,
    pub status: ObligationStatus,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
}

impl Obligation {
    pub fn open(id: ObligationId, claim: impl Into<String>) -> Self {
        Self {
            id,
            claim: claim.into(),
            status: ObligationStatus::Open,
            evidence: Vec::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(
            &self.status,
            ObligationStatus::Discharged | ObligationStatus::Waived { .. }
        )
    }
}

/// A compact summary for UI/reporting. It retains one optional grade per axis rather than
/// collapsing the three axes to an attractive but meaningless scalar score.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal: Option<FormalGrade>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numerical: Option<NumericalGrade>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empirical: Option<EmpiricalGrade>,
}

impl EvidenceProfile {
    pub fn ingest(&mut self, grade: &EvidenceGrade) {
        match grade {
            EvidenceGrade::Formal(g) => self.formal = Some(*g),
            EvidenceGrade::Numerical(g) => self.numerical = Some(*g),
            EvidenceGrade::Empirical(g) => self.empirical = Some(*g),
        }
    }

    pub fn from_items(items: &[EvidenceItem]) -> Self {
        let mut profile = Self::default();
        for item in items {
            profile.ingest(&item.grade);
        }
        profile
    }
}
