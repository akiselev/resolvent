//! FC0/FC1 variational-form artifacts.
//!
//! This module is deliberately additive. The V1 scalar form and execution-plan APIs remain
//! available as the differential oracle while V2 establishes truthful, serializable compiler
//! artifacts with a strict distinction between scientific fields and ephemeral form arguments.

use crate::id::Digest;
use crate::scientific::{
    Expr as ScientificExpr, FieldRoleV1, ScientificModel, SpaceFamily, ValueShapeV1,
};
use crate::scientific_weak::{
    WeakLoweringError, WeakOperatorProgram, WeakTerm, lower_scalar_h1_model,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

pub const VARIATIONAL_FORM_V2_SCHEMA: &str = "resolvent-variational-form/2";
pub const FORMULATION_DERIVATION_V2_SCHEMA: &str = "resolvent-formulation-derivation/2";
pub const COMPATIBILITY_WEAK_PROGRAM_V2_SCHEMA: &str = "resolvent-scalar-weak-compatibility/2";
pub const ARTIFACT_ENVELOPE_V2_SCHEMA: &str = "resolvent-compiler-artifact/2";
pub const REFINEMENT_RECEIPT_V2_SCHEMA: &str = "resolvent-refinement-receipt/2";

macro_rules! numeric_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u32);
    };
}

numeric_id!(SpaceRequirementIdV2);
numeric_id!(FormArgumentIdV2);
numeric_id!(FormCoefficientIdV2);
numeric_id!(FormConstantIdV2);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FrameIdV2(pub String);

impl FrameIdV2 {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IndexSetIdV2(pub String);

impl IndexSetIdV2 {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactIdV2(pub Digest);

impl ArtifactIdV2 {
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self, ArtifactCodecErrorV2> {
        Ok(Self(Digest::blake3(&serde_json::to_vec(value)?)))
    }

