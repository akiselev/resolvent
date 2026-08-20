#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BoundaryTermDispositionV2 {
    Emitted,
    AssumedZeroByCompatibility {
        reason: String,
    },
    RequiresBoundaryCondition {
        obligation: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedBoundaryTermV2 {
    pub id: String,
    pub equation: String,
    pub field: ScientificFieldIdV2,
    pub description: String,
    pub disposition: BoundaryTermDispositionV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulationDerivationV2 {
    pub id: FormulationDerivationIdV2,
    pub source_model: String,
    pub method_family: String,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub introduced_arguments: Vec<ArgumentIdV2>,
    #[serde(default)]
    pub generated_boundary_terms: Vec<GeneratedBoundaryTermV2>,
    #[serde(default)]
    pub assumptions: Vec<String>,
}

impl FormulationDerivationV2 {
    fn canonicalize(&mut self) {
        self.choices.sort();
        self.choices.dedup();
        self.introduced_arguments.sort();
        self.introduced_arguments.dedup();
        self.generated_boundary_terms
            .sort_by(|left, right| left.id.cmp(&right.id));
        self.assumptions.sort();
        self.assumptions.dedup();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarH1CompatibilityV2 {
    pub schema: String,
    pub source_digest: Digest,
    pub program: WeakOperatorProgram,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DerivativeArtifactStatusV2 {
    NotGenerated,
    Generated {
        artifact: Digest,
        evidence: Vec<Digest>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivativeArtifactsV2 {
    pub exact_jacobian: DerivativeArtifactStatusV2,
    pub dynamic_jacobian: DerivativeArtifactStatusV2,
    pub preconditioning_jacobian: DerivativeArtifactStatusV2,
    pub jvp: DerivativeArtifactStatusV2,
    pub vjp: DerivativeArtifactStatusV2,
    #[serde(default)]
    pub parameter_actions: Vec<DerivativeArtifactStatusV2>,
}

impl Default for DerivativeArtifactsV2 {
    fn default() -> Self {
        Self {
            exact_jacobian: DerivativeArtifactStatusV2::NotGenerated,
            dynamic_jacobian: DerivativeArtifactStatusV2::NotGenerated,
            preconditioning_jacobian: DerivativeArtifactStatusV2::NotGenerated,
            jvp: DerivativeArtifactStatusV2::NotGenerated,
            vjp: DerivativeArtifactStatusV2::NotGenerated,
            parameter_actions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorClaimV2 {
    pub property: String,
    #[serde(default)]
    pub conditions: Vec<String>,
    pub evidence: Vec<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulationReceiptV2 {
    pub schema: String,
    pub source_schema: String,
    pub source_digest: Digest,
    pub target_semantic_digest: Digest,
    pub relation: String,
    pub producer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactProvenanceV2 {
    pub producer: String,
    pub producer_version: String,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalArtifactPayloadV2 {
    pub form: VariationalFormV2,
    pub spaces: Vec<SpaceRequirementV2>,
    pub frames: Vec<FrameV2>,
    #[serde(default)]
    pub index_sets: Vec<IndexSetV2>,
    pub derivation: FormulationDerivationV2,
    pub receipt: FormulationReceiptV2,
    #[serde(default)]
    pub derivatives: DerivativeArtifactsV2,
    #[serde(default)]
    pub operator_claims: Vec<OperatorClaimV2>,
    pub provenance: ArtifactProvenanceV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scalar_h1_compatibility: Option<ScalarH1CompatibilityV2>,
}

impl VariationalArtifactPayloadV2 {
    fn canonicalize(&mut self) {
        self.form.canonicalize();
        self.spaces.sort_by(|left, right| left.id.cmp(&right.id));
        self.frames.sort_by(|left, right| left.id.cmp(&right.id));
        self.index_sets.sort_by(|left, right| left.id.cmp(&right.id));
        self.derivation.canonicalize();
        self.operator_claims
            .sort_by(|left, right| left.property.cmp(&right.property));
        for claim in &mut self.operator_claims {
            claim.conditions.sort();
            claim.conditions.dedup();
            claim.evidence.sort();
            claim.evidence.dedup();
        }
    }
}
