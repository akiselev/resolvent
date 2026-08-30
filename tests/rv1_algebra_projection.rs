//! RV1-C2/C3: exact literal ingress and the term-level algebra-operation
//! surface (`AlgebraOperation::{Differentiate,Simplify,Substitute}`).

use resolvent::{
    AlgebraBudget, Atom, OPERATOR_NAMESPACE, PiecewiseCase, Rational, RelationOperator, SymbolName,
    TermAlgebraError, TermAlgebraOperation, TermAlgebraRefusal, TermBudget, TermNode, TermStore,
};

fn budget() -> TermBudget {
    TermBudget::default()
}

fn algebra_budget() -> AlgebraBudget {
    AlgebraBudget::default()
}

fn symbol(store: &mut TermStore, namespace: &str, name: &str) -> resolvent::TermId {
    store
        .atom(Atom::Symbol(SymbolName::new(namespace, name)), budget())
        .unwrap()
}

fn operator(store: &mut TermStore, name: &str) -> resolvent::TermId {
    store
        .atom(
            Atom::Symbol(SymbolName::new(OPERATOR_NAMESPACE, name)),
            budget(),
        )
        .unwrap()
}

fn apply(
    store: &mut TermStore,
    head_name: &str,
    arguments: Vec<resolvent::TermId>,
) -> resolvent::TermId {
    let head = operator(store, head_name);
    store
        .intern(TermNode::Apply { head, arguments }, budget())
        .unwrap()
}

#[test]
fn exact_literal_atom_kinds_are_lexically_distinct_but_mathematically_equal() {
    let mut store = TermStore::new().unwrap();
    let five_i128 = store.exact_integer(5, budget()).unwrap();
    let five_big = store.exact_big_integer("5", budget()).unwrap();
    let five_rational = store
        .exact_rational(Rational::from_i128(5), budget())
        .unwrap();
    let five_decimal = store.exact_decimal(5, 0, budget()).unwrap();
    let five_decimal_lexeme = store.exact_decimal_lexeme("5.00", budget()).unwrap();

    // Integer text ingress is canonical, so `exact_integer` and
    // `exact_big_integer` hash-cons to the identical atom.
    assert_eq!(five_i128, five_big);
    // Trailing fractional zeros normalize away identically for `from_parts`
    // and lexeme parsing.
    assert_eq!(five_decimal, five_decimal_lexeme);

    // But an integer, a rational, and a decimal spelling of the same value
    // are structurally distinct Term atoms (different canonical bytes/digests).
    assert_ne!(five_i128, five_rational);
    assert_ne!(five_i128, five_decimal);
    assert_ne!(five_rational, five_decimal);
    let digests =
        [five_i128, five_rational, five_decimal].map(|id| store.digest(id, budget()).unwrap());
    assert_ne!(digests[0], digests[1]);
    assert_ne!(digests[0], digests[2]);
    assert_ne!(digests[1], digests[2]);

    // Yet every one of them has the identical exact rational value.
    let expected = Rational::from_i128(5);
    for id in [
        five_i128,
        five_big,
        five_rational,
        five_decimal,
        five_decimal_lexeme,
    ] {
        assert_eq!(
            store.exact_rational_value(id).unwrap(),
            Some(expected.clone())
        );
    }
}

#[test]
fn decimal_lexeme_point_one_is_exactly_one_tenth_never_via_f64() {
    let mut store = TermStore::new().unwrap();
    let from_parts = store.exact_decimal(1, -1, budget()).unwrap();
    let from_lexeme = store.exact_decimal_lexeme("0.1", budget()).unwrap();
    let from_trailing_zero = store.exact_decimal_lexeme("0.10", budget()).unwrap();

    assert_eq!(from_parts, from_lexeme);
    assert_eq!(from_lexeme, from_trailing_zero);

    let value = store.exact_rational_value(from_lexeme).unwrap().unwrap();
    assert_eq!(value, Rational::from_ratio(1, 10));
    // The nearest f64 to 0.1 is a different (much larger-denominator) rational.
    assert_ne!(value, Rational::from_f64(0.1).unwrap());
}

#[test]
fn exact_decimal_ingress_fails_closed_on_malformed_lexemes() {
    for text in ["", "abc", "1.2.3", "1..2", "1e", "1e1e1"] {
        assert!(
            Atom::exact_decimal_from_lexeme(text).is_err(),
            "expected {text:?} to be rejected"
        );
    }
    assert!(Atom::integer("007").is_err());
    assert!(Atom::integer("").is_err());
}

