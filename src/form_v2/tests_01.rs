
    fn scalar_space() -> (FrameV2, SpaceRequirementV2) {
        let frame = FrameV2 {
            id: FrameIdV2::new("frame::Omega"),
            dimension: 2,
        };
        let space = SpaceRequirementV2 {
            id: SpaceRequirementIdV2::new("space::u"),
            domain: "Omega".into(),
            spatial_frame: frame.id.clone(),
            sobolev: SobolevSpaceV2::H1,
            value_type: TensorTypeV2::scalar(ScalarKindV2::Real64),
        };
        (frame, space)
    }

    fn authored_payload(
        form: VariationalFormV2,
        spaces: Vec<SpaceRequirementV2>,
        frames: Vec<FrameV2>,
    ) -> VariationalArtifactPayloadV2 {
        let source_digest = Digest::blake3(form.name.as_bytes());
        let derivation = FormulationDerivationV2 {
            id: form.derivation.clone(),
            source_model: form.name.clone(),
            method_family: "authored_variational_form".into(),
            choices: Vec::new(),
            introduced_arguments: form
                .arguments
                .iter()
                .map(|argument| argument.id.clone())
                .collect(),
            generated_boundary_terms: Vec::new(),
            assumptions: Vec::new(),
        };
        VariationalArtifactPayloadV2 {
            form,
            spaces,
            frames,
            index_sets: Vec::new(),
            derivation,
            receipt: FormulationReceiptV2 {
                schema: FORMULATION_RECEIPT_V2_SCHEMA.into(),
                source_schema: "authored-test/1".into(),
                source_digest,
                target_semantic_digest: Digest::blake3(&[]),
                relation: "authored".into(),
                producer: "test".into(),
            },
            derivatives: DerivativeArtifactsV2::default(),
            operator_claims: Vec::new(),
            provenance: ArtifactProvenanceV2 {
                producer: "test".into(),
                producer_version: "1".into(),
                parameters: BTreeMap::new(),
            },
            scalar_h1_compatibility: None,
        }
    }
