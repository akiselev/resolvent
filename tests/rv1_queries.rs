use std::sync::Arc;

use resolvent::{
    Atom, BinderKind, ByteSpan, OriginBudget, OriginError, OriginKind, OriginMap, OriginRecord,
    PiecewiseCase, RuleKind, SymbolName, TermBudget, TermError, TermNode, TermPath, TermStore,
};

fn budget() -> TermBudget {
    TermBudget::default()
}

fn origin_budget() -> OriginBudget {
    OriginBudget::default()
}

fn symbol(store: &mut TermStore, namespace: &str, name: &str) -> resolvent::TermId {
    store
        .atom(Atom::Symbol(SymbolName::new(namespace, name)), budget())
        .unwrap()
}

fn apply(
    store: &mut TermStore,
    head: resolvent::TermId,
    arguments: Vec<resolvent::TermId>,
) -> resolvent::TermId {
    store
        .intern(TermNode::Apply { head, arguments }, budget())
        .unwrap()
}

#[test]
fn structural_queries_report_order_sharing_depth_and_binder_variables() {
    let mut store = TermStore::new().unwrap();
    let pair = symbol(&mut store, "System`", "Pair");
    let x = symbol(&mut store, "Global`", "x");
    let root = apply(&mut store, pair, vec![x, x]);

    assert_eq!(store.head(root).unwrap(), Some(pair));
    assert_eq!(store.head(x).unwrap(), None);
    assert_eq!(store.children(root).unwrap(), vec![pair, x, x]);
    assert_eq!(
        store.stats(root, budget()).unwrap(),
        resolvent::TermStats {
            unique_nodes: 3,
            edge_references: 3,
            max_depth: 2,
            shared_nodes: 1,
        }
    );

    let bound = store.atom(Atom::BoundVariable(0), budget()).unwrap();
    let body = apply(&mut store, pair, vec![bound, x]);
    let lambda = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Lambda,
                variable_count: 1,
                bounds: vec![],
                body,
            },
            budget(),
        )
        .unwrap();
    let analysis = store.variable_analysis(lambda, budget()).unwrap();
    assert_eq!(
        analysis.free_symbols,
        vec![
            SymbolName::new("Global`", "x"),
            SymbolName::new("System`", "Pair"),
        ]
    );
    assert_eq!(analysis.bound_variable_indices, vec![0]);
    assert_eq!(analysis.required_outer_depth, 0);
    assert_eq!(analysis.binder_nodes, 1);
    assert_eq!(
        store
            .variable_analysis(bound, budget())
            .unwrap()
            .required_outer_depth,
        1
    );
    assert_eq!(
        store.free_symbols(lambda, budget()).unwrap(),
        analysis.free_symbols
    );
    assert_eq!(store.interned_symbols().len(), 2);

    let store = Arc::new(store);
    let reader = Arc::clone(&store);
    std::thread::spawn(move || {
        assert_eq!(reader.stats(root, budget()).unwrap().shared_nodes, 1);
        assert_eq!(reader.free_symbols(root, budget()).unwrap().len(), 2);
    })
    .join()
    .unwrap();
}