    pub fn hex(&self) -> &str {
        &self.0.hex
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStageV2 {
    ScientificSystem,
    FormulationDerivation,
    VariationalForm,
    CompatibilityWeakProgram,
    TensorIr,
    QFunctionIr,
    StructuredKernel,
    RealizationPlan,
    Executable,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArtifactRefV2 {
    pub schema: String,
    pub stage: ArtifactStageV2,
    pub artifact_id: ArtifactIdV2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtifactEnvelopeV2<T> {
    pub envelope_schema: String,
    pub payload_schema: String,
    pub stage: ArtifactStageV2,
    pub artifact_id: ArtifactIdV2,
    pub payload: T,
}

#[derive(Serialize)]
struct ArtifactProjectionV2<'a, T> {
    payload_schema: &'a str,
    stage: &'a ArtifactStageV2,
    payload: &'a T,
}

impl<T: Serialize> ArtifactEnvelopeV2<T> {
    pub fn new(
        payload_schema: impl Into<String>,
        stage: ArtifactStageV2,
        payload: T,
    ) -> Result<Self, ArtifactCodecErrorV2> {
        let payload_schema = payload_schema.into();
        let artifact_id = ArtifactIdV2::from_serializable(&ArtifactProjectionV2 {
            payload_schema: &payload_schema,
            stage: &stage,
            payload: &payload,
        })?;
        Ok(Self {
            envelope_schema: ARTIFACT_ENVELOPE_V2_SCHEMA.into(),
            payload_schema,
            stage,
            artifact_id,
            payload,
        })
    }

    pub fn artifact_ref(&self) -> ArtifactRefV2 {
        ArtifactRefV2 {
            schema: self.payload_schema.clone(),
            stage: self.stage.clone(),
            artifact_id: self.artifact_id.clone(),
        }
    }

    pub fn verify(&self) -> Result<(), ArtifactCodecErrorV2> {
        if self.envelope_schema != ARTIFACT_ENVELOPE_V2_SCHEMA {
            return Err(ArtifactCodecErrorV2::InvalidEnvelopeSchema {
                expected: ARTIFACT_ENVELOPE_V2_SCHEMA.into(),
                found: self.envelope_schema.clone(),
            });
        }
        let computed = ArtifactIdV2::from_serializable(&ArtifactProjectionV2 {
            payload_schema: &self.payload_schema,
            stage: &self.stage,
            payload: &self.payload,
        })?;
        if computed == self.artifact_id {
            Ok(())
        } else {
            Err(ArtifactCodecErrorV2::DigestMismatch {
                expected: self.artifact_id.hex().into(),
                computed: computed.hex().into(),
            })
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, ArtifactCodecErrorV2> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn inspect(&self) -> Result<ArtifactInspectionV2, ArtifactCodecErrorV2> {
        self.verify()?;
        let mut summary = BTreeMap::new();
        summary.insert("payload_type".into(), std::any::type_name::<T>().into());
        summary.insert(
            "digest_algorithm".into(),
            self.artifact_id.0.algorithm.clone(),
        );
        Ok(ArtifactInspectionV2 {
            envelope_schema: self.envelope_schema.clone(),
            payload_schema: self.payload_schema.clone(),
            stage: self.stage.clone(),
            artifact_id: self.artifact_id.clone(),
            summary,
            payload: serde_json::to_value(&self.payload)?,
        })
    }
}

impl<T> ArtifactEnvelopeV2<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn from_json(input: &str) -> Result<Self, ArtifactCodecErrorV2> {
        let envelope: Self = serde_json::from_str(input)?;
        envelope.verify()?;
        Ok(envelope)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtifactInspectionV2 {
    pub envelope_schema: String,
    pub payload_schema: String,
    pub stage: ArtifactStageV2,
    pub artifact_id: ArtifactIdV2,
    pub summary: BTreeMap<String, String>,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub enum ArtifactCodecErrorV2 {
    Serialization(serde_json::Error),
    DigestMismatch { expected: String, computed: String },
    InvalidEnvelopeSchema { expected: String, found: String },
    InvalidArtifact(String),
}

impl fmt::Display for ArtifactCodecErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(f, "artifact serialization failed: {error}"),
            Self::DigestMismatch { expected, computed } => write!(
                f,
                "artifact digest mismatch: expected {expected}, computed {computed}"
            ),
            Self::InvalidEnvelopeSchema { expected, found } => write!(
                f,
                "artifact envelope schema mismatch: expected {expected}, found {found}"
            ),
            Self::InvalidArtifact(message) => write!(f, "invalid compiler artifact: {message}"),
        }
    }
}

impl Error for ArtifactCodecErrorV2 {}

impl From<serde_json::Error> for ArtifactCodecErrorV2 {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverityV2 {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormDiagnosticV2 {
    pub code: String,
    pub severity: DiagnosticSeverityV2,
    pub path: String,
    pub message: String,
}

impl FormDiagnosticV2 {
    pub fn error(
        code: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverityV2::Error,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormValidationErrorV2 {
    pub diagnostics: Vec<FormDiagnosticV2>,
}

impl FormValidationErrorV2 {
    pub fn has_code(&self, code: &str) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code)
    }
}

impl fmt::Display for FormValidationErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "variational form failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for FormValidationErrorV2 {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefinementReceiptV2 {
    pub schema: String,
    pub source: ArtifactRefV2,
    pub result: ArtifactRefV2,
    pub pass: String,
    pub pass_version: String,
    pub options_digest: ArtifactIdV2,
    pub assumptions: Vec<String>,
    pub obligations: Vec<String>,
    pub diagnostics: Vec<FormDiagnosticV2>,
}

impl RefinementReceiptV2 {
    pub fn new<T: Serialize>(
        source: ArtifactRefV2,
        result: ArtifactRefV2,
        pass: impl Into<String>,
        pass_version: impl Into<String>,
        options: &T,
    ) -> Result<Self, ArtifactCodecErrorV2> {
        Ok(Self {
            schema: REFINEMENT_RECEIPT_V2_SCHEMA.into(),
            source,
            result,
            pass: pass.into(),
            pass_version: pass_version.into(),
            options_digest: ArtifactIdV2::from_serializable(options)?,
            assumptions: vec![],
            obligations: vec![],
            diagnostics: vec![],
        })
    }

    pub fn semantic_digest(&self) -> Result<ArtifactIdV2, ArtifactCodecErrorV2> {
        ArtifactIdV2::from_serializable(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarKindV2 {
    Real32,
    Real64,
    Complex32,
    Complex64,
}

impl ScalarKindV2 {
    pub fn is_complex(self) -> bool {
        matches!(self, Self::Complex32 | Self::Complex64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceV2 {
    Contravariant,
    Covariant,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisKindV2 {
    Spatial {
        frame: FrameIdV2,
        variance: VarianceV2,
        extent: u16,
    },
    Species {
        set: IndexSetIdV2,
        extent: u16,
    },
    SlipSystem {
        set: IndexSetIdV2,
        extent: u16,
    },
    NetworkNode {
        set: IndexSetIdV2,
        extent: u16,
    },
    NetworkBranch {
        set: IndexSetIdV2,
        extent: u16,
    },
    MaterialComponent {
        set: IndexSetIdV2,
        extent: u16,
    },
    Algebraic {
        set: IndexSetIdV2,
        extent: u16,
    },
}

impl AxisKindV2 {
    pub fn extent(&self) -> u16 {
        match self {
            Self::Spatial { extent, .. }
            | Self::Species { extent, .. }
            | Self::SlipSystem { extent, .. }
            | Self::NetworkNode { extent, .. }
            | Self::NetworkBranch { extent, .. }
            | Self::MaterialComponent { extent, .. }
            | Self::Algebraic { extent, .. } => *extent,
        }
    }

    /// Compatibility for a metric contraction (`dot`/`inner`). Spatial axes must share
    /// frame and extent; the metric supplies the variance conversion explicitly implied by
    /// these nodes. Non-spatial axes must be identical.
    fn metric_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Spatial {
                    frame: left_frame,
                    extent: left_extent,
                    ..
                },
                Self::Spatial {
                    frame: right_frame,
                    extent: right_extent,
                    ..
                },
            ) => left_frame == right_frame && left_extent == right_extent,
            _ => self == other,
        }
    }

    /// Compatibility for an index contraction with no implicit metric. Spatial indices must
    /// be in the same frame, have the same extent, and carry opposite variance.
    fn dual_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Spatial {
                    frame: left_frame,
                    variance: left_variance,
                    extent: left_extent,
                },
                Self::Spatial {
                    frame: right_frame,
                    variance: right_variance,
                    extent: right_extent,
                },
            ) => {
                left_frame == right_frame
                    && left_extent == right_extent
                    && left_variance != right_variance
            }
            _ => self == other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TensorTypeV2 {
    pub scalar: ScalarKindV2,
    pub axes: Vec<AxisKindV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity_kind: Option<String>,
}

impl TensorTypeV2 {
    pub fn scalar(scalar: ScalarKindV2) -> Self {
        Self {
            scalar,
            axes: vec![],
            quantity_kind: None,
        }
    }

    pub fn vector(scalar: ScalarKindV2, axis: AxisKindV2) -> Self {
        Self {
            scalar,
            axes: vec![axis],
            quantity_kind: None,
        }
    }

    pub fn rank(&self) -> usize {
        self.axes.len()
    }

    pub fn is_scalar(&self) -> bool {
        self.axes.is_empty()
    }

    fn without_quantity(mut self) -> Self {
        self.quantity_kind = None;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceRequirementV2 {
    pub id: SpaceRequirementIdV2,
    pub domain: String,
    pub family: SpaceFamily,
    pub value_type: TensorTypeV2,
    pub frame: FrameIdV2,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeLevelV2 {
    Current,
    Previous(u16),
    Stage(u16),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormArgumentV2 {
    pub id: FormArgumentIdV2,
    pub name: String,
    pub number: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<u16>,
    pub space: SpaceRequirementIdV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormCoefficientV2 {
    pub id: FormCoefficientIdV2,
    pub field: String,
    pub space: SpaceRequirementIdV2,
    pub time_level: TimeLevelV2,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormConstantV2 {
    pub id: FormConstantIdV2,
    pub name: String,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSideV2 {
    Plus,
    Minus,
    Left,
    Right,
    Exterior,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureV2 {
    Cell {
        domain: String,
        region: Option<String>,
    },
    ExteriorFacet {
        domain: String,
        boundary: String,
    },
    InteriorFacet {
        domain: String,
        region: Option<String>,
    },
    Interface {
        interface: String,
        left_domain: String,
        right_domain: String,
    },
    Ridge {
        domain: String,
        region: String,
    },
    Vertex {
        domain: String,
        set: String,
    },
}

impl MeasureV2 {
    fn domains(&self) -> Vec<&str> {
        match self {
            Self::Cell { domain, .. }
            | Self::ExteriorFacet { domain, .. }
            | Self::InteriorFacet { domain, .. }
            | Self::Ridge { domain, .. }
            | Self::Vertex { domain, .. } => vec![domain],
            Self::Interface {
                left_domain,
                right_domain,
                ..
            } => vec![left_domain, right_domain],
        }
    }

    fn side_allowed(&self, side: TraceSideV2) -> bool {
        match self {
            Self::InteriorFacet { .. } => matches!(side, TraceSideV2::Plus | TraceSideV2::Minus),
            Self::Interface { .. } => matches!(side, TraceSideV2::Left | TraceSideV2::Right),
            Self::ExteriorFacet { .. } => matches!(side, TraceSideV2::Exterior),
            Self::Cell { .. } | Self::Ridge { .. } | Self::Vertex { .. } => false,
        }
    }

    fn requires_explicit_operand_sides(&self) -> bool {
        matches!(self, Self::InteriorFacet { .. } | Self::Interface { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConjugatedOperandV2 {
    Left,
    Right,
}

/// Canonical finite-element convention for `inner(a, b)`: the second operand is
/// conjugated for complex scalar kinds. This keeps forms linear in coefficients and
/// conjugate-linear in test arguments.
pub const INNER_CONJUGATED_OPERAND_V2: ConjugatedOperandV2 = ConjugatedOperandV2::Right;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjointKindV2 {
    Transpose,
    Hermitian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractionPairV2 {
    pub left_axis: u16,
    pub right_axis: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormExprV2 {
    Literal {
        value: f64,
        value_type: TensorTypeV2,
    },
    Argument(FormArgumentIdV2),
    Coefficient(FormCoefficientIdV2),
    Constant(FormConstantIdV2),
    Scientific {
        expression: ScientificExpr,
        value_type: TensorTypeV2,
    },
    Neg(Box<FormExprV2>),
    Add(Vec<FormExprV2>),
    Product(Vec<FormExprV2>),
    TimeDerivative(Box<FormExprV2>),
    Gradient {
        value: Box<FormExprV2>,
        frame: FrameIdV2,
        dimension: u16,
    },
    /// Bilinear metric contraction. This node never conjugates either operand.
    Dot {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
    },
    /// Sesquilinear metric contraction. For complex scalars the right operand is conjugated.
    /// The convention is exposed by [`INNER_CONJUGATED_OPERAND_V2`] so downstream
    /// lowering cannot silently choose the opposite convention.
    Inner {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
    },
    Contract {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
        pairs: Vec<ContractionPairV2>,
    },
    Conjugate(Box<FormExprV2>),
    /// Axis permutation with either no conjugation (`transpose`) or conjugation
    /// (`hermitian`).
    Adjoint {
        value: Box<FormExprV2>,
        permutation: Vec<u16>,
        kind: AdjointKindV2,
    },
    Trace {
        value: Box<FormExprV2>,
        side: TraceSideV2,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegralV2 {
    pub label: String,
    pub integrand: FormExprV2,
    pub measure: MeasureV2,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormRoleV2 {
    Objective,
    Residual,
    Jacobian,
    Higher { arity: u16 },
}

impl FormRoleV2 {
    fn expected_arity(self) -> u16 {
        match self {
            Self::Objective => 0,
            Self::Residual => 1,
            Self::Jacobian => 2,
            Self::Higher { arity } => arity,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorClaimKindV2 {
    Symmetric,
    ComplexSymmetric,
    Hermitian,
    PositiveDefinite,
    SkewSymmetric,
    SkewHermitian,
    Conservation,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvidenceBackedOperatorClaimV2 {
    pub claim: OperatorClaimKindV2,
    pub evidence: ArtifactRefV2,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormCapabilitiesV2 {
    #[serde(default)]
    pub derivative_artifacts: Vec<ArtifactRefV2>,
    #[serde(default)]
    pub operator_claims: Vec<EvidenceBackedOperatorClaimV2>,
}

impl FormCapabilitiesV2 {
    fn canonicalize(&mut self) {
        self.derivative_artifacts.sort();
        self.derivative_artifacts.dedup();
        self.operator_claims.sort();
        self.operator_claims.dedup();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalFormV2 {
    pub schema: String,
    pub name: String,
    pub role: FormRoleV2,
    pub scalar_kind: ScalarKindV2,
    pub spaces: Vec<SpaceRequirementV2>,
    pub arguments: Vec<FormArgumentV2>,
    pub coefficients: Vec<FormCoefficientV2>,
    pub constants: Vec<FormConstantV2>,
    pub integrals: Vec<IntegralV2>,
    pub derivation: ArtifactRefV2,
    #[serde(default)]
    pub obligations: Vec<String>,
    #[serde(default)]
    pub capabilities: FormCapabilitiesV2,
}

struct TypeEnvironmentV2<'a> {
    spaces: BTreeMap<SpaceRequirementIdV2, &'a SpaceRequirementV2>,
    arguments: BTreeMap<FormArgumentIdV2, &'a FormArgumentV2>,
    coefficients: BTreeMap<FormCoefficientIdV2, &'a FormCoefficientV2>,
    constants: BTreeMap<FormConstantIdV2, &'a FormConstantV2>,
}

impl VariationalFormV2 {
    pub fn canonicalize(&mut self) {
        self.spaces.sort_by_key(|space| space.id);
        self.arguments
            .sort_by_key(|argument| (argument.number, argument.part, argument.id));
        self.coefficients.sort_by(|left, right| {
            (&left.field, &left.time_level, left.id).cmp(&(
                &right.field,
                &right.time_level,
                right.id,
            ))
        });
        self.constants
            .sort_by(|left, right| (&left.name, left.id).cmp(&(&right.name, right.id)));
        // Integral order is provenance-significant: derivation receipts address generated
        // integrals by index. Determinism comes from structured serialization, not by silently
        // reordering a formulation after its derivation has been recorded.
        self.obligations.sort();
        self.obligations.dedup();
        self.capabilities.canonicalize();
    }

    pub fn semantic_digest(&self) -> Result<ArtifactIdV2, ArtifactCodecErrorV2> {
        let mut normalized = self.clone();
        normalized.canonicalize();
        ArtifactIdV2::from_serializable(&(VARIATIONAL_FORM_V2_SCHEMA, normalized))
    }

    pub fn into_envelope(
        mut self,
    ) -> Result<ArtifactEnvelopeV2<VariationalFormV2>, ArtifactCodecErrorV2> {
        self.canonicalize();
        ArtifactEnvelopeV2::new(
            VARIATIONAL_FORM_V2_SCHEMA,
            ArtifactStageV2::VariationalForm,
            self,
        )
    }

    pub fn arity(&self) -> u16 {
        self.arguments
            .iter()
            .map(|argument| argument.number)
            .max()
            .map_or(0, |number| number + 1)
    }

    pub fn validate(&self) -> Result<(), FormValidationErrorV2> {
        let mut diagnostics = Vec::new();
        if self.schema != VARIATIONAL_FORM_V2_SCHEMA {
            diagnostics.push(FormDiagnosticV2::error(
                "FORM-V2-SCHEMA",
                "schema",
                format!(
                    "expected `{VARIATIONAL_FORM_V2_SCHEMA}`, found `{}`",
                    self.schema
                ),
            ));
        }

        let mut spaces = BTreeMap::new();
        for (index, space) in self.spaces.iter().enumerate() {
            if spaces.insert(space.id, space).is_some() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-SPACE",
                    format!("spaces[{index}].id"),
                    format!("duplicate space id {}", space.id.0),
                ));
            }
            if space.domain.is_empty() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-EMPTY-DOMAIN",
                    format!("spaces[{index}].domain"),
                    "space requirements need an explicit domain",
                ));
            }
        }

        let mut arguments = BTreeMap::new();
        let mut argument_slots = BTreeSet::new();
        for (index, argument) in self.arguments.iter().enumerate() {
            if arguments.insert(argument.id, argument).is_some() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-ARGUMENT",
                    format!("arguments[{index}].id"),
                    format!("duplicate argument id {}", argument.id.0),
                ));
            }
            if !argument_slots.insert((argument.number, argument.part)) {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-ARGUMENT-SLOT",
                    format!("arguments[{index}]"),
                    format!(
                        "argument number {} part {:?} is declared more than once",
                        argument.number, argument.part
                    ),
                ));
            }
            if !spaces.contains_key(&argument.space) {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-UNKNOWN-ARGUMENT-SPACE",
                    format!("arguments[{index}].space"),
                    format!("unknown space id {}", argument.space.0),
                ));
            }
        }

        let mut coefficients = BTreeMap::new();
        let mut coefficient_slots = BTreeSet::new();
        for (index, coefficient) in self.coefficients.iter().enumerate() {
            if coefficients.insert(coefficient.id, coefficient).is_some() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-COEFFICIENT",
                    format!("coefficients[{index}].id"),
                    format!("duplicate coefficient id {}", coefficient.id.0),
                ));
            }
            if !coefficient_slots.insert((&coefficient.field, &coefficient.time_level)) {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-COEFFICIENT-SLOT",
                    format!("coefficients[{index}]"),
                    format!(
                        "field `{}` at {:?} is declared more than once",
                        coefficient.field, coefficient.time_level
                    ),
                ));
            }
            match spaces.get(&coefficient.space) {
                Some(space) if space.value_type != coefficient.value_type => {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-COEFFICIENT-TYPE",
                        format!("coefficients[{index}].value_type"),
                        format!(
                            "coefficient `{}` type does not match space {}",
                            coefficient.field, coefficient.space.0
                        ),
                    ));
                }
                Some(_) => {}
                None => diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-UNKNOWN-COEFFICIENT-SPACE",
                    format!("coefficients[{index}].space"),
                    format!("unknown space id {}", coefficient.space.0),
                )),
            }
        }

        let mut constants = BTreeMap::new();
        let mut constant_names = BTreeSet::new();
        for (index, constant) in self.constants.iter().enumerate() {
            if constants.insert(constant.id, constant).is_some() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-CONSTANT",
                    format!("constants[{index}].id"),
                    format!("duplicate constant id {}", constant.id.0),
                ));
            }
            if !constant_names.insert(&constant.name) {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DUPLICATE-CONSTANT-NAME",
                    format!("constants[{index}].name"),
                    format!("constant `{}` is declared more than once", constant.name),
                ));
            }
        }

        let arity = self.arity();
        let expected_arity = self.role.expected_arity();
        if arity != expected_arity {
            diagnostics.push(FormDiagnosticV2::error(
                "FORM-V2-ARITY",
                "arguments",
                format!(
                    "role {:?} requires arity {expected_arity}, but arguments imply arity {arity}",
                    self.role
                ),
            ));
        }
        let numbers = self
            .arguments
            .iter()
            .map(|argument| argument.number)
            .collect::<BTreeSet<_>>();
        let expected_numbers = (0..arity).collect::<BTreeSet<_>>();
        if numbers != expected_numbers {
            diagnostics.push(FormDiagnosticV2::error(
                "FORM-V2-ARGUMENT-NUMBERS",
                "arguments",
                "argument numbers must be contiguous from zero",
            ));
        }

        let coefficient_fields = self
            .coefficients
            .iter()
            .map(|coefficient| coefficient.field.as_str())
            .collect::<BTreeSet<_>>();
        let environment = TypeEnvironmentV2 {
            spaces,
            arguments,
            coefficients,
            constants,
        };
        let known_domains = self
            .spaces
            .iter()
            .map(|space| space.domain.as_str())
            .collect::<BTreeSet<_>>();

        if self.integrals.is_empty() {
            diagnostics.push(FormDiagnosticV2::error(
                "FORM-V2-EMPTY",
                "integrals",
                "a form must contain at least one integral",
            ));
        }

        for (index, integral) in self.integrals.iter().enumerate() {
            let path = format!("integrals[{index}]");
            for domain in integral.measure.domains() {
                if !known_domains.contains(domain) {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-UNKNOWN-MEASURE-DOMAIN",
                        format!("{path}.measure"),
                        format!("measure references undeclared domain `{domain}`"),
                    ));
                }
            }
            if let Some(value_type) = infer_type(
                &integral.integrand,
                &environment,
                &format!("{path}.integrand"),
                &mut diagnostics,
            ) {
                if !value_type.is_scalar() {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-NONSCALAR-INTEGRAND",
                        format!("{path}.integrand"),
                        format!(
                            "integrals must be scalar after contraction; inferred rank {}",
                            value_type.rank()
                        ),
                    ));
                }
                if value_type.scalar != self.scalar_kind {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-FORM-SCALAR-KIND",
                        format!("{path}.integrand"),
                        format!(
                            "integrand scalar kind {:?} differs from form scalar kind {:?}",
                            value_type.scalar, self.scalar_kind
                        ),
                    ));
                }
            }
            validate_measure_sides(integral, &path, &coefficient_fields, &mut diagnostics);
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(FormValidationErrorV2 { diagnostics })
        }
    }

