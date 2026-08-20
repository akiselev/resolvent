    #[test]
    fn scalar_adapter_separates_fields_from_arguments_and_roundtrips() {
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
        let module = crate::scientific::parse_scientific_module(source).unwrap();
        let artifact = adapt_scalar_h1_model_v2(&module.models[0]).unwrap();
        assert_eq!(artifact.payload.form.arity(), 1);
        assert_eq!(artifact.payload.form.arguments.len(), 1);
        assert_eq!(artifact.payload.form.coefficients.len(), 1);
        assert_ne!(
            artifact.payload.form.arguments[0].id.0,
            artifact.payload.form.coefficients[0].field.0
        );
        assert!(artifact.payload.operator_claims.is_empty());
        assert_eq!(
            artifact.payload.derivatives.jvp,
            DerivativeArtifactStatusV2::NotGenerated
        );
        assert_eq!(
            artifact.scalar_h1_compatibility_program().unwrap(),
            &crate::lower_scalar_h1_model(&module.models[0]).unwrap()
        );
        let json = artifact.to_pretty_json().unwrap();
        let roundtrip = VariationalFormArtifactV2::from_json(&json).unwrap();
        assert_eq!(roundtrip, artifact);
    }

    #[test]
    fn canonical_digest_ignores_argument_and_integral_construction_order() {
        let (frame, space) = scalar_space();
        let derivation = FormulationDerivationIdV2::new("d");
        let argument = |name: &str, part: u16| FormArgumentV2 {
            id: ArgumentIdV2::new(name),
            name: name.into(),
            number: 0,
            part: Some(part),
            space: space.id.clone(),
            value_type: space.value_type.clone(),
        };
        let integral = |id: &str, argument: &str| IntegralV2 {
            id: id.into(),
            measure: MeasureV2::Cell {
                domain: "Omega".into(),
                region: RegionSelectorV2::All,
            },
            integrand: FormExprV2::Argument {
                id: ArgumentIdV2::new(argument),
            },
            label: None,
        };
        let form = |reverse: bool| {
            let mut arguments = vec![argument("v0", 0), argument("v1", 1)];
            let mut integrals = vec![integral("i0", "v0"), integral("i1", "v1")];
            if reverse {
                arguments.reverse();
                integrals.reverse();
            }
            VariationalFormV2 {
                schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
                name: "mixed".into(),
                arity: 1,
                arguments,
                coefficients: Vec::new(),
                constants: Vec::new(),
                integrals,
                scalar_kind: ScalarKindV2::Real64,
                derivation: derivation.clone(),
                obligations: Vec::new(),
            }
        };
        let left = VariationalFormArtifactV2::build(authored_payload(
            form(false),
            vec![space.clone()],
            vec![frame.clone()],
        ))
        .unwrap();
        let right = VariationalFormArtifactV2::build(authored_payload(
            form(true),
            vec![space],
            vec![frame],
        ))
        .unwrap();
        assert_eq!(left.semantic_digest, right.semantic_digest);
        assert_eq!(left.artifact_digest, right.artifact_digest);
    }
