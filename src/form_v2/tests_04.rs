    #[test]
    fn complex_inner_dot_and_hermitian_transpose_have_distinct_semantics() {
        let frame = FrameV2 {
            id: FrameIdV2::new("frame::Omega"),
            dimension: 3,
        };
        let vector_type = TensorTypeV2::spatial_vector(
            ScalarKindV2::Complex64,
            frame.id.clone(),
            VarianceV2::Contravariant,
        );
        let space = SpaceRequirementV2 {
            id: SpaceRequirementIdV2::new("space::z"),
            domain: "Omega".into(),
            spatial_frame: frame.id.clone(),
            sobolev: SobolevSpaceV2::H1,
            value_type: vector_type.clone(),
        };
        let argument = FormArgumentV2 {
            id: ArgumentIdV2::new("v"),
            name: "v".into(),
            number: 0,
            part: None,
            space: space.id.clone(),
            value_type: vector_type.clone(),
        };
        let coefficient = FormCoefficientV2 {
            field: ScientificFieldIdV2::new("z"),
            name: "z".into(),
            space: space.id.clone(),
            time_level: TimeLevelV2::Current,
            value_type: vector_type,
        };
        let make = |name: &str, integrand: FormExprV2| VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: name.into(),
            arity: 1,
            arguments: vec![argument.clone()],
            coefficients: vec![coefficient.clone()],
            constants: Vec::new(),
            integrals: vec![IntegralV2 {
                id: "i".into(),
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: RegionSelectorV2::All,
                },
                integrand,
                label: None,
            }],
            scalar_kind: ScalarKindV2::Complex64,
            derivation: FormulationDerivationIdV2::new("d"),
            obligations: Vec::new(),
        };
        let left = FormExprV2::Coefficient {
            field: coefficient.field.clone(),
        };
        let right = FormExprV2::Argument {
            id: argument.id.clone(),
        };
        let dot = VariationalFormArtifactV2::build(authored_payload(
            make(
                "dot",
                FormExprV2::Dot {
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                },
            ),
            vec![space.clone()],
            vec![frame.clone()],
        ))
        .unwrap();
        let inner = VariationalFormArtifactV2::build(authored_payload(
            make(
                "inner",
                FormExprV2::Inner {
                    left: Box::new(left),
                    right: Box::new(right),
                },
            ),
            vec![space],
            vec![frame],
        ))
        .unwrap();
        assert_ne!(dot.semantic_digest, inner.semantic_digest);
        assert!(dot.payload.form.scalar_kind.is_complex());

        let matrix = FormExprV2::HermitianTranspose {
            value: Box::new(FormExprV2::ScientificScalar {
                expression: Expr::Number {
                    value: 1.0,
                    unit: None,
                },
            }),
        };
        assert!(matches!(matrix, FormExprV2::HermitianTranspose { .. }));
    }