    pub fn extract_block(
        &self,
        row_part: Option<u16>,
        column_part: Option<u16>,
    ) -> Result<FormBlockV2, FormValidationErrorV2> {
        self.validate()?;
        let argument_by_id = self
            .arguments
            .iter()
            .map(|argument| (argument.id, argument))
            .collect::<BTreeMap<_, _>>();
        let mut integral_indices = Vec::new();
        for (index, integral) in self.integrals.iter().enumerate() {
            let mut referenced = BTreeSet::new();
            collect_argument_ids(&integral.integrand, &mut referenced);
            let row_matches = if self.arity() == 0 {
                true
            } else {
                referenced.iter().any(|id| {
                    let argument = argument_by_id[id];
                    argument.number == 0 && argument.part == row_part
                })
            };
            let column_matches = if self.arity() < 2 {
                true
            } else {
                referenced.iter().any(|id| {
                    let argument = argument_by_id[id];
                    argument.number == 1 && argument.part == column_part
                })
            };
            if row_matches && column_matches {
                integral_indices.push(index);
            }
        }
        if integral_indices.is_empty() {
            return Err(FormValidationErrorV2 {
                diagnostics: vec![FormDiagnosticV2::error(
                    "FORM-V2-EMPTY-BLOCK",
                    "integrals",
                    format!(
                        "no integrals reference row part {row_part:?} and column part {column_part:?}"
                    ),
                )],
            });
        }
        Ok(FormBlockV2 {
            form_digest: self
                .semantic_digest()
                .map_err(|error| FormValidationErrorV2 {
                    diagnostics: vec![FormDiagnosticV2::error(
                        "FORM-V2-DIGEST",
                        "form",
                        error.to_string(),
                    )],
                })?,
            row_part,
            column_part,
            integral_indices,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormBlockV2 {
    pub form_digest: ArtifactIdV2,
    pub row_part: Option<u16>,
    pub column_part: Option<u16>,
    pub integral_indices: Vec<usize>,
}

impl FormBlockV2 {
    pub fn semantic_digest(&self) -> Result<ArtifactIdV2, ArtifactCodecErrorV2> {
        ArtifactIdV2::from_serializable(self)
    }
}

fn infer_type(
    expression: &FormExprV2,
    environment: &TypeEnvironmentV2<'_>,
    path: &str,
    diagnostics: &mut Vec<FormDiagnosticV2>,
) -> Option<TensorTypeV2> {
    match expression {
        FormExprV2::Literal { value, value_type } => {
            if !value.is_finite() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-NONFINITE-LITERAL",
                    path,
                    "form literals must be finite",
                ));
            }
            Some(value_type.clone())
        }
        FormExprV2::Argument(id) => {
            let Some(argument) = environment.arguments.get(id) else {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-UNKNOWN-ARGUMENT",
                    path,
                    format!("unknown form argument id {}", id.0),
                ));
                return None;
            };
            environment
                .spaces
                .get(&argument.space)
                .map(|space| space.value_type.clone())
        }
        FormExprV2::Coefficient(id) => {
            let Some(coefficient) = environment.coefficients.get(id) else {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-UNKNOWN-COEFFICIENT",
                    path,
                    format!("unknown form coefficient id {}", id.0),
                ));
                return None;
            };
            Some(coefficient.value_type.clone())
        }
        FormExprV2::Constant(id) => {
            let Some(constant) = environment.constants.get(id) else {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-UNKNOWN-CONSTANT",
                    path,
                    format!("unknown form constant id {}", id.0),
                ));
                return None;
            };
            Some(constant.value_type.clone())
        }
        FormExprV2::Scientific { value_type, .. } => Some(value_type.clone()),
        FormExprV2::Neg(value)
        | FormExprV2::TimeDerivative(value)
        | FormExprV2::Conjugate(value)
        | FormExprV2::Trace { value, .. } => {
            infer_type(value, environment, &format!("{path}.value"), diagnostics)
        }
        FormExprV2::Add(values) => {
            if values.is_empty() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-EMPTY-ADD",
                    path,
                    "addition requires at least one operand",
                ));
                return None;
            }
            let mut inferred = None;
            for (index, value) in values.iter().enumerate() {
                let next = infer_type(
                    value,
                    environment,
                    &format!("{path}.values[{index}]"),
                    diagnostics,
                );
                match (&inferred, next) {
                    (None, Some(next)) => inferred = Some(next),
                    (Some(current), Some(next)) if *current != next => diagnostics.push(
                        FormDiagnosticV2::error(
                            "FORM-V2-ADD-TYPE-MISMATCH",
                            format!("{path}.values[{index}]"),
                            "addition operands must have identical scalar, axes, frame, and quantity types",
                        ),
                    ),
                    _ => {}
                }
            }
            inferred
        }
        FormExprV2::Product(values) => {
            if values.is_empty() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-EMPTY-PRODUCT",
                    path,
                    "products require at least one operand",
                ));
                return None;
            }
            let mut inferred: Option<TensorTypeV2> = None;
            for (index, value) in values.iter().enumerate() {
                let Some(next) = infer_type(
                    value,
                    environment,
                    &format!("{path}.values[{index}]"),
                    diagnostics,
                ) else {
                    continue;
                };
                inferred = match inferred {
                    None => Some(next),
                    Some(current) if current.scalar != next.scalar => {
                        diagnostics.push(FormDiagnosticV2::error(
                            "FORM-V2-SCALAR-KIND-MISMATCH",
                            format!("{path}.values[{index}]"),
                            "product operands must have the same real/complex precision",
                        ));
                        Some(current)
                    }
                    Some(current) if current.is_scalar() => Some(next.without_quantity()),
                    Some(current) if next.is_scalar() => Some(current.without_quantity()),
                    Some(current) => {
                        diagnostics.push(FormDiagnosticV2::error(
                            "FORM-V2-NONSCALAR-PRODUCT",
                            format!("{path}.values[{index}]"),
                            "tensor-tensor products require an explicit dot, inner, or contract node",
                        ));
                        Some(current)
                    }
                };
            }
            inferred
        }
        FormExprV2::Gradient {
            value,
            frame,
            dimension,
        } => {
            let mut value_type =
                infer_type(value, environment, &format!("{path}.value"), diagnostics)?;
            if *dimension == 0 {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-GRADIENT-DIMENSION",
                    format!("{path}.dimension"),
                    "gradient dimension must be positive",
                ));
            }
            value_type.axes.push(AxisKindV2::Spatial {
                frame: frame.clone(),
                variance: VarianceV2::Covariant,
                extent: *dimension,
            });
            value_type.quantity_kind = None;
            Some(value_type)
        }
        FormExprV2::Dot { left, right } => {
            let left_type = infer_type(left, environment, &format!("{path}.left"), diagnostics)?;
            let right_type = infer_type(right, environment, &format!("{path}.right"), diagnostics)?;
            if left_type.scalar != right_type.scalar {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-SCALAR-KIND-MISMATCH",
                    path,
                    "dot operands must have identical scalar kinds",
                ));
            }
            if left_type.rank() != 1 || right_type.rank() != 1 {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-DOT-RANK",
                    path,
                    "dot is defined for rank-one operands; use explicit contract for higher rank",
                ));
            } else if !left_type.axes[0].metric_compatible(&right_type.axes[0]) {
                push_axis_mismatch(path, &left_type.axes[0], &right_type.axes[0], diagnostics);
            }
            Some(TensorTypeV2::scalar(left_type.scalar))
        }
        FormExprV2::Inner { left, right } => {
            let left_type = infer_type(left, environment, &format!("{path}.left"), diagnostics)?;
            let right_type = infer_type(right, environment, &format!("{path}.right"), diagnostics)?;
            if left_type.scalar != right_type.scalar {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-SCALAR-KIND-MISMATCH",
                    path,
                    "inner operands must have identical scalar kinds",
                ));
            }
            if left_type.rank() != right_type.rank() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-INNER-RANK",
                    path,
                    "inner operands must have identical ranks",
                ));
            } else {
                for (index, (left_axis, right_axis)) in
                    left_type.axes.iter().zip(&right_type.axes).enumerate()
                {
                    if !left_axis.metric_compatible(right_axis) {
                        push_axis_mismatch(
                            &format!("{path}.axes[{index}]"),
                            left_axis,
                            right_axis,
                            diagnostics,
                        );
                    }
                }
            }
            Some(TensorTypeV2::scalar(left_type.scalar))
        }
        FormExprV2::Contract { left, right, pairs } => {
            let left_type = infer_type(left, environment, &format!("{path}.left"), diagnostics)?;
            let right_type = infer_type(right, environment, &format!("{path}.right"), diagnostics)?;
            if left_type.scalar != right_type.scalar {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-SCALAR-KIND-MISMATCH",
                    path,
                    "contract operands must have identical scalar kinds",
                ));
            }
            let mut left_used = BTreeSet::new();
            let mut right_used = BTreeSet::new();
            for (index, pair) in pairs.iter().enumerate() {
                let left_index = pair.left_axis as usize;
                let right_index = pair.right_axis as usize;
                if left_index >= left_type.rank() || right_index >= right_type.rank() {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-CONTRACTION-AXIS",
                        format!("{path}.pairs[{index}]"),
                        "contraction axis is out of bounds",
                    ));
                    continue;
                }
                if !left_used.insert(left_index) || !right_used.insert(right_index) {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-CONTRACTION-DUPLICATE",
                        format!("{path}.pairs[{index}]"),
                        "a tensor axis can be contracted at most once",
                    ));
                }
                if !left_type.axes[left_index].dual_compatible(&right_type.axes[right_index]) {
                    push_axis_mismatch(
                        &format!("{path}.pairs[{index}]"),
                        &left_type.axes[left_index],
                        &right_type.axes[right_index],
                        diagnostics,
                    );
                }
            }
            let mut axes = left_type
                .axes
                .iter()
                .enumerate()
                .filter(|(index, _)| !left_used.contains(index))
                .map(|(_, axis)| axis.clone())
                .collect::<Vec<_>>();
            axes.extend(
                right_type
                    .axes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !right_used.contains(index))
                    .map(|(_, axis)| axis.clone()),
            );
            Some(TensorTypeV2 {
                scalar: left_type.scalar,
                axes,
                quantity_kind: None,
            })
        }
        FormExprV2::Adjoint {
            value,
            permutation,
            kind: _,
        } => {
            let value_type = infer_type(value, environment, &format!("{path}.value"), diagnostics)?;
            if permutation.len() != value_type.rank() {
                diagnostics.push(FormDiagnosticV2::error(
                    "FORM-V2-ADJOINT-PERMUTATION",
                    format!("{path}.permutation"),
                    "adjoint permutation length must equal tensor rank",
                ));
                return Some(value_type);
            }
            let mut used = BTreeSet::new();
            let mut axes = Vec::new();
            for (index, axis) in permutation.iter().copied().enumerate() {
                let axis = axis as usize;
                if axis >= value_type.rank() || !used.insert(axis) {
                    diagnostics.push(FormDiagnosticV2::error(
                        "FORM-V2-ADJOINT-PERMUTATION",
                        format!("{path}.permutation[{index}]"),
                        "adjoint permutation must contain each input axis exactly once",
                    ));
                    continue;
                }
                axes.push(value_type.axes[axis].clone());
            }
            Some(TensorTypeV2 {
                scalar: value_type.scalar,
                axes,
                quantity_kind: value_type.quantity_kind,
            })
        }
    }
}