#[test]
fn decimal_point_one_differentiates_and_simplifies_as_exactly_one_tenth() {
    let mut store = TermStore::new().unwrap();
    let literal = store.exact_decimal_lexeme("0.1", budget()).unwrap();
    let x = SymbolName::new("Global`", "x");

    let simplified = store
        .apply_algebra_operation(
            literal,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(
        store.exact_rational_value(simplified.root).unwrap(),
        Some(Rational::from_ratio(1, 10))
    );

    let derivative = store
        .apply_algebra_operation(
            literal,
            &TermAlgebraOperation::Differentiate { variable: x },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(
        store.exact_rational_value(derivative.root).unwrap(),
        Some(Rational::zero())
    );
    assert!(derivative.assumptions.is_empty());
    assert_eq!(
        derivative.receipt.schema,
        "resolvent-term-algebra-receipt/1"
    );
}

#[test]
fn differentiate_power_rule_matches_substitution_and_simplification() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let two = store.exact_integer(2, budget()).unwrap();
    let x_squared = apply(&mut store, "Pow", vec![x, two]);
    let two_x_squared = apply(&mut store, "Mul", vec![two, x_squared]);

    let variable = SymbolName::new("Global`", "x");
    let derivative = store
        .apply_algebra_operation(
            two_x_squared,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();

    let five = store.exact_integer(5, budget()).unwrap();
    let substituted = store
        .apply_algebra_operation(
            derivative.root,
            &TermAlgebraOperation::Substitute {
                bindings: vec![(variable, five)],
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    let simplified = store
        .apply_algebra_operation(
            substituted.root,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    // d/dx(2x^2) = 4x (exponent 2-1=1 collapses to the bare base under
    // canonicalize); at x=5, 4*5 = 20.
    assert_eq!(
        store.exact_rational_value(simplified.root).unwrap(),
        Some(Rational::from_i128(20))
    );
}

#[test]
fn simplify_does_not_evaluate_power_of_a_literal_base() {
    // `Expr::canonicalize` (RV0, unchanged here) constant-folds `Add`/`Mul`
    // chains but does not evaluate `Pow` of a literal base into a number.
    // `Simplify` inherits that behavior via its lossless `Expr` round trip,
    // so a substituted `5^2` stays a structural `Pow` node rather than
    // becoming the atom `25`. This is a documented scope boundary, not a
    // missed simplification.
    let mut store = TermStore::new().unwrap();
    let five = store.exact_integer(5, budget()).unwrap();
    let two = store.exact_integer(2, budget()).unwrap();
    let five_squared = apply(&mut store, "Pow", vec![five, two]);

    let simplified = store
        .apply_algebra_operation(
            five_squared,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    assert_eq!(store.exact_rational_value(simplified.root).unwrap(), None);
    match store.node(simplified.root).unwrap() {
        TermNode::Apply { arguments, .. } => {
            assert_eq!(arguments.len(), 2);
            // Values survive exactly; the `Expr` round trip re-embeds every
            // exact literal as `Atom::Rational`, so the operand atoms need
            // not be the *same* TermIds as the originals (which were plain
            // `Integer` atoms) even though their values are unchanged.
            assert_eq!(
                store.exact_rational_value(arguments[0]).unwrap(),
                Some(Rational::from_i128(5))
            );
            assert_eq!(
                store.exact_rational_value(arguments[1]).unwrap(),
                Some(Rational::from_i128(2))
            );
        }
        other => panic!("expected the Pow node to be preserved, got {other:?}"),
    }
}

#[test]
fn differentiate_refuses_relations_and_unrecognized_apply_heads() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let variable = SymbolName::new("Global`", "x");

    let relation = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::Equal,
                left: x,
                right: x,
            },
            budget(),
        )
        .unwrap();
    let outcome = store.apply_algebra_operation(
        relation,
        &TermAlgebraOperation::Differentiate {
            variable: variable.clone(),
        },
        budget(),
        algebra_budget(),
    );
    match outcome {
        Err(TermAlgebraError::Refused(refusal)) => {
            assert_eq!(refusal.code(), "ALGEBRA_OUTSIDE_OPERATION");
            assert!(matches!(
                refusal,
                TermAlgebraRefusal::OutsideOperation { ref construct } if construct == "Relation"
            ));
        }
        other => panic!("expected a refusal, got {other:?}"),
    }

    // An `Apply` head outside the reserved operator namespace is likewise
    // refused rather than silently treated as opaque/zero.
    let mystery_head = symbol(&mut store, "Global`", "MysteryFunction");
    let mystery_call = store
        .intern(
            TermNode::Apply {
                head: mystery_head,
                arguments: vec![x],
            },
            budget(),
        )
        .unwrap();
    let outcome = store.apply_algebra_operation(
        mystery_call,
        &TermAlgebraOperation::Differentiate { variable },
        budget(),
        algebra_budget(),
    );
    assert!(matches!(outcome, Err(TermAlgebraError::Refused(_))));
}

#[test]
fn simplify_preserves_relations_structurally_while_simplifying_scalar_sides() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let x_plus_zero = apply(&mut store, "Add", vec![x, zero]);
    let relation = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::Equal,
                left: x_plus_zero,
                right: y,
            },
            budget(),
        )
        .unwrap();

    let outcome = store
        .apply_algebra_operation(
            relation,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    match store.node(outcome.root).unwrap() {
        TermNode::Relation {
            operator,
            left,
            right,
        } => {
            assert_eq!(operator, RelationOperator::Equal);
            // `x + 0` simplifies down to the bare symbol `x`.
            assert_eq!(left, x);
            assert_eq!(right, y);
        }
        other => panic!("expected the Relation wrapper to be preserved, got {other:?}"),
    }
}

#[test]
fn differentiate_piecewise_branchwise_when_condition_is_variable_independent() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let two = store.exact_integer(2, budget()).unwrap();

    let condition = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::GreaterEqual,
                left: y,
                right: zero,
            },
            budget(),
        )
        .unwrap();
    let case_value = apply(&mut store, "Pow", vec![x, two]);
    let otherwise = apply(&mut store, "Mul", vec![two, x]);
    let piecewise = store
        .intern(
            TermNode::Piecewise {
                cases: vec![PiecewiseCase {
                    value: case_value,
                    condition,
                }],
                otherwise: Some(otherwise),
            },
            budget(),
        )
        .unwrap();

    let variable = SymbolName::new("Global`", "x");
    let derivative = store
        .apply_algebra_operation(
            piecewise,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert!(!derivative.assumptions.is_empty());

    let (new_cases, new_otherwise) = match store.node(derivative.root).unwrap() {
        TermNode::Piecewise { cases, otherwise } => (cases, otherwise),
        other => panic!("expected a Piecewise result, got {other:?}"),
    };
    assert_eq!(new_cases.len(), 1);
    // The condition is untouched: differentiation never rewrites it.
    assert_eq!(new_cases[0].condition, condition);
    assert!(new_otherwise.is_some());

    // d/dx(x^2) at x=7 is 2*7=14; d/dx(2x) at x=7 is 2.
    let seven = store.exact_integer(7, budget()).unwrap();
    for (branch, expected) in [
        (new_cases[0].value, 14i128),
        (new_otherwise.unwrap(), 2i128),
    ] {
        let substituted = store
            .apply_algebra_operation(
                branch,
                &TermAlgebraOperation::Substitute {
                    bindings: vec![(variable.clone(), seven)],
                },
                budget(),
                algebra_budget(),
            )
            .unwrap();
        let simplified = store
            .apply_algebra_operation(
                substituted.root,
                &TermAlgebraOperation::Simplify,
                budget(),
                algebra_budget(),
            )
            .unwrap();
        assert_eq!(
            store.exact_rational_value(simplified.root).unwrap(),
            Some(Rational::from_i128(expected))
        );
    }
}

