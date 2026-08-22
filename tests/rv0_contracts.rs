use resolvent::{
    AlgebraBudget, AlgebraError, Bernstein, Expr, FallibleScalar, Interval, Mat, PolyMat, QPoly,
    Rational, Real, Sign, SqrtExt,
};

fn q(value: i64) -> Rational {
    Rational::from_i64(value)
}

#[test]
fn total_ingress_rejects_invalid_values() {
    assert!(Rational::try_from_ratio(1, 0).is_none());
    assert!(q(0).checked_recip().is_none());
    assert!(Interval::try_point(f64::INFINITY).is_none());
    assert!(Interval::try_new(f64::NAN, 1.0).is_none());
    assert!(Interval::try_new(2.0, 1.0).is_none());
    assert!(Interval::new(-2.0, -1.0).try_sqrt().is_none());
    assert!(Sign::try_of_f64(f64::NAN).is_none());
    assert!(<f64 as FallibleScalar>::try_from_f64(f64::INFINITY).is_none());
    assert!(<Real as FallibleScalar>::try_from_f64(f64::NAN).is_none());
    assert!(<f64 as FallibleScalar>::try_from_ratio(1, 0).is_none());
    assert!(resolvent::ladder::try_sign_of_det2_f64(f64::NAN, 1.0, 2.0, 3.0).is_none());
    assert!(SqrtExt::try_new(q(0), q(1), q(-1)).is_err());
    let zero = SqrtExt::rational(q(0));
    assert!(zero.checked_div(&zero).is_none());

    let p = QPoly::from_i64s(&[1, 1]);
    assert!(Bernstein::try_from_power(&p, &q(1), &q(1)).is_err());
}

#[test]
fn malformed_matrix_inputs_fail_closed() {
    assert!(Mat::try_from_rows(&[vec![q(1)], vec![q(2), q(3)]]).is_err());
    assert!(Mat::try_from_cols(2, &[vec![q(1)]]).is_err());

    let a = Mat::zeros(2, 3);
    let b = Mat::zeros(2, 2);
    assert!(a.checked_matmul(&b).is_err());
    assert!(a.checked_add_mat(&b).is_err());
    assert!(a.checked_det().is_err());

    let malformed = Mat {
        rows: 2,
        cols: 2,
        a: vec![q(1)],
    };
    assert!(serde_json::to_string(&malformed).is_err());
}

#[test]
fn negative_power_of_zero_is_a_typed_error() {
    let expression = Expr::integer(0).pow(-1);
    assert_eq!(
        expression.evaluate(&Default::default()),
        Err(AlgebraError::DivisionByZero)
    );
    assert_eq!(q(1).pow(i32::MIN), q(1));
    assert_eq!(q(-1).pow(i32::MIN), q(1));
    assert_eq!(q(0).checked_pow(i32::MIN), None);
    assert_eq!(
        Expr::integer(-1)
            .pow(i32::MIN)
            .evaluate(&Default::default()),
        Ok(q(1))
    );
    assert_eq!(
        Expr::integer(0).pow(i32::MIN).evaluate(&Default::default()),
        Err(AlgebraError::DivisionByZero)
    );
}