fn push_axis_mismatch(
    path: &str,
    left: &AxisKindV2,
    right: &AxisKindV2,
    diagnostics: &mut Vec<FormDiagnosticV2>,
) {
    let code = match (left, right) {
        (
            AxisKindV2::Spatial {
                frame: left_frame, ..
            },
            AxisKindV2::Spatial {
                frame: right_frame, ..
            },
        ) if left_frame != right_frame => "FORM-V2-AXIS-FRAME-MISMATCH",
        _ if left.extent() != right.extent() => "FORM-V2-AXIS-EXTENT-MISMATCH",
        (
            AxisKindV2::Spatial {
                variance: left_variance,
                ..
            },
            AxisKindV2::Spatial {
                variance: right_variance,
                ..
            },
        ) if left_variance == right_variance => "FORM-V2-AXIS-VARIANCE-MISMATCH",
        _ => "FORM-V2-AXIS-KIND-MISMATCH",
    };
    diagnostics.push(FormDiagnosticV2::error(
        code,
        path,
        format!("cannot contract axis {left:?} with {right:?}"),
    ));
}

fn validate_measure_sides(
    integral: &IntegralV2,
    path: &str,
    coefficient_fields: &BTreeSet<&str>,
    diagnostics: &mut Vec<FormDiagnosticV2>,
) {
    let mut traces = Vec::new();
    let mut unsided_operands = Vec::new();
    collect_sides(
        &integral.integrand,
        false,
        coefficient_fields,
        &mut traces,
        &mut unsided_operands,
    );
    for side in traces {
        if !integral.measure.side_allowed(side) {
            diagnostics.push(FormDiagnosticV2::error(
                "FORM-V2-TRACE-SIDE",
                format!("{path}.integrand"),
                format!(
                    "trace side {side:?} is invalid for measure {:?}",
                    integral.measure
                ),
            ));
        }
    }
    if integral.measure.requires_explicit_operand_sides() && !unsided_operands.is_empty() {
        diagnostics.push(FormDiagnosticV2::error(
            "FORM-V2-UNSIDED-FACET-OPERAND",
            format!("{path}.integrand"),
            format!(
                "interior/interface operands require explicit trace sides; unsided operands: {}",
                unsided_operands.join(", ")
            ),
        ));
    }
}

fn collect_sides(
    expression: &FormExprV2,
    under_trace: bool,
    coefficient_fields: &BTreeSet<&str>,
    traces: &mut Vec<TraceSideV2>,
    unsided_operands: &mut Vec<String>,
) {
    match expression {
        FormExprV2::Argument(id) if !under_trace => {
            unsided_operands.push(format!("argument:{}", id.0));
        }
        FormExprV2::Coefficient(id) if !under_trace => {
            unsided_operands.push(format!("coefficient:{}", id.0));
        }
        FormExprV2::Trace { value, side } => {
            traces.push(*side);
            collect_sides(value, true, coefficient_fields, traces, unsided_operands);
        }
        FormExprV2::Neg(value)
        | FormExprV2::TimeDerivative(value)
        | FormExprV2::Conjugate(value)
        | FormExprV2::Gradient { value, .. }
        | FormExprV2::Adjoint { value, .. } => {
            collect_sides(
                value,
                under_trace,
                coefficient_fields,
                traces,
                unsided_operands,
            );
        }
        FormExprV2::Add(values) | FormExprV2::Product(values) => {
            for value in values {
                collect_sides(
                    value,
                    under_trace,
                    coefficient_fields,
                    traces,
                    unsided_operands,
                );
            }
        }
        FormExprV2::Dot { left, right }
        | FormExprV2::Inner { left, right }
        | FormExprV2::Contract { left, right, .. } => {
            collect_sides(
                left,
                under_trace,
                coefficient_fields,
                traces,
                unsided_operands,
            );
            collect_sides(
                right,
                under_trace,
                coefficient_fields,
                traces,
                unsided_operands,
            );
        }
        FormExprV2::Scientific { expression, .. } if !under_trace => {
            let mut referenced = BTreeSet::new();
            collect_scientific_coefficient_names(expression, coefficient_fields, &mut referenced);
            unsided_operands.extend(
                referenced
                    .into_iter()
                    .map(|name| format!("scientific-coefficient:{name}")),
            );
        }
        FormExprV2::Literal { .. }
        | FormExprV2::Constant(_)
        | FormExprV2::Scientific { .. }
        | FormExprV2::Argument(_)
        | FormExprV2::Coefficient(_) => {}
    }
}

