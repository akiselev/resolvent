    #[test]
    fn invalid_frames_and_missing_facet_sides_are_rejected() {
        let frame_a = FrameV2 {
            id: FrameIdV2::new("frame::A"),
            dimension: 2,
        };
        let frame_b = FrameV2 {
            id: FrameIdV2::new("frame::B"),
            dimension: 2,
        };
        let vector_a = TensorTypeV2::spatial_vector(
            ScalarKindV2::Real64,
            frame_a.id.clone(),
            VarianceV2::Contravariant,
        );
        let vector_b = TensorTypeV2::spatial_vector(
            ScalarKindV2::Real64,
            frame_b.id.clone(),
            VarianceV2::Contravariant,
        );
        let spaces = vec![
            SpaceRequirementV2 {
                id: SpaceRequirementIdV2::new("sa"),
                domain: "A".into(),
                spatial_frame: frame_a.id.clone(),
                sobolev: SobolevSpaceV2::H1,
                value_type: vector_a.clone(),
            },
            SpaceRequirementV2 {
                id: SpaceRequirementIdV2::new("sb"),
                domain: "B".into(),
                spatial_frame: frame_b.id.clone(),
                sobolev: SobolevSpaceV2::H1,
                value_type: vector_b.clone(),
            },
        ];
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "bad-frame".into(),
            arity: 1,
            arguments: vec![
                FormArgumentV2 {
                    id: ArgumentIdV2::new("a"),
                    name: "a".into(),
                    number: 0,
                    part: Some(0),
                    space: spaces[0].id.clone(),
                    value_type: vector_a,
                },
                FormArgumentV2 {
                    id: ArgumentIdV2::new("b"),
                    name: "b".into(),
                    number: 0,
                    part: Some(1),
                    space: spaces[1].id.clone(),
                    value_type: vector_b,
                },
            ],
            coefficients: Vec::new(),
            constants: Vec::new(),
            integrals: vec![IntegralV2 {
                id: "i".into(),
                measure: MeasureV2::Cell {
                    domain: "A".into(),
                    region: RegionSelectorV2::All,
                },
                integrand: FormExprV2::Inner {
                    left: Box::new(FormExprV2::Argument {
                        id: ArgumentIdV2::new("a"),
                    }),
                    right: Box::new(FormExprV2::Argument {
                        id: ArgumentIdV2::new("b"),
                    }),
                },
                label: None,
            }],
            scalar_kind: ScalarKindV2::Real64,
            derivation: FormulationDerivationIdV2::new("d"),
            obligations: Vec::new(),
        };
        let error = VariationalFormArtifactV2::build(authored_payload(
            form,
            spaces,
            vec![frame_a, frame_b],
        ))
        .unwrap_err();
        assert!(matches!(error, FormV2Error::InvalidContraction { .. }));

        let (frame, space) = scalar_space();
        let facet = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "facet".into(),
            arity: 1,
            arguments: vec![FormArgumentV2 {
                id: ArgumentIdV2::new("v"),
                name: "v".into(),
                number: 0,
                part: None,
                space: space.id.clone(),
                value_type: space.value_type.clone(),
            }],
            coefficients: Vec::new(),
            constants: Vec::new(),
            integrals: vec![IntegralV2 {
                id: "facet".into(),
                measure: MeasureV2::InteriorFacet {
                    domain: "Omega".into(),
                    region: RegionSelectorV2::All,
                },
                integrand: FormExprV2::Argument {
                    id: ArgumentIdV2::new("v"),
                },
                label: None,
            }],
            scalar_kind: ScalarKindV2::Real64,
            derivation: FormulationDerivationIdV2::new("d"),
            obligations: Vec::new(),
        };
        let error = VariationalFormArtifactV2::build(authored_payload(
            facet,
            vec![space],
            vec![frame],
        ))
        .unwrap_err();
        assert_eq!(error.code(), "FORM-SIDE-002");
    }