#[test]
fn closed_substitution_and_exact_paths_are_capture_safe_and_location_specific() {
    let mut store = TermStore::new().unwrap();
    let pair = symbol(&mut store, "System`", "Pair");
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let bound = store.atom(Atom::BoundVariable(0), budget()).unwrap();
    let body = apply(&mut store, pair, vec![bound, x]);
    let lambda = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Lambda,
                variable_count: 1,
                bounds: vec![],
                body,
            },
            budget(),
        )
        .unwrap();

    let substituted = store
        .substitute_closed(lambda, &[(x, y)], budget())
        .unwrap();
    assert_eq!(
        store
            .variable_analysis(substituted, budget())
            .unwrap()
            .required_outer_depth,
        0
    );
    assert_eq!(
        store.substitute_closed(lambda, &[(x, bound)], budget()),
        Err(TermError::InvalidSubstitution(
            "replacement must be a closed term"
        ))
    );
    assert_eq!(
        store.substitute_closed(lambda, &[(x, y), (x, y)], budget()),
        Err(TermError::InvalidSubstitution(
            "each source term may appear only once"
        ))
    );

    let shared = apply(&mut store, pair, vec![x, x]);
    let one_occurrence = store
        .replace_at_path_closed(shared, &TermPath::new([1]), y, budget())
        .unwrap();
    assert_eq!(store.children(one_occurrence).unwrap(), vec![pair, y, x]);
    let all_occurrences = store
        .substitute_closed(shared, &[(x, y)], budget())
        .unwrap();
    assert_eq!(store.children(all_occurrences).unwrap(), vec![pair, y, y]);
    assert_eq!(
        store.replace_at_path_closed(shared, &TermPath::new([99]), y, budget()),
        Err(TermError::InvalidPath)
    );

    let truth = store.atom(Atom::Boolean(true), budget()).unwrap();
    let piecewise = store
        .intern(
            TermNode::Piecewise {
                cases: vec![PiecewiseCase {
                    value: x,
                    condition: truth,
                }],
                otherwise: Some(x),
            },
            budget(),
        )
        .unwrap();
    let replaced_otherwise = store
        .replace_at_path_closed(piecewise, &TermPath::new([2]), y, budget())
        .unwrap();
    assert_eq!(
        store.children(replaced_otherwise).unwrap(),
        vec![x, truth, y]
    );

    let slice = store
        .intern(
            TermNode::Slice {
                target: x,
                start: None,
                end: Some(x),
                step: None,
            },
            budget(),
        )
        .unwrap();
    let replaced_end = store
        .replace_at_path_closed(slice, &TermPath::new([1]), y, budget())
        .unwrap();
    assert_eq!(store.children(replaced_end).unwrap(), vec![x, y]);

    let rule = store
        .intern(
            TermNode::Rule {
                kind: RuleKind::Immediate,
                pattern: x,
                replacement: x,
                condition: Some(truth),
            },
            budget(),
        )
        .unwrap();
    let replaced_condition = store
        .replace_at_path_closed(rule, &TermPath::new([2]), y, budget())
        .unwrap();
    assert_eq!(store.children(replaced_condition).unwrap(), vec![x, x, y]);

    let replaced_body = store
        .replace_at_path_closed(lambda, &TermPath::new([0]), y, budget())
        .unwrap();
    assert_eq!(store.children(replaced_body).unwrap(), vec![y]);
}

#[test]
fn deep_substitution_is_iterative() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let mut root = x;
    for _ in 0..20_000 {
        root = store
            .intern(TermNode::Held { expression: root }, budget())
            .unwrap();
    }
    let replaced = store.substitute_closed(root, &[(x, y)], budget()).unwrap();
    assert_eq!(store.stats(replaced, budget()).unwrap().max_depth, 20_001);
}

#[test]
fn batch_cardinality_and_late_failures_are_bounded_and_transactional() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let first = store
        .intern(TermNode::Held { expression: x }, budget())
        .unwrap();
    let root = store
        .intern(TermNode::Held { expression: first }, budget())
        .unwrap();
    let before = store.metrics();
    let late = TermBudget {
        max_nodes: before.terms + 1,
        ..budget()
    };
    for _ in 0..2 {
        assert_eq!(
            store.substitute_closed(root, &[(x, y)], late),
            Err(TermError::BudgetExceeded {
                resource: "nodes",
                limit: before.terms + 1,
            })
        );
        assert_eq!(store.metrics(), before);
        assert_eq!(
            store.replace_at_path_closed(root, &TermPath::new([0, 0]), y, late),
            Err(TermError::BudgetExceeded {
                resource: "nodes",
                limit: before.terms + 1,
            })
        );
        assert_eq!(store.metrics(), before);
    }

    let narrow = TermBudget {
        max_children_per_node: 1,
        ..budget()
    };
    assert!(matches!(
        store.substitute_closed(root, &[(x, y), (y, x)], narrow),
        Err(TermError::BudgetExceeded {
            resource: "substitution requests",
            limit: 1,
        })
    ));
    assert!(matches!(
        store.rebuild_roots(&[root, root], narrow),
        Err(TermError::BudgetExceeded {
            resource: "root requests",
            limit: 1,
        })
    ));
    assert!(matches!(
        store.replace_at_path_closed(
            root,
            &TermPath::new([0, 0]),
            y,
            TermBudget {
                max_depth: 1,
                ..budget()
            }
        ),
        Err(TermError::BudgetExceeded {
            resource: "path length",
            limit: 1,
        })
    ));
    assert_eq!(store.metrics(), before);

    let (rebuilt, roots) = store
        .rebuild_roots(
            &[first, root, first],
            TermBudget {
                max_nodes: 3,
                max_children_per_node: 3,
                ..budget()
            },
        )
        .unwrap();
    assert_eq!(rebuilt.len(), 3);
    assert_eq!(roots[0], roots[2]);
}

