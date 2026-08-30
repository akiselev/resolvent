//! RV1-C2/C3: exact literal ingress and the term-level algebra-operation
//! surface (`AlgebraOperation::{Differentiate,Simplify,Substitute}`).

use resolvent::{
    AlgebraBudget, AlgebraError, Atom, BinderKind, CollectionKind, OPERATOR_NAMESPACE,
    PiecewiseCase, Rational, RelationOperator, RuleKind, SymbolName, TermAlgebraError,
    TermAlgebraOperation, TermAlgebraRefusal, TermBudget, TermNode, TermStore,
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
fn simplify_folds_pow_of_a_literal_base_to_an_exact_literal() {
    // GX-F6: `Expr::canonicalize` now folds `Pow` of an exact literal base
    // and an integer exponent to an exact literal. `Simplify` inherits this
    // via its lossless `Expr` round trip, so a substituted `5^2` becomes the
    // atom `25` rather than staying a structural `Pow` node. This closes the
    // scope boundary the earlier RV1-C2/C3 slice documented.
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

    assert_eq!(
        store.exact_rational_value(simplified.root).unwrap(),
        Some(Rational::from_i128(25))
    );
    assert!(matches!(
        store.node(simplified.root).unwrap(),
        TermNode::Atom(Atom::Rational(_))
    ));
}

#[test]
fn simplify_folds_negative_and_fractional_literal_powers() {
    let mut store = TermStore::new().unwrap();
    let half = store
        .exact_rational(Rational::from_ratio(1, 2), budget())
        .unwrap();
    let neg_one = store.exact_integer(-1, budget()).unwrap();
    let half_to_neg_one = apply(&mut store, "Pow", vec![half, neg_one]);

    let simplified = store
        .apply_algebra_operation(
            half_to_neg_one,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    assert_eq!(
        store.exact_rational_value(simplified.root).unwrap(),
        Some(Rational::from_i128(2))
    );
}

#[test]
fn simplify_pow_zero_to_the_zero_folds_to_one() {
    // `0^0` folds to the literal `1`. This is the pre-existing `Expr::Pow`
    // canonicalization rule (any base raised to exponent `0` folds to `1`,
    // regardless of whether the base is a literal); it matches the
    // convention `Expr::evaluate` already applies via `Rational::checked_pow`
    // treating a zero exponent as the empty product.
    let mut store = TermStore::new().unwrap();
    let zero = store.exact_integer(0, budget()).unwrap();
    let zero_to_zero = apply(&mut store, "Pow", vec![zero, zero]);

    let simplified = store
        .apply_algebra_operation(
            zero_to_zero,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    assert_eq!(
        store.exact_rational_value(simplified.root).unwrap(),
        Some(Rational::from_i128(1))
    );
}

#[test]
fn simplify_pow_negative_exponent_of_zero_is_a_division_by_zero_error() {
    // `0` raised to a negative exponent is a genuine division by exact zero;
    // it refuses rather than silently producing a value, matching what
    // `Expr::evaluate` already reports for the identical construct.
    let mut store = TermStore::new().unwrap();
    let zero = store.exact_integer(0, budget()).unwrap();
    let neg_two = store.exact_integer(-2, budget()).unwrap();
    let zero_to_neg_two = apply(&mut store, "Pow", vec![zero, neg_two]);

    let outcome = store.apply_algebra_operation(
        zero_to_neg_two,
        &TermAlgebraOperation::Simplify,
        budget(),
        algebra_budget(),
    );
    assert!(matches!(
        outcome,
        Err(TermAlgebraError::Algebra(AlgebraError::DivisionByZero))
    ));
}

#[test]
fn simplify_pow_literal_fold_is_charged_against_the_coefficient_bit_budget() {
    // An adversarial exponent must not force unbounded exact-integer growth:
    // `2^10000` needs roughly 10,000 bits, which a 32-bit budget refuses.
    let mut store = TermStore::new().unwrap();
    let two = store.exact_integer(2, budget()).unwrap();
    let big_exponent = store.exact_integer(10_000, budget()).unwrap();
    let two_to_big = apply(&mut store, "Pow", vec![two, big_exponent]);

    let tight_budget = AlgebraBudget {
        max_coefficient_bits: 32,
        ..AlgebraBudget::default()
    };
    let outcome = store.apply_algebra_operation(
        two_to_big,
        &TermAlgebraOperation::Simplify,
        budget(),
        tight_budget,
    );
    assert!(matches!(
        outcome,
        Err(TermAlgebraError::Algebra(
            AlgebraError::CoefficientBits { .. }
        ))
    ));
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

// GX-F6: `Simplify`/`Substitute` recursion into `Collection`/`OrderedMap`/
// `Index`/`Slice`/`Rule`, elementwise `Differentiate` over a `Collection` of
// scalar entries, and `Held`/`Binder` opacity.

#[test]
fn simplify_recurses_into_collection_elements_and_preserves_kind() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let x_plus_zero = apply(&mut store, "Add", vec![x, zero]);
    let list = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::List,
                elements: vec![x_plus_zero, y],
            },
            budget(),
        )
        .unwrap();

    let outcome = store
        .apply_algebra_operation(
            list,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    match store.node(outcome.root).unwrap() {
        TermNode::Collection { kind, elements } => {
            assert_eq!(kind, CollectionKind::List);
            assert_eq!(elements.len(), 2);
            // `x + 0` simplifies down to the bare symbol `x`.
            assert_eq!(elements[0], x);
            assert_eq!(elements[1], y);
        }
        other => panic!("expected a Collection, got {other:?}"),
    }
}

#[test]
fn simplify_recurses_into_ordered_map_values_but_preserves_keys_structurally() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let key = apply(&mut store, "Add", vec![x, zero]); // x + 0
    let value = apply(&mut store, "Add", vec![y, zero]); // y + 0
    let map = store
        .intern(
            TermNode::OrderedMap {
                entries: vec![(key, value)],
            },
            budget(),
        )
        .unwrap();

    let outcome = store
        .apply_algebra_operation(
            map,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();

    match store.node(outcome.root).unwrap() {
        TermNode::OrderedMap { entries } => {
            assert_eq!(entries.len(), 1);
            // The key is preserved byte-for-byte: still the unsimplified
            // `x + 0`, never simplified alongside the value.
            assert_eq!(entries[0].0, key);
            // The value simplifies down to the bare symbol `y`.
            assert_eq!(entries[0].1, y);
        }
        other => panic!("expected an OrderedMap, got {other:?}"),
    }
}

#[test]
fn simplify_recurses_into_index_and_slice_targets_and_positions() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let zero = store.exact_integer(0, budget()).unwrap();
    let two = store.exact_integer(2, budget()).unwrap();
    let one = store.exact_integer(1, budget()).unwrap();
    let x_plus_zero = apply(&mut store, "Add", vec![x, zero]);
    let one_plus_one = apply(&mut store, "Add", vec![one, one]);

    let index = store
        .intern(
            TermNode::Index {
                target: x_plus_zero,
                indices: vec![one_plus_one],
            },
            budget(),
        )
        .unwrap();
    let outcome = store
        .apply_algebra_operation(
            index,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    match store.node(outcome.root).unwrap() {
        TermNode::Index { target, indices } => {
            assert_eq!(target, x);
            assert_eq!(indices.len(), 1);
            assert_eq!(
                store.exact_rational_value(indices[0]).unwrap(),
                Some(Rational::from_i128(2))
            );
        }
        other => panic!("expected an Index, got {other:?}"),
    }

    let slice = store
        .intern(
            TermNode::Slice {
                target: x_plus_zero,
                start: Some(zero),
                end: Some(two),
                step: None,
            },
            budget(),
        )
        .unwrap();
    let outcome = store
        .apply_algebra_operation(
            slice,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    match store.node(outcome.root).unwrap() {
        TermNode::Slice {
            target,
            start,
            end,
            step,
        } => {
            assert_eq!(target, x);
            // The literal atoms round-trip through `Expr` and come back as
            // `Atom::Rational`, so they are not the *same* TermIds as the
            // original `Atom::Integer` atoms even though their values are
            // unchanged (see the module's "Literal spelling is normalized"
            // documentation).
            assert_eq!(
                start.and_then(|id| store.exact_rational_value(id).unwrap()),
                Some(Rational::zero())
            );
            assert_eq!(
                end.and_then(|id| store.exact_rational_value(id).unwrap()),
                Some(Rational::from_i128(2))
            );
            assert_eq!(step, None);
        }
        other => panic!("expected a Slice, got {other:?}"),
    }
}

#[test]
fn simplify_recurses_into_rule_pattern_replacement_and_condition() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.exact_integer(0, budget()).unwrap();
    let pattern = apply(&mut store, "Add", vec![x, zero]);
    let replacement = apply(&mut store, "Add", vec![y, zero]);
    let condition = apply(&mut store, "Add", vec![x, zero]);
    let rule = store
        .intern(
            TermNode::Rule {
                kind: RuleKind::Delayed,
                pattern,
                replacement,
                condition: Some(condition),
            },
            budget(),
        )
        .unwrap();

    let outcome = store
        .apply_algebra_operation(
            rule,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    match store.node(outcome.root).unwrap() {
        TermNode::Rule {
            kind,
            pattern,
            replacement,
            condition,
        } => {
            assert_eq!(kind, RuleKind::Delayed);
            assert_eq!(pattern, x);
            assert_eq!(replacement, y);
            assert_eq!(condition, Some(x));
        }
        other => panic!("expected a Rule, got {other:?}"),
    }
}

#[test]
fn simplify_leaves_held_and_binder_nodes_byte_identical() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let zero = store.exact_integer(0, budget()).unwrap();
    let unsimplified = apply(&mut store, "Add", vec![x, zero]); // x + 0

    let held = store
        .intern(
            TermNode::Held {
                expression: unsimplified,
            },
            budget(),
        )
        .unwrap();
    let outcome = store
        .apply_algebra_operation(
            held,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(outcome.root, held);

    let lambda = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Lambda,
                variable_count: 1,
                bounds: vec![],
                body: unsimplified,
            },
            budget(),
        )
        .unwrap();
    let outcome = store
        .apply_algebra_operation(
            lambda,
            &TermAlgebraOperation::Simplify,
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(outcome.root, lambda);
}

#[test]
fn differentiate_collection_elementwise_matches_scalar_differentiation_per_element() {
    // This is what a vector-valued case binding such as `[sin(x), 0]` needs.
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let zero = store.exact_integer(0, budget()).unwrap();
    let sin_x = apply(&mut store, "Sin", vec![x]);
    let variable = SymbolName::new("Global`", "x");

    // Scalar baseline: differentiate `sin(x)` alone.
    let scalar_derivative = store
        .apply_algebra_operation(
            sin_x,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();

    // Vector case: differentiate `[sin(x), 0]` elementwise.
    let vector = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::List,
                elements: vec![sin_x, zero],
            },
            budget(),
        )
        .unwrap();
    let vector_derivative = store
        .apply_algebra_operation(
            vector,
            &TermAlgebraOperation::Differentiate { variable },
            budget(),
            algebra_budget(),
        )
        .unwrap();

    let (kind, elements) = match store.node(vector_derivative.root).unwrap() {
        TermNode::Collection { kind, elements } => (kind, elements),
        other => panic!("expected a Collection, got {other:?}"),
    };
    assert_eq!(kind, CollectionKind::List);
    assert_eq!(elements.len(), 2);
    // Hash-consing means an identically-derived scalar subtree is the exact
    // same TermId whether it was produced standalone or as one element of
    // the elementwise vector differentiation.
    assert_eq!(elements[0], scalar_derivative.root);
    assert_eq!(
        store.exact_rational_value(elements[1]).unwrap(),
        Some(Rational::zero())
    );
}

#[test]
fn differentiate_collection_propagates_the_offending_elements_own_construct_label() {
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
    let vector = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::List,
                elements: vec![x, relation],
            },
            budget(),
        )
        .unwrap();

    let outcome = store.apply_algebra_operation(
        vector,
        &TermAlgebraOperation::Differentiate { variable },
        budget(),
        algebra_budget(),
    );
    match outcome {
        Err(TermAlgebraError::Refused(TermAlgebraRefusal::OutsideOperation { construct })) => {
            // The refusal names the offending element's own construct, not a
            // generic "Collection" label.
            assert_eq!(construct, "Relation");
        }
        other => panic!("expected a Relation refusal, got {other:?}"),
    }
}

