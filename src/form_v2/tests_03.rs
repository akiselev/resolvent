    #[test]
    fn objective_jacobian_and_mixed_block_arities_are_explicit() {
        let (frame, space) = scalar_space();
        let derivation = FormulationDerivationIdV2::new("d");
        let objective = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "objective".into(),
            arity: 0,
            arguments: Vec::new(),
            coefficients: Vec::new(),
            constants: Vec::new(),
            integrals: vec![IntegralV2 {
                id: "objective".into(),
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: RegionSelectorV2::All,
                },
                integrand: FormExprV2::ScientificScalar {
                    expression: Expr::Number {
                        value: 1.0,
                        unit: None,
                    },
                },
                label: None,
            }],
            scalar_kind: ScalarKindV2::Real64,
            derivation: derivation.clone(),
            obligations: Vec::new(),
        };
        let objective = VariationalFormArtifactV2::build(authored_payload(
            objective,
            vec![space.clone()],
            vec![frame.clone()],
        ))
        .unwrap();
        assert_eq!(objective.payload.form.arity(), 0);

        let arguments = vec![
            FormArgumentV2 {
                id: ArgumentIdV2::new("test0"),
                name: "v0".into(),
                number: 0,
                part: Some(0),
                space: space.id.clone(),
                value_type: space.value_type.clone(),
            },
            FormArgumentV2 {
                id: ArgumentIdV2::new("test1"),
                name: "v1".into(),
                number: 0,
                part: Some(1),
                space: space.id.clone(),
                value_type: space.value_type.clone(),
            },
            FormArgumentV2 {
                id: ArgumentIdV2::new("trial0"),
                name: "du0".into(),
                number: 1,
                part: Some(0),
                space: space.id.clone(),
                value_type: space.value_type.clone(),
            },
            FormArgumentV2 {
                id: ArgumentIdV2::new("trial1"),
                name: "du1".into(),
                number: 1,
                part: Some(1),
                space: space.id.clone(),
                value_type: space.value_type.clone(),
            },
        ];
        let integrals = [("b00", "test0", "trial0"), ("b11", "test1", "trial1")]
            .into_iter()
            .map(|(id, test, trial)| IntegralV2 {
                id: id.into(),
                measure: MeasureV2::Cell {
                    domain: "Omega".into(),
                    region: RegionSelectorV2::All,
                },
                integrand: FormExprV2::Product {
                    values: vec![
                        FormExprV2::Argument {
                            id: ArgumentIdV2::new(test),
                        },
                        FormExprV2::Argument {
                            id: ArgumentIdV2::new(trial),
                        },
                    ],
                },
                label: None,
            })
            .collect();
        let jacobian = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "jacobian".into(),
            arity: 2,
            arguments,
            coefficients: Vec::new(),
            constants: Vec::new(),
            integrals,
            scalar_kind: ScalarKindV2::Real64,
            derivation,
            obligations: Vec::new(),
        };
        assert_eq!(jacobian.arity(), 2);
        let block = jacobian
            .extract_block(&BTreeMap::from([(0, 1), (1, 1)]))
            .unwrap();
        assert_eq!(block.arguments.len(), 2);
        assert_eq!(block.integrals.len(), 1);
        assert_eq!(block.integrals[0].id, "b11");
    }