#[test]
fn epoch_rebuild_bounds_a_million_transient_terms_without_rebinding_handles() {
    let epoch_budget = TermBudget {
        max_nodes: 20_100,
        max_depth: 20_100,
        ..budget()
    };
    let mut store = TermStore::new().unwrap();
    let mut persistent = store
        .atom(
            Atom::Symbol(SymbolName::new("Session`", "persistent")),
            epoch_budget,
        )
        .unwrap();
    let stable_digest = store.digest(persistent, epoch_budget).unwrap();
    let mut constructed = 0usize;

    for _ in 0..50 {
        let mut transient = store.atom(Atom::Boolean(false), epoch_budget).unwrap();
        for _ in 0..20_000 {
            transient = store
                .intern(
                    TermNode::Held {
                        expression: transient,
                    },
                    epoch_budget,
                )
                .unwrap();
            constructed += 1;
        }
        assert!(store.len() >= 20_002);
        let old_persistent = persistent;
        let (next, roots) = store.rebuild_roots(&[persistent], epoch_budget).unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next.node(old_persistent), Err(TermError::ForeignTerm));
        persistent = roots[0];
        assert_eq!(
            next.digest(persistent, epoch_budget).unwrap(),
            stable_digest
        );
        store = next;
    }
    assert_eq!(constructed, 1_000_000);
    assert_eq!(store.len(), 1);
}

#[test]
fn provenance_sidecars_are_multi_origin_identity_neutral_and_thread_readable() {
    let mut store = TermStore::new().unwrap();
    let term = symbol(&mut store, "Global`", "x");
    let bytes = store.canonical_bytes(term, budget()).unwrap();
    let digest = store.digest(term, budget()).unwrap();
    let authored = OriginRecord {
        locator: Some("memory://first".into()),
        byte_span: Some(ByteSpan::new(4, 9).unwrap()),
        kind: OriginKind::Authored,
        consumer_id: Some("caller:decl-1".into()),
    };
    let generated = OriginRecord {
        locator: None,
        byte_span: None,
        kind: OriginKind::Generated {
            transformation: "differentiate".into(),
            parent: Some(digest),
        },
        consumer_id: Some("caller:generated-2".into()),
    };
    let mut origins = OriginMap::new();
    assert!(
        origins
            .attach_term(&store, term, authored.clone(), budget(), origin_budget())
            .unwrap()
    );
    assert!(
        !origins
            .attach_digest(digest, authored.clone(), origin_budget())
            .unwrap()
    );
    assert!(
        origins
            .attach_digest(digest, generated.clone(), origin_budget())
            .unwrap()
    );
    assert_eq!(origins.term_count(), 1);
    assert_eq!(origins.record_count(), 2);
    assert_eq!(origins.records(digest), &[authored, generated]);
    assert_eq!(store.canonical_bytes(term, budget()).unwrap(), bytes);

    let mut independent = TermStore::new().unwrap();
    let same_term = symbol(&mut independent, "Global`", "x");
    let same_digest = independent.digest(same_term, budget()).unwrap();
    assert_eq!(same_digest, digest);
    assert_eq!(origins.records(same_digest).len(), 2);

    let origins = Arc::new(origins);
    let reader = Arc::clone(&origins);
    std::thread::spawn(move || assert_eq!(reader.records(digest).len(), 2))
        .join()
        .unwrap();

    assert_eq!(
        ByteSpan::new(9, 4),
        Err(OriginError::Invalid("byte span is reversed"))
    );
    let mut invalid = OriginMap::new();
    assert_eq!(
        invalid.attach_digest(
            digest,
            OriginRecord {
                locator: Some("   ".into()),
                byte_span: None,
                kind: OriginKind::Authored,
                consumer_id: None,
            },
            origin_budget(),
        ),
        Err(OriginError::Invalid(
            "locator and consumer ID must be nonblank"
        ))
    );
    assert_eq!(invalid.record_count(), 0);
    assert_eq!(
        invalid.attach_digest(
            digest,
            OriginRecord {
                locator: None,
                byte_span: Some(ByteSpan { start: 0, end: 1 }),
                kind: OriginKind::Authored,
                consumer_id: Some("caller:id".into()),
            },
            origin_budget(),
        ),
        Err(OriginError::Invalid("byte span requires a locator"))
    );
    assert_eq!(
        invalid.attach_digest(
            digest,
            OriginRecord {
                locator: None,
                byte_span: None,
                kind: OriginKind::Generated {
                    transformation: " \t".into(),
                    parent: None,
                },
                consumer_id: Some("caller:id".into()),
            },
            origin_budget(),
        ),
        Err(OriginError::Invalid(
            "generated transformation must be nonblank"
        ))
    );
}