#[test]
fn differentiate_refuses_ordered_map_index_slice_and_rule_containers() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let variable = SymbolName::new("Global`", "x");

    let ordered_map = store
        .intern(
            TermNode::OrderedMap {
                entries: vec![(x, x)],
            },
            budget(),
        )
        .unwrap();
    let index = store
        .intern(
            TermNode::Index {
                target: x,
                indices: vec![x],
            },
            budget(),
        )
        .unwrap();
    let slice = store
        .intern(
            TermNode::Slice {
                target: x,
                start: Some(x),
                end: None,
                step: None,
            },
            budget(),
        )
        .unwrap();
    let rule = store
        .intern(
            TermNode::Rule {
                kind: RuleKind::Immediate,
                pattern: x,
                replacement: x,
                condition: None,
            },
            budget(),
        )
        .unwrap();

    for (term, expected_construct) in [
        (ordered_map, "OrderedMap"),
        (index, "Index"),
        (slice, "Slice"),
        (rule, "Rule"),
    ] {
        let outcome = store.apply_algebra_operation(
            term,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        );
        match outcome {
            Err(TermAlgebraError::Refused(TermAlgebraRefusal::OutsideOperation { construct })) => {
                assert_eq!(construct, expected_construct);
            }
            other => panic!("expected a {expected_construct} refusal, got {other:?}"),
        }
    }
}

