use resolvent::{
    AlgebraBudget, AlgebraOperation, AlgebraReceipt, Expr, Polynomial, Rational, Sign,
};
use std::collections::BTreeMap;

fn q(value: i64) -> Rational {
    Rational::from_integer(value.into())
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
    let x2_minus_2 = Polynomial::new(vec![q(-2), q(0), q(1)]);
    let x_minus_1 = Polynomial::new(vec![q(-1), q(1)]);
    assert_eq!(
        x2_minus_2
            .resultant(&x_minus_1, AlgebraBudget::default())
            .unwrap(),
        q(-1)
    );
    let intervals = x2_minus_2
        .isolate_real_roots(AlgebraBudget::default())
        .unwrap();
    assert_eq!(intervals.len(), 2);
    assert!(intervals[0].upper <= q(0));
    assert!(intervals[1].lower >= q(0));
}

#[test]
fn repeated_roots_are_reported_once() {
    let x_minus_1 = Polynomial::new(vec![q(-1), q(1)]);
    let repeated = x_minus_1.mul(&x_minus_1);
    let intervals = repeated
        .isolate_real_roots(AlgebraBudget::default())
        .unwrap();
    assert_eq!(intervals.len(), 1);
    assert!(intervals[0].lower <= q(1) && intervals[0].upper >= q(1));
}
