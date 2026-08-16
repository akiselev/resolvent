use crate::evidence::{EvidenceItem, EvidenceProfile, Obligation};
use crate::id::{Digest, ObligationId};
use crate::model::Scope;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    LeanDeclaration,
    ScientificSpec,
    Expression,
    System,
    Form,
    DiscreteProgram,
    OperatorProgram,
    Executable,
    Dataset,
    Observable,
    ValidationBundle,
    Certificate,
    External(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub kind: ArtifactKind,
    pub digest: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

impl ArtifactRef {
    pub fn of<T: Serialize>(kind: ArtifactKind, value: &T) -> Result<Self, RefinementError> {
        let bytes = serde_json::to_vec(value)?;
        Ok(Self {
            kind,
            digest: Digest::blake3(&bytes),
            label: None,
            locator: None,
        })
    }

    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// What mathematical/scientific claim relates source and target. These are not collapsed
/// to "lowering": equality, consequence, discretization and finite-precision implementation
/// carry fundamentally different proof obligations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefinementRelation {
    DefinitionallyEqual,
    MathematicallyEquivalent,
    LogicalConsequence,
    Specialization,
    StructuralReduction,
    StrongToWeakForm,
    Discretization { scheme: String, declared_order: Option<u8> },
    ConsistentApproximation,
    ConvergentApproximation,
    BoundedApproximation { bound: String },
    AlgebraicImplementation,
    FloatingPointImplementation { arithmetic: String },
    CompiledImplementation { target: String },
    MeasurementModel,
    Custom { name: String },
}

/// Scope is not inferred. In particular, a restricted-orbit or parameter-family result
/// cannot become a global result by omission of metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScopeTransition {
    Preserved,
    Narrowed { reason: String },
    /// Broadening is legal only when the named obligation exists in the same record.
    Broadened { obligation: ObligationId, reason: String },
    /// Non-orderable changes (different convention/domain parameterization) likewise need
    /// an explicit obligation connecting the two meanings.
    Changed { obligation: ObligationId, reason: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    #[serde(default)]
    pub parents: Vec<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementRecord {
    pub schema_version: String,
    pub source: ArtifactRef,
    pub target: ArtifactRef,
    pub relation: RefinementRelation,
    pub source_scope: Scope,
    pub target_scope: Scope,
    pub scope_transition: ScopeTransition,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub obligations: Vec<Obligation>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
    #[serde(default)]
    pub provenance: RefinementProvenance,
}

impl RefinementRecord {
    pub fn new(source: ArtifactRef, target: ArtifactRef, relation: RefinementRelation) -> Self {
        Self {
            schema_version: "resolvent-refinement/0.1".into(),
            source,
            target,
            relation,
            source_scope: Scope::default(),
            target_scope: Scope::default(),
            scope_transition: ScopeTransition::Preserved,
            assumptions: Vec::new(),
            obligations: Vec::new(),
            evidence: Vec::new(),
            provenance: RefinementProvenance::default(),
        }
    }

    pub fn validate(&self) -> Result<(), RefinementError> {
        let required = match self.scope_transition {
            ScopeTransition::Broadened { obligation, .. }
            | ScopeTransition::Changed { obligation, .. } => Some(obligation),
            ScopeTransition::Preserved | ScopeTransition::Narrowed { .. } => None,
        };
        if let Some(id) = required
            && !self.obligations.iter().any(|o| o.id == id)
        {
            return Err(RefinementError::MissingScopeObligation(id));
        }
        Ok(())
    }

    pub fn evidence_profile(&self) -> EvidenceProfile {
        EvidenceProfile::from_items(&self.evidence)
    }

    /// "Promotion-ready" is deliberately stricter than structurally valid: every
    /// obligation must be closed and at least one explicit evidence item must warrant the
    /// claimed relation. Refuted obligations can never be promoted.
    pub fn is_promotion_ready(&self) -> bool {
        self.validate().is_ok()
            && !self.evidence.is_empty()
            && self.obligations.iter().all(Obligation::is_closed)
    }

    pub fn digest(&self) -> Result<Digest, RefinementError> {
        Ok(Digest::blake3(&serde_json::to_vec(self)?))
    }
}

#[derive(Debug, Error)]
pub enum RefinementError {
    #[error("scope transition references missing obligation {0}")]
    MissingScopeObligation(ObligationId),
    #[error("cannot serialize refinement artifact: {0}")]
    Serialization(#[from] serde_json::Error),
}
