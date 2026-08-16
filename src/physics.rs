use crate::Context;
use crate::id::Digest;
use crate::rsl::{ElaboratedRsl, RslModel, parse_rsl};
use crate::source::SourceDiagnostic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsLock {
    pub schema: String,
    pub source_digest: Digest,
    pub semantic_digest: Digest,
    pub compiler_schema: String,
    #[serde(default)]
    pub obligations: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Error)]
pub enum PhysicsError {
    #[error("source did not elaborate: {0:?}")]
    Diagnostics(Vec<SourceDiagnostic>),
    #[error("could not serialize semantic artifact: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("physics lock does not match source: expected {expected:?}, got {actual:?}")]
    SourceDrift { expected: Digest, actual: Digest },
    #[error(
        "physics lock does not match elaborated semantics: expected {expected:?}, got {actual:?}"
    )]
    SemanticDrift { expected: Digest, actual: Digest },
}

pub fn parse_and_elaborate(
    source: &str,
) -> Result<(Context, RslModel, ElaboratedRsl), PhysicsError> {
    let model = parse_rsl(source).map_err(PhysicsError::Diagnostics)?;
    let mut ctx = Context::new();
    let elaborated = model
        .elaborate(&mut ctx)
        .map_err(PhysicsError::Diagnostics)?;
    Ok((ctx, model, elaborated))
}

pub fn freeze(source: &str) -> Result<PhysicsLock, PhysicsError> {
    let (ctx, _model, elaborated) = parse_and_elaborate(source)?;
    let source_digest = Digest::blake3(source.as_bytes());
    let semantic = ctx
        .rooted_artifact_ref(crate::ArtifactKind::ScientificSpec, &elaborated.spec)
        .map_err(|e| {
            PhysicsError::Serialize(serde_json::Error::io(std::io::Error::other(e.to_string())))
        })?;
    Ok(PhysicsLock {
        schema: "resolvent-physics-lock/0.1".into(),
        source_digest,
        semantic_digest: semantic.digest,
        compiler_schema: crate::SCIENTIFIC_SCHEMA_VERSION.into(),
        obligations: vec![],
        evidence: vec![],
        metadata: BTreeMap::new(),
    })
}

pub fn validate_lock(source: &str, lock: &PhysicsLock) -> Result<(), PhysicsError> {
    let actual = Digest::blake3(source.as_bytes());
    if actual != lock.source_digest {
        return Err(PhysicsError::SourceDrift {
            expected: lock.source_digest.clone(),
            actual,
        });
    }
    let current = freeze(source)?;
    if current.semantic_digest != lock.semantic_digest {
        return Err(PhysicsError::SemanticDrift {
            expected: lock.semantic_digest.clone(),
            actual: current.semantic_digest,
        });
    }
    Ok(())
}

/// Embed a validated physics source in Rust without duplicating its mathematics. The macro
/// generates a tiny typed wrapper; the RSL file remains the source of semantic truth.
#[macro_export]
macro_rules! include_physics {
    ($vis:vis $name:ident = $path:literal) => {
        $vis struct $name;
        impl $name {
            pub const SOURCE: &'static str = include_str!($path);
            pub fn parse() -> Result<$crate::rsl::RslModel, Vec<$crate::source::SourceDiagnostic>> { $crate::rsl::parse_rsl(Self::SOURCE) }
            pub fn elaborate() -> Result<($crate::Context, $crate::rsl::RslModel, $crate::rsl::ElaboratedRsl), $crate::physics::PhysicsError> { $crate::physics::parse_and_elaborate(Self::SOURCE) }
            pub fn freeze() -> Result<$crate::physics::PhysicsLock, $crate::physics::PhysicsError> { $crate::physics::freeze(Self::SOURCE) }
        }
    };
}

/// Inline counterpart for experiments and tests. Production physics should normally use an
/// included `.res` file so the same artifact can be exercised by the CLI and Sinbad Lab.
#[macro_export]
macro_rules! physics {
    ($vis:vis $name:ident = $source:expr) => {
        $vis struct $name;
        impl $name {
            pub const SOURCE: &'static str = $source;
            pub fn parse() -> Result<$crate::rsl::RslModel, Vec<$crate::source::SourceDiagnostic>> { $crate::rsl::parse_rsl(Self::SOURCE) }
            pub fn elaborate() -> Result<($crate::Context, $crate::rsl::RslModel, $crate::rsl::ElaboratedRsl), $crate::physics::PhysicsError> { $crate::physics::parse_and_elaborate(Self::SOURCE) }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::physics!(pub Heat = r#"model Heat { domain Omega dim=2; field T: state H1(1) [K] on Omega; equation latex "T = T"; }"#);
    #[test]
    fn lock_rejects_source_drift() {
        let lock = freeze(Heat::SOURCE).unwrap();
        assert!(validate_lock(Heat::SOURCE, &lock).is_ok());
        assert!(matches!(
            validate_lock(&format!("{} ", Heat::SOURCE), &lock),
            Err(PhysicsError::SourceDrift { .. })
        ));
    }
    #[test]
    fn macro_uses_same_parser_and_elaborator() {
        assert_eq!(Heat::parse().unwrap().name, "Heat");
        let (_ctx, parsed, elaborated) = Heat::elaborate().unwrap();
        assert_eq!(parsed.name, "Heat");
        assert_eq!(elaborated.system.equations.len(), 1);
    }
}