#[test]
fn collection_differentiate_receipts_are_deterministic() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let two = store.exact_integer(2, budget()).unwrap();
    let x_squared = apply(&mut store, "Pow", vec![x, two]);
    let vector = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::List,
                elements: vec![x_squared],
            },
            budget(),
        )
        .unwrap();

    let variable = SymbolName::new("Global`", "x");
    let first = store
        .apply_algebra_operation(
            vector,
            &TermAlgebraOperation::Differentiate {
                variable: variable.clone(),
            },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    let second = store
        .apply_algebra_operation(
            vector,
            &TermAlgebraOperation::Differentiate { variable },
            budget(),
            algebra_budget(),
        )
        .unwrap();
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.root, second.root);
}

#[test]
fn substitute_replaces_symbols_inside_a_collection() {
    // `TermAlgebraOperation::Substitute` never went through `Expr`; it
    // already delegates to the binder-safe `TermStore::substitute_closed`,
    // which handles every node kind (including `Collection`) structurally.
    // This is a positive check confirming that pre-existing coverage.
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let list = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::List,
                elements: vec![x, y],
            },
            budget(),
        )
        .unwrap();

    let three = store.exact_integer(3, budget()).unwrap();
    let four = store.exact_integer(4, budget()).unwrap();
    let outcome = store
        .apply_algebra_operation(
            list,
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

    match store.node(outcome.root).unwrap() {
        TermNode::Collection { kind, elements } => {
            assert_eq!(kind, CollectionKind::List);
            assert_eq!(elements, vec![three, four]);
        }
        other => panic!("expected a Collection, got {other:?}"),
    }
}