#[test]
fn provenance_budgets_are_typed_indexed_and_atomic() {
    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let x_digest = store.digest(x, budget()).unwrap();
    let y_digest = store.digest(y, budget()).unwrap();
    let first = OriginRecord {
        locator: Some("memory://first".into()),
        byte_span: Some(ByteSpan::new(0, 1).unwrap()),
        kind: OriginKind::Authored,
        consumer_id: Some("first".into()),
    };
    let second = OriginRecord {
        locator: Some("memory://second".into()),
        byte_span: Some(ByteSpan::new(2, 3).unwrap()),
        kind: OriginKind::Authored,
        consumer_id: Some("second".into()),
    };

    let mut origins = OriginMap::new();
    let per_term = OriginBudget {
        max_records_per_term: 1,
        ..origin_budget()
    };
    assert_eq!(
        origins.attach_many_digest(x_digest, &[first.clone(), second.clone()], per_term),
        Err(OriginError::BudgetExceeded {
            resource: "records per term",
            limit: 1,
        })
    );
    assert_eq!(origins.record_count(), 0);
    assert_eq!(origins.text_bytes(), 0);

    assert_eq!(
        origins.attach_many_digest(
            x_digest,
            &[first.clone(), second.clone()],
            OriginBudget {
                max_work: 1,
                ..origin_budget()
            },
        ),
        Err(OriginError::BudgetExceeded {
            resource: "work",
            limit: 1,
        })
    );
    assert_eq!(origins.record_count(), 0);
    assert!(matches!(
        origins.attach_digest(
            x_digest,
            first.clone(),
            OriginBudget {
                max_text_bytes: 1,
                ..origin_budget()
            },
        ),
        Err(OriginError::BudgetExceeded {
            resource: "text bytes",
            limit: 1,
        })
    ));
    assert_eq!(origins.record_count(), 0);

    assert_eq!(
        origins
            .attach_many_digest(x_digest, &[first.clone(), first.clone()], origin_budget(),)
            .unwrap(),
        1
    );
    let stable_count = origins.record_count();
    let stable_text = origins.text_bytes();
    assert_eq!(
        origins.attach_digest(
            y_digest,
            second,
            OriginBudget {
                max_total_records: 1,
                ..origin_budget()
            },
        ),
        Err(OriginError::BudgetExceeded {
            resource: "total records",
            limit: 1,
        })
    );
    assert_eq!(origins.record_count(), stable_count);
    assert_eq!(origins.text_bytes(), stable_text);
    assert!(origins.records(y_digest).is_empty());
}