#[test]
fn qpoly_and_matrix_wire_vectors_are_schema_owned() {
    let polynomial = QPoly::from_i64s(&[-2, 0, 1]);
    let polynomial_json = serde_json::to_string(&polynomial).unwrap();
    assert_eq!(
        polynomial_json,
        r#"{"schema":"resolvent-qpoly/1","coefficients":[{"numerator":"-2","denominator":"1"},{"numerator":"0","denominator":"1"},{"numerator":"1","denominator":"1"}]}"#
    );
    assert_eq!(
        serde_json::from_str::<QPoly>(&polynomial_json).unwrap(),
        polynomial
    );
    assert_eq!(
        serde_json::to_string(&QPoly::zero_poly()).unwrap(),
        r#"{"schema":"resolvent-qpoly/1","coefficients":[]}"#
    );
    assert_eq!(
        serde_json::to_string(&QPoly::from_i64s(&[1])).unwrap(),
        r#"{"schema":"resolvent-qpoly/1","coefficients":[{"numerator":"1","denominator":"1"}]}"#
    );
    assert!(
        serde_json::from_str::<QPoly>(r#"{"schema":"resolvent-qpoly/2","coefficients":[]}"#)
            .is_err()
    );
    assert!(
        serde_json::from_str::<QPoly>(
            r#"{"schema":"resolvent-qpoly/1","coefficients":[{"numerator":"1","denominator":"1"},{"numerator":"0","denominator":"1"}]}"#
        )
        .is_err(),
        "trailing zero coefficients are a non-canonical second encoding"
    );
    assert!(
        serde_json::from_str::<QPoly>(
            r#"{"schema":"resolvent-qpoly/1","coefficients":[],"extra":0}"#
        )
        .is_err()
    );

    let matrix = Mat::from_rows(&[vec![q(1), q(2)]]);
    let matrix_json = serde_json::to_string(&matrix).unwrap();
    assert_eq!(
        matrix_json,
        r#"{"schema":"resolvent-rational-matrix/1","rows":1,"cols":2,"entries":[{"numerator":"1","denominator":"1"},{"numerator":"2","denominator":"1"}]}"#
    );
    assert_eq!(serde_json::from_str::<Mat>(&matrix_json).unwrap(), matrix);
    assert_eq!(
        serde_json::to_string(&Mat::zeros(0, 0)).unwrap(),
        r#"{"schema":"resolvent-rational-matrix/1","rows":0,"cols":0,"entries":[]}"#
    );
    assert_eq!(
        serde_json::to_string(&Mat::zeros(0, 3)).unwrap(),
        r#"{"schema":"resolvent-rational-matrix/1","rows":0,"cols":3,"entries":[]}"#
    );
    assert!(
        serde_json::from_str::<Mat>(
            r#"{"schema":"resolvent-rational-matrix/1","rows":2,"cols":2,"entries":[]}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Mat>(
            r#"{"schema":"resolvent-rational-matrix/2","rows":0,"cols":0,"entries":[]}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Mat>(
            r#"{"schema":"resolvent-rational-matrix/1","rows":18446744073709551615,"cols":2,"entries":[]}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Mat>(
            r#"{"schema":"resolvent-rational-matrix/1","rows":0,"cols":0,"entries":[],"extra":0}"#
        )
        .is_err()
    );
}

#[test]
fn root_refinement_rejects_hostile_widths_and_exhausts_deterministically() {
    let mut root = resolvent::isolate_roots(&QPoly::from_i64s(&[-2, 0, 1]))
        .unwrap()
        .remove(1);
    assert_eq!(
        root.refine_to_width_with_budget(&q(0), AlgebraBudget::default()),
        Err(AlgebraError::NonPositiveRefinementWidth)
    );
    assert_eq!(
        root.refine_to_width_with_budget(&q(-1), AlgebraBudget::default()),
        Err(AlgebraError::NonPositiveRefinementWidth)
    );

    let before = root.certificate();
    let tiny_width = q(2).pow(-100);
    let no_bisections = AlgebraBudget {
        max_root_bisections: 0,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        root.refine_to_width_with_budget(&tiny_width, no_bisections),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert_eq!(root.certificate(), before);

    let no_arithmetic = AlgebraBudget {
        max_expression_nodes: 0,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        root.refine_to_width_with_budget(&tiny_width, no_arithmetic),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
}

#[test]
fn root_isolation_uses_aggregate_work_and_bisection_budgets() {
    let polynomial = QPoly::from_i64s(&[-2, 0, 1]);
    assert!(matches!(
        resolvent::isolate_roots_with_budget(
            &polynomial,
            AlgebraBudget {
                max_root_bisections: 47,
                ..AlgebraBudget::default()
            }
        ),
        Err(resolvent::RootError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        resolvent::isolate_roots_with_budget(
            &polynomial,
            AlgebraBudget {
                max_expression_nodes: 50,
                ..AlgebraBudget::default()
            }
        ),
        Err(resolvent::RootError::WorkBudgetExceeded { .. })
    ));
}

#[test]
fn root_certificate_decode_is_bounded_and_restoration_validates_mathematics() {
    let roots = resolvent::isolate_roots(&QPoly::from_i64s(&[-2, 0, 1])).unwrap();
    let certificate = roots[1].certificate();
    let json = serde_json::to_string(&certificate).unwrap();
    assert!(serde_json::from_str::<resolvent::RealRootCertificate>(&json).is_ok());

    let wrong_schema = json.replace(
        "resolvent-real-root-certificate/1",
        "resolvent-real-root-certificate/2",
    );
    assert!(serde_json::from_str::<resolvent::RealRootCertificate>(&wrong_schema).is_err());

    let exact = resolvent::RealRoot::exact_rational(Rational::from_ratio(3, 2)).certificate();
    assert_eq!(
        serde_json::to_string(&exact).unwrap(),
        r#"{"schema":"resolvent-real-root-certificate/1","polynomial":{"schema":"resolvent-qpoly/1","coefficients":[{"numerator":"-3","denominator":"2"},{"numerator":"1","denominator":"1"}]},"lower":{"numerator":"3","denominator":"2"},"upper":{"numerator":"3","denominator":"2"},"multiplicity":1}"#
    );
    let too_little_restore_work = AlgebraBudget {
        max_expression_nodes: 3,
        ..AlgebraBudget::default()
    };
    assert!(
        resolvent::RealRoot::from_certificate_with_budget(exact.clone(), too_little_restore_work)
            .is_err(),
        "point-certificate Horner evaluation must consume work"
    );

    let affine_growth = resolvent::RealRootCertificate {
        schema: "resolvent-real-root-certificate/1".into(),
        polynomial: QPoly::from_i64s(&[-2, 0, 1]),
        lower: q(1),
        upper: q(2),
        multiplicity: 1,
    };
    assert!(
        resolvent::RealRoot::from_certificate_with_budget(
            affine_growth,
            AlgebraBudget {
                max_coefficient_bits: 3,
                ..AlgebraBudget::default()
            }
        )
        .is_err(),
        "certificate affine transforms must check intermediate growth"
    );

    let mut mathematically_invalid = certificate;
    mathematically_invalid.lower = q(3);
    mathematically_invalid.upper = q(4);
    let bad_bounds = serde_json::to_string(&mathematically_invalid).unwrap();
    let decoded = serde_json::from_str::<resolvent::RealRootCertificate>(&bad_bounds).unwrap();
    assert!(resolvent::RealRoot::from_certificate(decoded).is_err());

    let oversized = resolvent::RealRootCertificate {
        schema: "resolvent-real-root-certificate/1".into(),
        polynomial: QPoly::new(vec![
            q(1);
            AlgebraBudget::default().max_polynomial_degree + 2
        ]),
        lower: q(0),
        upper: q(1),
        multiplicity: 1,
    };
    let oversized_json = serde_json::to_string(&oversized).unwrap();
    assert!(
        serde_json::from_str::<resolvent::RealRootCertificate>(&oversized_json).is_err(),
        "serde envelope validation must reject before algebraic restoration"
    );
    let excessive_work = resolvent::RealRootCertificate {
        schema: "resolvent-real-root-certificate/1".into(),
        polynomial: QPoly::new(vec![q(1); 49]),
        lower: q(0),
        upper: q(1),
        multiplicity: 1,
    };
    assert!(
        serde_json::from_str::<resolvent::RealRootCertificate>(
            &serde_json::to_string(&excessive_work).unwrap()
        )
        .is_err()
    );
}

#[test]
fn polynomial_and_lazy_work_are_budgeted() {
    let polynomial = QPoly::from_i64s(&[1, 0, 1]);
    let degree_budget = AlgebraBudget {
        max_polynomial_degree: 1,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        polynomial.resultant(&polynomial, degree_budget),
        Err(AlgebraError::PolynomialDegree { .. })
    ));
    assert!(matches!(
        polynomial.mul_poly_with_budget(&polynomial, degree_budget),
        Err(AlgebraError::PolynomialDegree { .. })
    ));
    let tight_bits = AlgebraBudget {
        max_coefficient_bits: 3,
        ..AlgebraBudget::default()
    };
    let two_x_plus_two = QPoly::from_i64s(&[2, 2]);
    assert!(matches!(
        two_x_plus_two.mul_poly_with_budget(&two_x_plus_two, tight_bits),
        Err(AlgebraError::CoefficientBits { .. })
    ));
    let no_work = AlgebraBudget {
        max_expression_nodes: 0,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        polynomial.gcd_with_budget(&polynomial, no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        polynomial.div_rem_with_budget(&QPoly::from_i64s(&[1, 1]), no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));

    let matrix = Mat::from_rows(&[vec![q(1), q(2)], vec![q(3), q(4)]]);
    let one_dimension = AlgebraBudget {
        max_matrix_dimension: 1,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        matrix.checked_det_with_budget(one_dimension),
        Err(AlgebraError::MatrixDimension { .. })
    ));
    assert!(matches!(
        matrix.rref_with_budget(one_dimension),
        Err(AlgebraError::MatrixDimension { .. })
    ));
    let growth_matrix = Mat::from_rows(&[vec![q(2), q(2)], vec![q(2), q(-2)]]);
    assert!(matches!(
        growth_matrix.checked_det_with_budget(tight_bits),
        Err(AlgebraError::CoefficientBits { .. })
    ));
    let fractional_matrix = Mat::from_rows(&[vec![q(2), q(1)], vec![q(1), q(2)]]);
    assert!(matches!(
        fractional_matrix.rref_with_budget(tight_bits),
        Err(AlgebraError::CoefficientBits { .. })
    ));
    assert!(matches!(
        matrix.checked_matmul_with_budget(&matrix, no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        matrix.checked_det_with_budget(no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        matrix.rref_with_budget(no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        Mat::zeros(2, 2).checked_det_with_budget(no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        Mat::zeros(2, 2).rref_with_budget(no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));

    let polynomial_matrix = PolyMat::pencil(&Mat::ident(2), &Mat::ident(2));
    assert!(matches!(
        polynomial_matrix.det_with_budget(no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert!(matches!(
        resolvent::polymat::combinations_with_budget(8, 4, no_work),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    let division_budget = AlgebraBudget {
        max_expression_nodes: 3,
        ..AlgebraBudget::default()
    };
    let one_by_one = PolyMat::pencil(&Mat::ident(1), &Mat::ident(1));
    assert!(matches!(
        one_by_one.invariant_factors_with_budget(division_budget),
        Err(AlgebraError::BudgetExceeded { .. })
    ));

    let one = Real::from_exact(q(1));
    let two = &one + &one;
    let zero_budget = AlgebraBudget {
        max_lazy_nodes: 0,
        ..AlgebraBudget::default()
    };
    assert!(matches!(
        two.exact_with_budget(zero_budget),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
    assert_eq!(
        two.exact_with_budget(AlgebraBudget::default()).unwrap(),
        &q(2)
    );
}

#[test]
fn polymat_invariants_share_one_aggregate_work_meter() {
    let matrix = PolyMat::pencil(&Mat::ident(2), &Mat::ident(2));
    let budget = AlgebraBudget {
        max_expression_nodes: 17,
        ..AlgebraBudget::default()
    };
    assert!(matrix.determinantal_divisor_with_budget(1, budget).is_ok());
    assert!(matrix.determinantal_divisor_with_budget(2, budget).is_ok());
    assert!(matches!(
        matrix.invariant_factors_with_budget(budget),
        Err(AlgebraError::BudgetExceeded { .. })
    ));
}
