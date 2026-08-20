#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalFormArtifactV2 {
    pub artifact_schema: String,
    pub artifact_digest: Digest,
    pub semantic_digest: Digest,
    pub payload: VariationalArtifactPayloadV2,
}

#[derive(Serialize)]
struct SemanticDigestInput<'a> {
    form: &'a VariationalFormV2,
    spaces: &'a [SpaceRequirementV2],
    frames: &'a [FrameV2],
    index_sets: &'a [IndexSetV2],
    derivation: &'a FormulationDerivationV2,
}

#[derive(Serialize)]
struct ArtifactDigestInput<'a> {
    schema: &'a str,
    semantic_digest: &'a Digest,
    payload: &'a VariationalArtifactPayloadV2,
}

impl VariationalFormArtifactV2 {
    pub fn build(
        mut payload: VariationalArtifactPayloadV2,
    ) -> Result<Self, FormV2Error> {
        payload.canonicalize();
        validate_payload(&payload, false)?;
        let semantic_digest = digest_serialized(&SemanticDigestInput {
            form: &payload.form,
            spaces: &payload.spaces,
            frames: &payload.frames,
            index_sets: &payload.index_sets,
            derivation: &payload.derivation,
        })?;
        payload.receipt.target_semantic_digest = semantic_digest.clone();
        validate_payload(&payload, true)?;
        let artifact_digest = digest_serialized(&ArtifactDigestInput {
            schema: VARIATIONAL_ARTIFACT_V2_SCHEMA,
            semantic_digest: &semantic_digest,
            payload: &payload,
        })?;
        Ok(Self {
            artifact_schema: VARIATIONAL_ARTIFACT_V2_SCHEMA.into(),
            artifact_digest,
            semantic_digest,
            payload,
        })
    }

    pub fn verify(&self) -> Result<(), FormV2Error> {
        if self.artifact_schema != VARIATIONAL_ARTIFACT_V2_SCHEMA {
            return Err(FormV2Error::Schema {
                expected: VARIATIONAL_ARTIFACT_V2_SCHEMA.into(),
                got: self.artifact_schema.clone(),
            });
        }
        let mut canonical = self.payload.clone();
        canonical.canonicalize();
        if canonical != self.payload {
            return Err(FormV2Error::NonCanonicalArtifact);
        }
        validate_payload(&self.payload, true)?;
        let semantic_digest = digest_serialized(&SemanticDigestInput {
            form: &self.payload.form,
            spaces: &self.payload.spaces,
            frames: &self.payload.frames,
            index_sets: &self.payload.index_sets,
            derivation: &self.payload.derivation,
        })?;
        if semantic_digest != self.semantic_digest
            || self.payload.receipt.target_semantic_digest != self.semantic_digest
        {
            return Err(FormV2Error::DigestMismatch {
                which: "semantic".into(),
            });
        }
        let artifact_digest = digest_serialized(&ArtifactDigestInput {
            schema: &self.artifact_schema,
            semantic_digest: &self.semantic_digest,
            payload: &self.payload,
        })?;
        if artifact_digest != self.artifact_digest {
            return Err(FormV2Error::DigestMismatch {
                which: "artifact".into(),
            });
        }
        Ok(())
    }

    pub fn to_pretty_json(&self) -> Result<String, FormV2Error> {
        self.verify()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| FormV2Error::Serialization(error.to_string()))
    }

    pub fn from_json(source: &str) -> Result<Self, FormV2Error> {
        let artifact: Self = serde_json::from_str(source)
            .map_err(|error| FormV2Error::Serialization(error.to_string()))?;
        artifact.verify()?;
        Ok(artifact)
    }

    pub fn scalar_h1_compatibility_program(
        &self,
    ) -> Result<&WeakOperatorProgram, FormV2Error> {
        self.verify()?;
        let compatibility = self
            .payload
            .scalar_h1_compatibility
            .as_ref()
            .ok_or(FormV2Error::MissingCompatibilityOracle)?;
        let digest = digest_serialized(&compatibility.program)?;
        if digest != compatibility.source_digest
            || digest != self.payload.receipt.source_digest
        {
            return Err(FormV2Error::DigestMismatch {
                which: "compatibility source".into(),
            });
        }
        Ok(&compatibility.program)
    }

    pub fn inspect(&self) -> Result<VariationalFormInspectionV2, FormV2Error> {
        self.verify()?;
        let mut measures = BTreeMap::<String, usize>::new();
        for integral in &self.payload.form.integrals {
            *measures
                .entry(integral.measure.kind_name().into())
                .or_default() += 1;
        }
        Ok(VariationalFormInspectionV2 {
            schema: "resolvent-variational-inspection/2".into(),
            artifact_digest: self.artifact_digest.clone(),
            semantic_digest: self.semantic_digest.clone(),
            name: self.payload.form.name.clone(),
            arity: self.payload.form.arity(),
            argument_parts: self.payload.form.argument_parts(),
            scalar_kind: self.payload.form.scalar_kind,
            integrals: self.payload.form.integrals.len(),
            measures,
            generated_boundary_terms: self.payload.derivation.generated_boundary_terms.len(),
            derivatives: self.payload.derivatives.clone(),
            operator_claims: self.payload.operator_claims.len(),
            compatibility_source_digest: self
                .payload
                .scalar_h1_compatibility
                .as_ref()
                .map(|compatibility| compatibility.source_digest.clone()),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariationalFormInspectionV2 {
    pub schema: String,
    pub artifact_digest: Digest,
    pub semantic_digest: Digest,
    pub name: String,
    pub arity: u16,
    pub argument_parts: BTreeMap<u16, Vec<Option<u16>>>,
    pub scalar_kind: ScalarKindV2,
    pub integrals: usize,
    pub measures: BTreeMap<String, usize>,
    pub generated_boundary_terms: usize,
    pub derivatives: DerivativeArtifactsV2,
    pub operator_claims: usize,
    pub compatibility_source_digest: Option<Digest>,
}
