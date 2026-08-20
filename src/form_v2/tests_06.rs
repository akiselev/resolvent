    #[test]
    fn generated_derivative_claims_require_evidence() {
        let (frame, space) = scalar_space();
        let form = VariationalFormV2 {
            schema: VARIATIONAL_FORM_V2_SCHEMA.into(),
            name: "claim".into(),
            arity: 0,
            arguments: Vec::new(),
            coefficients: Vec::new(),
            constants: Vec::new(),
            integrals: Vec::new(),
            scalar_kind: ScalarKindV2::Real64,
            derivation: FormulationDerivationIdV2::new("d"),
            obligations: Vec::new(),
        };
        let mut payload = authored_payload(form, vec![space], vec![frame]);
        payload.derivatives.jvp = DerivativeArtifactStatusV2::Generated {
            artifact: Digest::blake3(b"jvp"),
            evidence: Vec::new(),
        };
        let error = VariationalFormArtifactV2::build(payload).unwrap_err();
        assert_eq!(error.code(), "DERIV-EVIDENCE-001");
    }

    #[test]
    fn unsupported_differential_terms_are_structured_failures() {
        let source = r#"
module test.transport;
model Transport {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field c: state scalar H1(order=1) on Omega { time_role = differential; };
  field velocity: coefficient scalar H1(order=1) on Omega;
  equation transport on Omega { dt(c) + div(velocity * c) = 0; }
}
"#;
        let module = crate::scientific::parse_scientific_module(source).unwrap();
        let error = adapt_scalar_h1_model_v2(&module.models[0]).unwrap_err();
        assert_eq!(error.code(), "FORM-LOWER-001");
    }
