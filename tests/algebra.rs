use resolvent::{AlgebraBudget, AlgebraOperation, AlgebraReceipt, Expr, QPoly, Rational, Sign};
use std::collections::BTreeMap;

fn q(value: i64) -> Rational {
    Rational::from_i64(value)
}

#[test]
fn canonicalization_is_order_independent_and_budgeted() {
    let x = Expr::symbol("x");
    let left = Expr::add([x.clone(), Expr::integer(2), Expr::integer(3)]);
    let right = Expr::add([Expr::integer(3), Expr::integer(2), x]);
    assert_eq!(
        left.canonicalize(AlgebraBudget::default()).unwrap(),
        right.canonicalize(AlgebraBudget::default()).unwrap()
    );
    assert!(
        left.canonicalize(AlgebraBudget {
            max_expression_nodes: 1,
            ..AlgebraBudget::default()
        })
        .is_err()
    );
}

#[test]
fn differentiation_and_sign_serve_distinct_consumers() {
    let x = Expr::symbol("x");
    let expression = Expr::add([
        Expr::mul([Expr::integer(3), x.clone().pow(2)]),
        Expr::function("sin", [x.clone()]),
    ]);
    let derivative = expression
        .differentiate("x", AlgebraBudget::default())
        .unwrap();
    let expected = Expr::add([
        Expr::mul([Expr::integer(6), x]),
        Expr::function("cos", [Expr::symbol("x")]),
    ])
    .canonicalize(AlgebraBudget::default())
    .unwrap();
    assert_eq!(derivative, expected);

    let affine = Expr::add([Expr::symbol("clearance"), Expr::integer(-2)])
        .canonicalize(AlgebraBudget::default())
        .unwrap();
    let environment = BTreeMap::from([("clearance".into(), q(3))]);
    assert_eq!(affine.exact_sign(&environment).unwrap(), Sign::Positive);

    let receipt =
        AlgebraReceipt::for_expressions(AlgebraOperation::Differentiate, &expression, &derivative)
            .unwrap();
    assert_eq!(receipt.schema, "resolvent-algebra-receipt/1");
}

#[test]
fn polynomial_resultant_and_root_isolation_are_exact() {
    let x2_minus_2 = QPoly::new(vec![q(-2), q(0), q(1)]);
    let x_minus_1 = QPoly::new(vec![q(-1), q(1)]);
    assert_eq!(
        x2_minus_2
            .resultant(&x_minus_1, AlgebraBudget::default())
            .unwrap(),
        q(-1)
    );
    let mut roots = resolvent::isolate_roots(&x2_minus_2).unwrap();
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0].cmp_rational(&q(0)), std::cmp::Ordering::Less);
    assert_eq!(roots[1].cmp_rational(&q(0)), std::cmp::Ordering::Greater);
}

#[test]
fn polynomial_receipts_are_deterministic_and_identity_sensitive() {
    let polynomial = QPoly::new(vec![q(-2), q(0), q(1)]);
    let output = q(-1);
    let receipt =
        AlgebraReceipt::for_polynomial(AlgebraOperation::Resultant, &polynomial, &output).unwrap();
    let repeated =
        AlgebraReceipt::for_polynomial(AlgebraOperation::Resultant, &polynomial, &output).unwrap();
    assert_eq!(receipt, repeated);
    assert_eq!(receipt.schema, "resolvent-algebra-receipt/1");
    assert_eq!(receipt.operation, AlgebraOperation::Resultant);

    let changed_input = AlgebraReceipt::for_polynomial(
        AlgebraOperation::Resultant,
        &QPoly::new(vec![q(-3), q(0), q(1)]),
        &output,
    )
    .unwrap();
    assert_ne!(receipt.input_digest, changed_input.input_digest);

    let changed_output =
        AlgebraReceipt::for_polynomial(AlgebraOperation::Resultant, &polynomial, &q(1)).unwrap();
    assert_ne!(receipt.output_digest, changed_output.output_digest);
}

#[test]
fn repeated_roots_are_reported_once() {
    let x_minus_1 = QPoly::new(vec![q(-1), q(1)]);
    let repeated = x_minus_1.mul_poly(&x_minus_1);
    let mut roots = resolvent::isolate_roots(&repeated).unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].cmp_rational(&q(1)), std::cmp::Ordering::Equal);
}

#[test]
fn rational_wire_encoding_is_explicit_and_canonical() {
    let value = Rational::from_ratio(-6, 14);
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"numerator":"-3","denominator":"7"}"#);
    assert_eq!(serde_json::from_str::<Rational>(&json).unwrap(), value);
    assert!(serde_json::from_str::<Rational>(r#"{"numerator":"1","denominator":"0"}"#).is_err());
}

#[test]
fn real_root_certificates_round_trip_and_validate() {
    let polynomial = QPoly::new(vec![q(-2), q(0), q(1)]);
    let roots =
        resolvent::isolate_roots_with_budget(&polynomial, AlgebraBudget::default()).unwrap();
    let certificate = roots[1].certificate();
    let json = serde_json::to_string(&certificate).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    let mut restored = resolvent::RealRoot::from_certificate(decoded).unwrap();
    assert_eq!(restored.cmp_rational(&q(0)), std::cmp::Ordering::Greater);

    let mut invalid = certificate;
    invalid.lower = q(3);
    invalid.upper = q(4);
    assert!(resolvent::RealRoot::from_certificate(invalid).is_err());

    let boundary_root = resolvent::RealRootCertificate {
        polynomial: QPoly::new(vec![q(0), q(1)]),
        lower: q(0),
        upper: q(1),
        multiplicity: 1,
    };
    assert!(resolvent::RealRoot::from_certificate(boundary_root).is_err());
}

#[test]
fn root_isolation_reports_budget_exhaustion() {
    let polynomial = QPoly::new(vec![q(-2), q(0), q(1)]);
    let budget = AlgebraBudget {
        max_root_bisections: 0,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        resolvent::isolate_roots_with_budget(&polynomial, budget),
        Err(resolvent::RootError::BudgetExceeded { limit: 0 })
    ));
}