#[test]
fn differentiate_piecewise_refuses_a_variable_dependent_condition() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let zero = store.exact_integer(0, budget()).unwrap();
    let two = store.exact_integer(2, budget()).unwrap();

    let condition = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::GreaterEqual,
                left: x,
                right: zero,
            },
            budget(),
        )
        .unwrap();
    let case_value = apply(&mut store, "Pow", vec![x, two]);
    let piecewise = store
        .intern(
            TermNode::Piecewise {
                cases: vec![PiecewiseCase {
                    value: case_value,
                    condition,
                }],
                otherwise: None,
            },
            budget(),
        )
        .unwrap();

    let outcome = store.apply_algebra_operation(
        piecewise,
        &TermAlgebraOperation::Differentiate {
            variable: SymbolName::new("Global`", "x"),
        },
        budget(),
        algebra_budget(),
    );
    match outcome {
        Err(TermAlgebraError::Refused(TermAlgebraRefusal::OutsideOperation { construct })) => {
            assert_eq!(construct, "Piecewise[variable-dependent condition]");
        }
        other => panic!("expected a piecewise refusal, got {other:?}"),
    }
}

#[test]
fn substitute_is_binder_safe_and_preserves_structure_until_simplified() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let sum = apply(&mut store, "Add", vec![x, y]);
    let relation = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::Equal,
                left: sum,
                right: zero,
            },
            budget(),
        )
        .unwrap();

    let three = store.exact_integer(3, budget()).unwrap();
    let four = store.exact_integer(4, budget()).unwrap();
    let substituted = store
        .apply_algebra_operation(
            relation,
            &TermAlgebraOperation::Substitute {
                bindings: vec![
                    (SymbolName::new("Global`", "x"), three),
                    (SymbolName::new("Global`", "y"), four),
                ],
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    // Substitute alone does not evaluate arithmetic: the Relation and the
    // unevaluated Add are both preserved structurally.
    match store.node(substituted.root).unwrap() {
        TermNode::Relation { left, .. } => match store.node(left).unwrap() {
            TermNode::Apply { arguments, .. } => {
                assert_eq!(arguments, vec![three, four]);
            }
            other => panic!("expected Add(3, 4), got {other:?}"),
        },
        other => panic!("expected a Relation, got {other:?}"),
    }

    let simplified = store
        .apply_algebra_operation(
            substituted.root,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    match store.node(simplified.root).unwrap() {
        TermNode::Relation { left, right, .. } => {
            assert_eq!(
                store.exact_rational_value(left).unwrap(),
                Some(Rational::from_i128(7))
            );
            assert_eq!(
                store.exact_rational_value(right).unwrap(),
                Some(Rational::zero())
            );
        }
        other => panic!("expected a Relation, got {other:?}"),
    }
}

#[test]
fn identical_inputs_give_byte_identical_receipts() {
    let mut store_a = TermStore::new().unwrap();
    let x_a = symbol(&mut store_a, "Global`", "x");
    let two_a = store_a.exact_integer(2, budget()).unwrap();
    let expr_a = apply(&mut store_a, "Mul", vec![two_a, x_a]);

    let mut store_b = TermStore::new().unwrap();
    // Build in a different order to exercise insertion-order independence.
    let two_b = store_b.exact_integer(2, budget()).unwrap();
    let x_b = symbol(&mut store_b, "Global`", "x");
    let expr_b = apply(&mut store_b, "Mul", vec![two_b, x_b]);

    let variable = SymbolName::new("Global`", "x");
    let outcome_a = store_a
        .apply_algebra_operation(
            expr_a,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    let outcome_b = store_b
        .apply_algebra_operation(
            expr_b,
            &TermAlgebraOperation::Differentiate { variable },
            budget(),
            algebra_budget(),
        )
        .unwrap();

    assert_eq!(outcome_a.receipt, outcome_b.receipt);
    assert_eq!(
        serde_json::to_string(&outcome_a.receipt).unwrap(),
        serde_json::to_string(&outcome_b.receipt).unwrap()
    );

    // Replaying the same operation on the same store is idempotent.
    let outcome_a_again = store_a
        .apply_algebra_operation(
            expr_a,
            &TermAlgebraOperation::Differentiate {
                variable: SymbolName::new("Global`", "x"),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(outcome_a.root, outcome_a_again.root);
    assert_eq!(outcome_a.receipt, outcome_a_again.receipt);
}

#[test]
fn receipt_wire_is_schema_pinned_and_fails_closed() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let outcome = store
        .apply_algebra_operation(
            x,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    let json = serde_json::to_string(&outcome.receipt).unwrap();
    let decoded: resolvent::TermAlgebraReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, outcome.receipt);

    assert!(
        serde_json::from_str::<resolvent::TermAlgebraReceipt>(&json.replace(
            "resolvent-term-algebra-receipt/1",
            "resolvent-term-algebra-receipt/2"
        ))
        .is_err()
    );
    assert!(
        serde_json::from_str::<resolvent::TermAlgebraReceipt>(
            &json.replace(&outcome.receipt.input_digest, "not-hex")
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<resolvent::TermAlgebraReceipt>(&json.replace('}', r#","extra":0}"#))
            .is_err()
    );
}
