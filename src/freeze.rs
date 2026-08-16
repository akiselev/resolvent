use crate::author::{AuthorError, ElaboratedModel, elaborate};
use crate::id::Digest;
use crate::refinement::ArtifactKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLock {
    pub schema_version: String,
    pub compiler_schema: String,
    pub source_digest: String,
    pub semantic_digest: Digest,
    pub model_name: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub open_obligations: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl SemanticLock {
    pub fn from_elaborated(model: &ElaboratedModel) -> Result<Self, FreezeError> {
        let artifact = model
            .context
            .rooted_artifact_ref(ArtifactKind::ScientificSpec, &model.spec)?;
        Ok(Self {
            schema_version: "resolvent-lock/0.1".into(),
            compiler_schema: crate::SCIENTIFIC_SCHEMA_VERSION.into(),
            source_digest: model.source_digest.clone(),
            semantic_digest: artifact.digest,
            model_name: model.spec.name.clone(),
            assumptions: model
                .spec
                .assumptions
                .iter()
                .map(|a| a.statement.clone())
                .collect(),
            open_obligations: Vec::new(),
            metadata: BTreeMap::new(),
        })
    }
    pub fn from_source(source: &str) -> Result<Self, FreezeError> {
        let model = elaborate(source)?;
        Self::from_elaborated(&model)
    }
    pub fn verify_source(&self, source: &str) -> Result<(), FreezeError> {
        let actual = Self::from_source(source)?;
        if self.source_digest != actual.source_digest {
            return Err(FreezeError::SourceDrift);
        }
        if self.semantic_digest != actual.semantic_digest {
            return Err(FreezeError::SemanticDrift);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum FreezeError {
    #[error(transparent)]
    Author(#[from] AuthorError),
    #[error(transparent)]
    Refinement(#[from] crate::refinement::RefinementError),
    #[error("source digest differs from lock")]
    SourceDrift,
    #[error("semantic digest differs from lock")]
    SemanticDrift,
}