fn collect_scientific_coefficient_names(
    expression: &ScientificExpr,
    coefficient_fields: &BTreeSet<&str>,
    out: &mut BTreeSet<String>,
) {
    match expression {
        ScientificExpr::Name(name) if coefficient_fields.contains(name.as_str()) => {
            out.insert(name.clone());
        }
        ScientificExpr::Unary { arg, .. } => {
            collect_scientific_coefficient_names(arg, coefficient_fields, out);
        }
        ScientificExpr::Binary { lhs, rhs, .. } => {
            collect_scientific_coefficient_names(lhs, coefficient_fields, out);
            collect_scientific_coefficient_names(rhs, coefficient_fields, out);
        }
        ScientificExpr::Call { args, .. } => {
            for arg in args {
                collect_scientific_coefficient_names(arg, coefficient_fields, out);
            }
        }
        ScientificExpr::Index { value, indices } => {
            collect_scientific_coefficient_names(value, coefficient_fields, out);
            for index in indices {
                collect_scientific_coefficient_names(index, coefficient_fields, out);
            }
        }
        ScientificExpr::Vector(values) => {
            for value in values {
                collect_scientific_coefficient_names(value, coefficient_fields, out);
            }
        }
        ScientificExpr::Number { .. } | ScientificExpr::String(_) | ScientificExpr::Name(_) => {}
    }
}

fn collect_argument_ids(expression: &FormExprV2, out: &mut BTreeSet<FormArgumentIdV2>) {
    match expression {
        FormExprV2::Argument(id) => {
            out.insert(*id);
        }
        FormExprV2::Neg(value)
        | FormExprV2::TimeDerivative(value)
        | FormExprV2::Conjugate(value)
        | FormExprV2::Gradient { value, .. }
        | FormExprV2::Adjoint { value, .. }
        | FormExprV2::Trace { value, .. } => collect_argument_ids(value, out),
        FormExprV2::Add(values) | FormExprV2::Product(values) => {
            for value in values {
                collect_argument_ids(value, out);
            }
        }
        FormExprV2::Dot { left, right }
        | FormExprV2::Inner { left, right }
        | FormExprV2::Contract { left, right, .. } => {
            collect_argument_ids(left, out);
            collect_argument_ids(right, out);
        }
        FormExprV2::Literal { .. }
        | FormExprV2::Coefficient(_)
        | FormExprV2::Constant(_)
        | FormExprV2::Scientific { .. } => {}
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulationDerivationV2 {
    pub schema: String,
    pub model: ArtifactRefV2,
    pub equation: String,
    pub profile: String,
    pub steps: Vec<DerivationStepV2>,
    pub generated_boundary_terms: Vec<GeneratedBoundaryTermV2>,
    pub assumptions: Vec<String>,
    pub obligations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationStepV2 {
    pub rule: String,
    pub source_term: usize,
    pub result_integral: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedBoundaryTermV2 {
    pub field: String,
    pub domain: String,
    pub coefficient: ScientificExpr,
    pub disposition: BoundaryTermDispositionV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryTermDispositionV2 {
    MatchedCondition { condition: String },
    AssumedZero { assumption: String },
    Deferred { obligation: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarFormCompatibilityBundleV2 {
    pub schema: String,
    pub model: ArtifactRefV2,
    pub derivations: Vec<ArtifactEnvelopeV2<FormulationDerivationV2>>,
    pub forms: Vec<ArtifactEnvelopeV2<VariationalFormV2>>,
    pub legacy: ArtifactEnvelopeV2<WeakOperatorProgram>,
    pub receipts: Vec<RefinementReceiptV2>,
}

impl ScalarFormCompatibilityBundleV2 {
    pub fn legacy_program(&self) -> &WeakOperatorProgram {
        &self.legacy.payload
    }

    pub fn verify(&self) -> Result<(), ArtifactCodecErrorV2> {
        if self.schema != "resolvent-scalar-form-compatibility-bundle/2" {
            return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                "unexpected compatibility bundle schema `{}`",
                self.schema
            )));
        }
        self.legacy.verify()?;
        if self.legacy.stage != ArtifactStageV2::CompatibilityWeakProgram
            || self.legacy.payload_schema != COMPATIBILITY_WEAK_PROGRAM_V2_SCHEMA
        {
            return Err(ArtifactCodecErrorV2::InvalidArtifact(
                "legacy payload is not the V2 scalar compatibility stage".into(),
            ));
        }

        let mut known = BTreeSet::from([self.model.clone(), self.legacy.artifact_ref()]);
        for derivation in &self.derivations {
            derivation.verify()?;
            if derivation.stage != ArtifactStageV2::FormulationDerivation
                || derivation.payload_schema != FORMULATION_DERIVATION_V2_SCHEMA
                || derivation.payload.schema != FORMULATION_DERIVATION_V2_SCHEMA
            {
                return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                    "derivation `{}` has an inconsistent stage or schema",
                    derivation.artifact_id.hex()
                )));
            }
            if derivation.payload.model != self.model {
                return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                    "derivation `{}` references a different scientific model",
                    derivation.artifact_id.hex()
                )));
            }
            for generated in &derivation.payload.generated_boundary_terms {
                if let BoundaryTermDispositionV2::Deferred { obligation } = &generated.disposition
                    && !derivation.payload.obligations.contains(obligation)
                {
                    return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                        "deferred boundary term obligation `{obligation}` is absent from the derivation"
                    )));
                }
            }
            known.insert(derivation.artifact_ref());
        }
        for form in &self.forms {
            form.verify()?;
            if form.stage != ArtifactStageV2::VariationalForm
                || form.payload_schema != VARIATIONAL_FORM_V2_SCHEMA
                || form.payload.schema != VARIATIONAL_FORM_V2_SCHEMA
            {
                return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                    "form `{}` has an inconsistent stage or schema",
                    form.artifact_id.hex()
                )));
            }
            form.payload
                .validate()
                .map_err(|error| ArtifactCodecErrorV2::InvalidArtifact(error.to_string()))?;
            let derivation = self
                .derivations
                .iter()
                .find(|candidate| candidate.artifact_ref() == form.payload.derivation)
                .ok_or_else(|| {
                    ArtifactCodecErrorV2::InvalidArtifact(format!(
                        "form `{}` has a dangling derivation reference",
                        form.artifact_id.hex()
                    ))
                })?;
            for step in &derivation.payload.steps {
                if step.result_integral >= form.payload.integrals.len() {
                    return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                        "derivation step points to missing integral {} in form `{}`",
                        step.result_integral, form.payload.name
                    )));
                }
            }
            known.insert(form.artifact_ref());
        }
        let mut required_receipts = BTreeSet::new();
        for derivation in &self.derivations {
            required_receipts.insert((self.model.clone(), derivation.artifact_ref()));
        }
        for form in &self.forms {
            required_receipts.insert((form.payload.derivation.clone(), form.artifact_ref()));
            required_receipts.insert((form.artifact_ref(), self.legacy.artifact_ref()));
        }

        let mut observed_receipts = BTreeSet::new();
        for receipt in &self.receipts {
            if receipt.schema != REFINEMENT_RECEIPT_V2_SCHEMA {
                return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                    "unexpected refinement receipt schema `{}`",
                    receipt.schema
                )));
            }
            receipt.semantic_digest()?;
            if !known.contains(&receipt.source) || !known.contains(&receipt.result) {
                return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                    "receipt `{}` -> `{}` has a dangling artifact reference",
                    receipt.source.artifact_id.hex(),
                    receipt.result.artifact_id.hex()
                )));
            }
            observed_receipts.insert((receipt.source.clone(), receipt.result.clone()));
        }
        if let Some((source, result)) = required_receipts.difference(&observed_receipts).next() {
            return Err(ArtifactCodecErrorV2::InvalidArtifact(format!(
                "missing refinement receipt `{}` -> `{}`",
                source.artifact_id.hex(),
                result.artifact_id.hex()
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalarAdapterOptionsV2 {
    pub scalar_kind: ScalarKindV2,
    pub formulation_profile: String,
}

impl Default for ScalarAdapterOptionsV2 {
    fn default() -> Self {
        Self {
            scalar_kind: ScalarKindV2::Real64,
            formulation_profile: "scalar_h1_galerkin_compatibility".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalarFormAdapterErrorV2 {
    pub diagnostic: FormDiagnosticV2,
}

impl fmt::Display for ScalarFormAdapterErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}: {}",
            self.diagnostic.code, self.diagnostic.path, self.diagnostic.message
        )
    }
}

impl Error for ScalarFormAdapterErrorV2 {}

pub fn adapt_scalar_h1_model_v2(
    model: &ScientificModel,
) -> Result<ScalarFormCompatibilityBundleV2, ScalarFormAdapterErrorV2> {
    adapt_scalar_h1_model_v2_with_options(model, &ScalarAdapterOptionsV2::default())
}

pub fn adapt_scalar_h1_model_v2_with_options(
    model: &ScientificModel,
    options: &ScalarAdapterOptionsV2,
) -> Result<ScalarFormCompatibilityBundleV2, ScalarFormAdapterErrorV2> {
    let weak = lower_scalar_h1_model(model).map_err(map_weak_error)?;
    let model_id = semantic_model_id(model)
        .map_err(|error| adapter_error("FORM-V2-MODEL-DIGEST", "model", error.to_string()))?;
    let model_ref = ArtifactRefV2 {
        schema: "resolvent-scientific-v1/model-semantic".into(),
        stage: ArtifactStageV2::ScientificSystem,
        artifact_id: model_id,
    };

    let mut physical_fields = model
        .fields
        .iter()
        .filter(|field| {
            !matches!(
                field.role,
                FieldRoleV1::Test | FieldRoleV1::Trial | FieldRoleV1::Parameter
            )
        })
        .collect::<Vec<_>>();
    physical_fields.sort_by(|left, right| left.name.cmp(&right.name));
    for field in &physical_fields {
        if field.space.family != SpaceFamily::H1 || field.shape != ValueShapeV1::Scalar {
            return Err(adapter_error(
                "FORM-V2-SCALAR-ADAPTER-SPACE",
                format!("fields.{}", field.name),
                format!(
                    "scalar compatibility accepts scalar H1 fields; `{}` is {:?} {:?}",
                    field.name, field.space.family, field.shape
                ),
            ));
        }
    }

    let mut derivations = Vec::new();
    let mut forms = Vec::new();
    let mut receipts = Vec::new();
    for (block_index, block) in weak.blocks.iter().enumerate() {
        let primary = model
            .fields
            .iter()
            .find(|field| field.name == block.primary_field)
            .ok_or_else(|| {
                adapter_error(
                    "FORM-V2-PRIMARY-FIELD",
                    format!("equations.{}", block.name),
                    format!("primary field `{}` is not declared", block.primary_field),
                )
            })?;
        let domain = block
            .domain
            .clone()
            .unwrap_or_else(|| primary.domain.clone());
        let domain_decl = model
            .domains
            .iter()
            .find(|candidate| candidate.name == domain)
            .ok_or_else(|| {
                adapter_error(
                    "FORM-V2-DOMAIN",
                    format!("equations.{}", block.name),
                    format!("domain `{domain}` is not declared"),
                )
            })?;
        let frame = FrameIdV2::new(format!("{}::spatial", domain_decl.name));

        let mut spaces = Vec::new();
        let mut coefficients = Vec::new();
        let mut coefficient_ids = BTreeMap::new();
        for (field_index, field) in physical_fields.iter().enumerate() {
            let field_domain = model
                .domains
                .iter()
                .find(|candidate| candidate.name == field.domain)
                .ok_or_else(|| {
                    adapter_error(
                        "FORM-V2-DOMAIN",
                        format!("fields.{}", field.name),
                        format!("domain `{}` is not declared", field.domain),
                    )
                })?;
            let space_id = SpaceRequirementIdV2(field_index as u32 + 1);
            let value_type = TensorTypeV2::scalar(options.scalar_kind);
            spaces.push(SpaceRequirementV2 {
                id: space_id,
                domain: field.domain.clone(),
                family: SpaceFamily::H1,
                value_type: value_type.clone(),
                frame: FrameIdV2::new(format!("{}::spatial", field_domain.name)),
            });
            let coefficient_id = FormCoefficientIdV2(field_index as u32);
            coefficient_ids.insert(field.name.as_str(), coefficient_id);
            coefficients.push(FormCoefficientV2 {
                id: coefficient_id,
                field: field.name.clone(),
                space: space_id,
                time_level: TimeLevelV2::Current,
                value_type,
            });
        }

        let primary_space = coefficients
            .iter()
            .find(|coefficient| coefficient.field == block.primary_field)
            .map(|coefficient| coefficient.space)
            .ok_or_else(|| {
                adapter_error(
                    "FORM-V2-PRIMARY-COEFFICIENT",
                    format!("equations.{}", block.name),
                    format!(
                        "primary field `{}` is not a physical coefficient",
                        block.primary_field
                    ),
                )
            })?;
        let test_id = FormArgumentIdV2(0);
        let arguments = vec![FormArgumentV2 {
            id: test_id,
            name: format!("test_{}", block.primary_field),
            number: 0,
            part: (weak.blocks.len() > 1).then_some(block_index as u16),
            space: primary_space,
        }];

        let mut integrals = Vec::new();
        let mut steps = Vec::new();
        let mut generated_boundary_terms = Vec::new();
        let mut obligations = Vec::new();
        for (term_index, term) in block.terms.iter().enumerate() {
            let (integrand, rule) = match term {
                WeakTerm::Mass { field, coefficient } => {
                    let coefficient_id = *coefficient_ids.get(field.as_str()).ok_or_else(|| {
                        adapter_error(
                            "FORM-V2-UNKNOWN-FIELD",
                            format!("equations.{}.terms[{term_index}]", block.name),
                            format!("mass term references undeclared field `{field}`"),
                        )
                    })?;
                    (
                        FormExprV2::Product(vec![
                            FormExprV2::Scientific {
                                expression: coefficient.clone(),
                                value_type: TensorTypeV2::scalar(options.scalar_kind),
                            },
                            FormExprV2::TimeDerivative(Box::new(FormExprV2::Coefficient(
                                coefficient_id,
                            ))),
                            FormExprV2::Argument(test_id),
                        ]),
                        "recognize_mass",
                    )
                }
                WeakTerm::Diffusion { field, coefficient } => {
                    let coefficient_id = *coefficient_ids.get(field.as_str()).ok_or_else(|| {
                        adapter_error(
                            "FORM-V2-UNKNOWN-FIELD",
                            format!("equations.{}.terms[{term_index}]", block.name),
                            format!("diffusion term references undeclared field `{field}`"),
                        )
                    })?;
                    let obligation = format!("boundary-term:{}:{term_index}", block.name);
                    obligations.push(obligation.clone());
                    generated_boundary_terms.push(GeneratedBoundaryTermV2 {
                        field: field.clone(),
                        domain: domain.clone(),
                        coefficient: coefficient.clone(),
                        disposition: BoundaryTermDispositionV2::Deferred { obligation },
                    });
                    (
                        FormExprV2::Product(vec![
                            FormExprV2::Scientific {
                                expression: coefficient.clone(),
                                value_type: TensorTypeV2::scalar(options.scalar_kind),
                            },
                            FormExprV2::Inner {
                                left: Box::new(FormExprV2::Gradient {
                                    value: Box::new(FormExprV2::Coefficient(coefficient_id)),
                                    frame: frame.clone(),
                                    dimension: domain_decl.dimension as u16,
                                }),
                                right: Box::new(FormExprV2::Gradient {
                                    value: Box::new(FormExprV2::Argument(test_id)),
                                    frame: frame.clone(),
                                    dimension: domain_decl.dimension as u16,
                                }),
                            },
                        ]),
                        "integrate_divergence_by_parts",
                    )
                }
                WeakTerm::Pointwise { expression } => (
                    FormExprV2::Product(vec![
                        FormExprV2::Scientific {
                            expression: expression.clone(),
                            value_type: TensorTypeV2::scalar(options.scalar_kind),
                        },
                        FormExprV2::Argument(test_id),
                    ]),
                    "retain_pointwise_term",
                ),
            };
            integrals.push(IntegralV2 {
                label: format!("{}::term::{term_index}", block.name),
                integrand,
                measure: MeasureV2::Cell {
                    domain: domain.clone(),
                    region: None,
                },
                metadata: BTreeMap::from([(
                    "compatibility_source".into(),
                    format!("weak_term:{term_index}"),
                )]),
            });
            steps.push(DerivationStepV2 {
                rule: rule.into(),
                source_term: term_index,
                result_integral: term_index,
            });
        }

        let derivation_payload = FormulationDerivationV2 {
            schema: FORMULATION_DERIVATION_V2_SCHEMA.into(),
            model: model_ref.clone(),
            equation: block.name.clone(),
            profile: options.formulation_profile.clone(),
            steps,
            generated_boundary_terms,
            assumptions: vec![],
            obligations: obligations.clone(),
        };
        let derivation = ArtifactEnvelopeV2::new(
            FORMULATION_DERIVATION_V2_SCHEMA,
            ArtifactStageV2::FormulationDerivation,
            derivation_payload,
        )
        .map_err(|error| {
            adapter_error(
                "FORM-V2-DERIVATION-ARTIFACT",
                format!("equations.{}", block.name),
                error.to_string(),
            )
        })?;

        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: format!("{}::{}::residual", model.name, block.name),
            role: FormRoleV2::Residual,
            scalar_kind: options.scalar_kind,
            spaces,
            arguments,
            coefficients,
            constants: vec![],
            integrals,
            derivation: derivation.artifact_ref(),
            obligations,
            capabilities: FormCapabilitiesV2::default(),
        };
        form.validate().map_err(|error| ScalarFormAdapterErrorV2 {
            diagnostic: error.diagnostics.into_iter().next().unwrap_or_else(|| {
                FormDiagnosticV2::error(
                    "FORM-V2-VALIDATION",
                    format!("equations.{}", block.name),
                    "form validation failed without a diagnostic",
                )
            }),
        })?;
        let form = form.into_envelope().map_err(|error| {
            adapter_error(
                "FORM-V2-FORM-ARTIFACT",
                format!("equations.{}", block.name),
                error.to_string(),
            )
        })?;

        let mut derivation_receipt = RefinementReceiptV2::new(
            model_ref.clone(),
            derivation.artifact_ref(),
            "derive_scalar_h1_formulation",
            "fc0-fc1/1",
            options,
        )
        .map_err(|error| adapter_error("FORM-V2-RECEIPT", "receipts", error.to_string()))?;
        derivation_receipt.obligations = derivation.payload.obligations.clone();
        let mut form_receipt = RefinementReceiptV2::new(
            derivation.artifact_ref(),
            form.artifact_ref(),
            "emit_variational_form_v2",
            "fc0-fc1/1",
            options,
        )
        .map_err(|error| adapter_error("FORM-V2-RECEIPT", "receipts", error.to_string()))?;
        form_receipt.obligations = form.payload.obligations.clone();
        receipts.push(derivation_receipt);
        receipts.push(form_receipt);
        derivations.push(derivation);
        forms.push(form);
    }

    let legacy = ArtifactEnvelopeV2::new(
        COMPATIBILITY_WEAK_PROGRAM_V2_SCHEMA,
        ArtifactStageV2::CompatibilityWeakProgram,
        weak,
    )
    .map_err(|error| adapter_error("FORM-V2-LEGACY-ARTIFACT", "legacy", error.to_string()))?;
    for form in &forms {
        let mut receipt = RefinementReceiptV2::new(
            form.artifact_ref(),
            legacy.artifact_ref(),
            "scalar_v1_compatibility_projection",
            "fc0-fc1/1",
            options,
        )
        .map_err(|error| adapter_error("FORM-V2-RECEIPT", "receipts", error.to_string()))?;
        receipt.assumptions.push(
            "projection is admitted only for the scalar H1 mass/diffusion/pointwise subset".into(),
        );
        receipts.push(receipt);
    }

    let bundle = ScalarFormCompatibilityBundleV2 {
        schema: "resolvent-scalar-form-compatibility-bundle/2".into(),
        model: model_ref,
        derivations,
        forms,
        legacy,
        receipts,
    };
    bundle
        .verify()
        .map_err(|error| adapter_error("FORM-V2-BUNDLE", "bundle", error.to_string()))?;
    Ok(bundle)
}

fn map_weak_error(error: WeakLoweringError) -> ScalarFormAdapterErrorV2 {
    match error {
        WeakLoweringError::MissingEquation(equation) => adapter_error(
            "FORM-V2-MISSING-EQUATION",
            format!("equations.{equation}"),
            format!("equation `{equation}` was not found"),
        ),
        WeakLoweringError::AliasCycle(alias) => adapter_error(
            "FORM-V2-ALIAS-CYCLE",
            format!("aliases.{alias}"),
            format!("cyclic semantic alias while expanding `{alias}`"),
        ),
        WeakLoweringError::MissingPrimaryField(equation) => adapter_error(
            "FORM-V2-MISSING-PRIMARY-FIELD",
            format!("equations.{equation}"),
            "equation has no primary differential field",
        ),
        WeakLoweringError::UnsupportedDifferential {
            equation,
            expression,
        } => adapter_error(
            "FORM-V2-UNLOWERED-DIFFERENTIAL",
            format!("equations.{equation}"),
            format!("unsupported differential expression: {expression:?}"),
        ),
    }
}

fn adapter_error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ScalarFormAdapterErrorV2 {
    ScalarFormAdapterErrorV2 {
        diagnostic: FormDiagnosticV2::error(code, path, message),
    }
}

fn semantic_model_id(model: &ScientificModel) -> Result<ArtifactIdV2, ArtifactCodecErrorV2> {
    let mut value = serde_json::to_value(model)?;
    canonicalize_semantic_value(&mut value);
    ArtifactIdV2::from_serializable(&value)
}

fn canonicalize_semantic_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("span");
            for child in map.values_mut() {
                canonicalize_semantic_value(child);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items.iter_mut() {
                canonicalize_semantic_value(child);
            }
            if items.iter().all(|item| {
                item.get("name")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
            }) {
                items.sort_by(|left, right| {
                    left.get("name")
                        .and_then(serde_json::Value::as_str)
                        .cmp(&right.get("name").and_then(serde_json::Value::as_str))
                });
            } else if items.iter().all(|item| item.as_str().is_some()) {
                items.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scientific::parse_scientific_module;

    fn dummy_derivation() -> ArtifactRefV2 {
        ArtifactRefV2 {
            schema: FORMULATION_DERIVATION_V2_SCHEMA.into(),
            stage: ArtifactStageV2::FormulationDerivation,
            artifact_id: ArtifactIdV2(Digest::blake3(b"test-derivation")),
        }
    }

    fn scalar_space(id: u32, domain: &str) -> SpaceRequirementV2 {
        SpaceRequirementV2 {
            id: SpaceRequirementIdV2(id),
            domain: domain.into(),
            family: SpaceFamily::H1,
            value_type: TensorTypeV2::scalar(ScalarKindV2::Real64),
            frame: FrameIdV2::new(format!("{domain}::spatial")),
        }
    }

    #[test]
    fn scalar_adapter_is_lossless_serializable_and_claim_free() {
        let source = r#"
module test.heat;
model Heat {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field T: state scalar H1(order=1) on Omega { time_role = differential; };
  property rho = density(T);
  property cp = specific_heat(T);
  property k = thermal_conductivity(T);
  source Q: VolumetricHeatSource;
  equation energy on Omega { rho * cp * dt(T) - div(k * grad(T)) = Q; }
}
"#;
        let model = parse_scientific_module(source).unwrap().models.remove(0);
        let legacy = lower_scalar_h1_model(&model).unwrap();
        let bundle = adapt_scalar_h1_model_v2(&model).unwrap();
        assert_eq!(bundle.legacy_program(), &legacy);
        assert_eq!(bundle.forms.len(), 1);
        let form = &bundle.forms[0];
        form.payload.validate().unwrap();
        assert!(form.payload.capabilities.derivative_artifacts.is_empty());
        assert!(form.payload.capabilities.operator_claims.is_empty());
        assert_eq!(form.payload.arguments.len(), 1);
        assert!(
            form.payload
                .coefficients
                .iter()
                .any(|field| field.field == "T")
        );
        assert!(
            form.payload
                .obligations
                .iter()
                .any(|value| value.starts_with("boundary-term:"))
        );
        let json = form.to_json_pretty().unwrap();
        let decoded = ArtifactEnvelopeV2::<VariationalFormV2>::from_json(&json).unwrap();
        assert_eq!(decoded.artifact_id, form.artifact_id);
        assert_eq!(
            decoded.payload.semantic_digest().unwrap(),
            form.payload.semantic_digest().unwrap()
        );
        assert_eq!(
            form.inspect().unwrap().stage,
            ArtifactStageV2::VariationalForm
        );
        let mut tampered = form.clone();
        tampered.envelope_schema = "resolvent-compiler-artifact/1".into();
        assert!(matches!(
            tampered.verify(),
            Err(ArtifactCodecErrorV2::InvalidEnvelopeSchema { .. })
        ));
    }

    #[test]
    fn invalid_contraction_reports_frame_mismatch() {
        let left_type = TensorTypeV2::vector(
            ScalarKindV2::Real64,
            AxisKindV2::Spatial {
                frame: FrameIdV2::new("world"),
                variance: VarianceV2::Contravariant,
                extent: 3,
            },
        );
        let right_type = TensorTypeV2::vector(
            ScalarKindV2::Real64,
            AxisKindV2::Spatial {
                frame: FrameIdV2::new("material"),
                variance: VarianceV2::Covariant,
                extent: 3,
            },
        );
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "invalid-frame".into(),
            role: FormRoleV2::Objective,
            scalar_kind: ScalarKindV2::Real64,
            spaces: vec![scalar_space(0, "Omega")],
            arguments: vec![],
            coefficients: vec![],
            constants: vec![
                FormConstantV2 {
                    id: FormConstantIdV2(0),
                    name: "left".into(),
                    value_type: left_type,
                },
                FormConstantV2 {
                    id: FormConstantIdV2(1),
                    name: "right".into(),
                    value_type: right_type,
                },
            ],
            integrals: vec![IntegralV2 {
                label: "bad-inner".into(),
                integrand: FormExprV2::Inner {
                    left: Box::new(FormExprV2::Constant(FormConstantIdV2(0))),
                    right: Box::new(FormExprV2::Constant(FormConstantIdV2(1))),
                },
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: None,
                },
                metadata: BTreeMap::new(),
            }],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        let error = form.validate().unwrap_err();
        assert!(error.has_code("FORM-V2-AXIS-FRAME-MISMATCH"));
    }

    #[test]
    fn mixed_jacobian_blocks_have_stable_digests() {
        let spaces = vec![scalar_space(0, "Omega"), scalar_space(1, "Omega")];
        let arguments = vec![
            FormArgumentV2 {
                id: FormArgumentIdV2(0),
                name: "v0".into(),
                number: 0,
                part: Some(0),
                space: SpaceRequirementIdV2(0),
            },
            FormArgumentV2 {
                id: FormArgumentIdV2(1),
                name: "v1".into(),
                number: 0,
                part: Some(1),
                space: SpaceRequirementIdV2(1),
            },
            FormArgumentV2 {
                id: FormArgumentIdV2(2),
                name: "du0".into(),
                number: 1,
                part: Some(0),
                space: SpaceRequirementIdV2(0),
            },
            FormArgumentV2 {
                id: FormArgumentIdV2(3),
                name: "du1".into(),
                number: 1,
                part: Some(1),
                space: SpaceRequirementIdV2(1),
            },
        ];
        let integral = |label: &str, row: u32, column: u32| IntegralV2 {
            label: label.into(),
            integrand: FormExprV2::Product(vec![
                FormExprV2::Argument(FormArgumentIdV2(row)),
                FormExprV2::Argument(FormArgumentIdV2(column)),
            ]),
            measure: MeasureV2::Cell {
                domain: "Omega".into(),
                region: None,
            },
            metadata: BTreeMap::new(),
        };
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "mixed-jacobian".into(),
            role: FormRoleV2::Jacobian,
            scalar_kind: ScalarKindV2::Real64,
            spaces,
            arguments,
            coefficients: vec![],
            constants: vec![],
            integrals: vec![
                integral("00", 0, 2),
                integral("01", 0, 3),
                integral("10", 1, 2),
                integral("11", 1, 3),
            ],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        form.validate().unwrap();
        let block_01 = form.extract_block(Some(0), Some(1)).unwrap();
        let block_10 = form.extract_block(Some(1), Some(0)).unwrap();
        assert_eq!(block_01.integral_indices.len(), 1);
        assert_eq!(block_10.integral_indices.len(), 1);
        assert_ne!(
            block_01.semantic_digest().unwrap(),
            block_10.semantic_digest().unwrap()
        );
        let json = serde_json::to_string(&form).unwrap();
        let decoded: VariationalFormV2 = serde_json::from_str(&json).unwrap();
        assert_eq!(
            decoded.semantic_digest().unwrap(),
            form.semantic_digest().unwrap()
        );
    }

    #[test]
    fn interior_facet_requires_explicit_sides() {
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "facet".into(),
            role: FormRoleV2::Residual,
            scalar_kind: ScalarKindV2::Real64,
            spaces: vec![scalar_space(0, "Omega")],
            arguments: vec![FormArgumentV2 {
                id: FormArgumentIdV2(0),
                name: "v".into(),
                number: 0,
                part: None,
                space: SpaceRequirementIdV2(0),
            }],
            coefficients: vec![],
            constants: vec![],
            integrals: vec![IntegralV2 {
                label: "unsided".into(),
                integrand: FormExprV2::Argument(FormArgumentIdV2(0)),
                measure: MeasureV2::InteriorFacet {
                    domain: "Omega".into(),
                    region: None,
                },
                metadata: BTreeMap::new(),
            }],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        assert!(
            form.validate()
                .unwrap_err()
                .has_code("FORM-V2-UNSIDED-FACET-OPERAND")
        );
    }

    #[test]
    fn complex_inner_and_hermitian_adjoint_are_distinct_and_typed() {
        let axis = AxisKindV2::Spatial {
            frame: FrameIdV2::new("world"),
            variance: VarianceV2::Contravariant,
            extent: 3,
        };
        let vector = TensorTypeV2::vector(ScalarKindV2::Complex64, axis);
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "complex-functional".into(),
            role: FormRoleV2::Objective,
            scalar_kind: ScalarKindV2::Complex64,
            spaces: vec![SpaceRequirementV2 {
                id: SpaceRequirementIdV2(0),
                domain: "Omega".into(),
                family: SpaceFamily::H1,
                value_type: TensorTypeV2::scalar(ScalarKindV2::Complex64),
                frame: FrameIdV2::new("Omega::spatial"),
            }],
            arguments: vec![],
            coefficients: vec![],
            constants: vec![
                FormConstantV2 {
                    id: FormConstantIdV2(0),
                    name: "a".into(),
                    value_type: vector.clone(),
                },
                FormConstantV2 {
                    id: FormConstantIdV2(1),
                    name: "b".into(),
                    value_type: vector,
                },
            ],
            integrals: vec![IntegralV2 {
                label: "sesquilinear".into(),
                integrand: FormExprV2::Inner {
                    left: Box::new(FormExprV2::Adjoint {
                        value: Box::new(FormExprV2::Constant(FormConstantIdV2(0))),
                        permutation: vec![0],
                        kind: AdjointKindV2::Hermitian,
                    }),
                    right: Box::new(FormExprV2::Constant(FormConstantIdV2(1))),
                },
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: None,
                },
                metadata: BTreeMap::new(),
            }],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        form.validate().unwrap();
        assert!(form.scalar_kind.is_complex());
        assert!(matches!(
            &form.integrals[0].integrand,
            FormExprV2::Inner { .. }
        ));
        let inner_digest = form.semantic_digest().unwrap();
        let mut bilinear = form.clone();
        bilinear.integrals[0].integrand = FormExprV2::Dot {
            left: Box::new(FormExprV2::Adjoint {
                value: Box::new(FormExprV2::Constant(FormConstantIdV2(0))),
                permutation: vec![0],
                kind: AdjointKindV2::Transpose,
            }),
            right: Box::new(FormExprV2::Constant(FormConstantIdV2(1))),
        };
        bilinear.validate().unwrap();
        assert_ne!(inner_digest, bilinear.semantic_digest().unwrap());
    }

    #[test]
    fn explicit_contract_rejects_equal_spatial_variance() {
        let axis = AxisKindV2::Spatial {
            frame: FrameIdV2::new("world"),
            variance: VarianceV2::Covariant,
            extent: 3,
        };
        let tensor = TensorTypeV2::vector(ScalarKindV2::Real64, axis);
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "invalid-variance".into(),
            role: FormRoleV2::Objective,
            scalar_kind: ScalarKindV2::Real64,
            spaces: vec![scalar_space(0, "Omega")],
            arguments: vec![],
            coefficients: vec![],
            constants: vec![
                FormConstantV2 {
                    id: FormConstantIdV2(0),
                    name: "a".into(),
                    value_type: tensor.clone(),
                },
                FormConstantV2 {
                    id: FormConstantIdV2(1),
                    name: "b".into(),
                    value_type: tensor,
                },
            ],
            integrals: vec![IntegralV2 {
                label: "bad-contract".into(),
                integrand: FormExprV2::Contract {
                    left: Box::new(FormExprV2::Constant(FormConstantIdV2(0))),
                    right: Box::new(FormExprV2::Constant(FormConstantIdV2(1))),
                    pairs: vec![ContractionPairV2 {
                        left_axis: 0,
                        right_axis: 0,
                    }],
                },
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: None,
                },
                metadata: BTreeMap::new(),
            }],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        assert!(
            form.validate()
                .unwrap_err()
                .has_code("FORM-V2-AXIS-VARIANCE-MISMATCH")
        );
    }

    #[test]
    fn inner_conjugates_the_test_operand_and_adapter_places_it_on_the_right() {
        assert_eq!(INNER_CONJUGATED_OPERAND_V2, ConjugatedOperandV2::Right);
        let source = r#"
module test.inner;
model InnerConvention {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: state scalar H1(order=1) on Omega { time_role = differential; };
  property k = conductivity(u);
  equation balance on Omega { -div(k * grad(u)) = 0; }
}
"#;
        let model = parse_scientific_module(source).unwrap().models.remove(0);
        let bundle = adapt_scalar_h1_model_v2(&model).unwrap();
        let FormExprV2::Product(values) = &bundle.forms[0].payload.integrals[0].integrand else {
            panic!("diffusion compatibility integral must be a product");
        };
        let inner = values
            .iter()
            .find(|value| matches!(value, FormExprV2::Inner { .. }))
            .expect("diffusion compatibility integral must contain inner");
        let FormExprV2::Inner { left, right } = inner else {
            unreachable!();
        };
        assert!(matches!(
            left.as_ref(),
            FormExprV2::Gradient { value, .. }
                if matches!(value.as_ref(), FormExprV2::Coefficient(_))
        ));
        assert!(matches!(
            right.as_ref(),
            FormExprV2::Gradient { value, .. }
                if matches!(value.as_ref(), FormExprV2::Argument(_))
        ));
    }

    #[test]
    fn scalar_adapter_form_identity_ignores_field_declaration_order() {
        let source_a = r#"
module test.order;
model OrderedFields {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: state scalar H1(order=1) on Omega { time_role = differential; };
  field q: unknown scalar H1(order=1) on Omega;
  equation balance on Omega { dt(u) = q; }
}
"#;
        let source_b = r#"
module test.order;
model OrderedFields {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field q: unknown scalar H1(order=1) on Omega;
  field u: state scalar H1(order=1) on Omega { time_role = differential; };
  equation balance on Omega { dt(u) = q; }
}
"#;
        let model_a = parse_scientific_module(source_a).unwrap().models.remove(0);
        let model_b = parse_scientific_module(source_b).unwrap().models.remove(0);
        let form_a = adapt_scalar_h1_model_v2(&model_a).unwrap().forms.remove(0);
        let form_b = adapt_scalar_h1_model_v2(&model_b).unwrap().forms.remove(0);
        assert_eq!(form_a.artifact_id, form_b.artifact_id);
        assert_eq!(
            form_a.payload.semantic_digest().unwrap(),
            form_b.payload.semantic_digest().unwrap()
        );
    }

    #[test]
    fn capability_order_and_duplicates_do_not_change_form_identity() {
        let source = r#"
module test.capabilities;
model Capabilities {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: state scalar H1(order=1) on Omega { time_role = differential; };
  equation balance on Omega { dt(u) = 0; }
}
"#;
        let model = parse_scientific_module(source).unwrap().models.remove(0);
        let mut first = adapt_scalar_h1_model_v2(&model)
            .unwrap()
            .forms
            .remove(0)
            .payload;
        let evidence_a = ArtifactRefV2 {
            schema: "evidence/1".into(),
            stage: ArtifactStageV2::Executable,
            artifact_id: ArtifactIdV2(Digest::blake3(b"evidence-a")),
        };
        let evidence_b = ArtifactRefV2 {
            schema: "evidence/1".into(),
            stage: ArtifactStageV2::Executable,
            artifact_id: ArtifactIdV2(Digest::blake3(b"evidence-b")),
        };
        first.capabilities.derivative_artifacts =
            vec![evidence_b.clone(), evidence_a.clone(), evidence_b.clone()];
        first.capabilities.operator_claims = vec![
            EvidenceBackedOperatorClaimV2 {
                claim: OperatorClaimKindV2::Hermitian,
                evidence: evidence_b.clone(),
            },
            EvidenceBackedOperatorClaimV2 {
                claim: OperatorClaimKindV2::ComplexSymmetric,
                evidence: evidence_a.clone(),
            },
        ];
        let mut second = first.clone();
        second.capabilities.derivative_artifacts.reverse();
        second.capabilities.operator_claims.reverse();
        assert_eq!(
            first.semantic_digest().unwrap(),
            second.semantic_digest().unwrap()
        );
    }

    #[test]
    fn interior_facets_reject_scientific_field_references_without_trace_sides() {
        let mut form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "hidden-facet-coefficient".into(),
            role: FormRoleV2::Residual,
            scalar_kind: ScalarKindV2::Real64,
            spaces: vec![scalar_space(0, "Omega")],
            arguments: vec![FormArgumentV2 {
                id: FormArgumentIdV2(0),
                name: "v".into(),
                number: 0,
                part: None,
                space: SpaceRequirementIdV2(0),
            }],
            coefficients: vec![FormCoefficientV2 {
                id: FormCoefficientIdV2(0),
                field: "u".into(),
                space: SpaceRequirementIdV2(0),
                time_level: TimeLevelV2::Current,
                value_type: TensorTypeV2::scalar(ScalarKindV2::Real64),
            }],
            constants: vec![],
            integrals: vec![IntegralV2 {
                label: "hidden".into(),
                integrand: FormExprV2::Product(vec![
                    FormExprV2::Scientific {
                        expression: ScientificExpr::Name("u".into()),
                        value_type: TensorTypeV2::scalar(ScalarKindV2::Real64),
                    },
                    FormExprV2::Trace {
                        value: Box::new(FormExprV2::Argument(FormArgumentIdV2(0))),
                        side: TraceSideV2::Plus,
                    },
                ]),
                measure: MeasureV2::InteriorFacet {
                    domain: "Omega".into(),
                    region: None,
                },
                metadata: BTreeMap::new(),
            }],
            derivation: dummy_derivation(),
            obligations: vec![],
            capabilities: FormCapabilitiesV2::default(),
        };
        let error = form.validate().unwrap_err();
        assert!(error.has_code("FORM-V2-UNSIDED-FACET-OPERAND"));
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("scientific-coefficient:u") })
        );

        let FormExprV2::Product(values) = &mut form.integrals[0].integrand else {
            unreachable!();
        };
        values[0] = FormExprV2::Trace {
            value: Box::new(values[0].clone()),
            side: TraceSideV2::Minus,
        };
        form.validate().unwrap();
    }

    #[test]
    fn compatibility_bundle_requires_every_refinement_receipt_edge() {
        let source = r#"
module test.receipts;
model Receipts {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: state scalar H1(order=1) on Omega { time_role = differential; };
  equation balance on Omega { dt(u) = 0; }
}
"#;
        let model = parse_scientific_module(source).unwrap().models.remove(0);
        let mut bundle = adapt_scalar_h1_model_v2(&model).unwrap();
        let form = bundle.forms[0].artifact_ref();
        let legacy = bundle.legacy.artifact_ref();
        bundle
            .receipts
            .retain(|receipt| receipt.source != form || receipt.result != legacy);
        let error = bundle.verify().unwrap_err();
        assert!(error.to_string().contains("missing refinement receipt"));
    }
}
