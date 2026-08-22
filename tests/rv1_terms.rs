use resolvent::{
    Atom, BinderKind, BooleanOperator, CollectionKind, ExactDecimal, PiecewiseCase, PrecisionReal,
    Rational, RelationOperator, RuleKind, SymbolName, SymbolicConstant, TermBudget, TermError,
    TermNode, TermStore, decode_canonical_term,
};

fn budget() -> TermBudget {
    TermBudget::default()
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn exact_atom_classes_have_frozen_tagged_wire_vectors() {
    let atoms = vec![
        Atom::integer("-12345678901234567890").unwrap(),
        Atom::Rational(Rational::from_ratio(1, 3)),
        Atom::ExactDecimal(ExactDecimal::parse("0.10").unwrap()),
        Atom::ExactIeee754Bits(0x3fb9_9999_9999_999a),
        Atom::MachineFloatBits(0x3fb9_9999_9999_999a),
        Atom::PrecisionReal(PrecisionReal::new("12300", -5, 80).unwrap()),
        Atom::String("λ".into()),
        Atom::Bytes(vec![0, 255]),
        Atom::Symbol(SymbolName::new("Global`", "x")),
        Atom::Boolean(true),
        Atom::Constant(SymbolicConstant::Pi),
    ];
    let expected = [
        "5245534f4c56454e542d5445524d0001010101152d313233343536373839303132333435363738393000",
        "5245534f4c56454e542d5445524d00010101020131013300",
        "5245534f4c56454e542d5445524d000101010301310200",
        "5245534f4c56454e542d5445524d00010101043fb999999999999a00",
        "5245534f4c56454e542d5445524d00010101053fb999999999999a00",
        "5245534f4c56454e542d5445524d000101010603313233055000",
        "5245534f4c56454e542d5445524d000101010702cebb00",
        "5245534f4c56454e542d5445524d00010101080200ff00",
        "5245534f4c56454e542d5445524d000101010907476c6f62616c60017800",
        "5245534f4c56454e542d5445524d000101010a0100",
        "5245534f4c56454e542d5445524d000101010b0000",
    ];
    let expected_digests = [
        "ef74db39caa1b2e8905ab57f61843de3fbcefa689ebfcfda41c1e6fe6da5f1d5",
        "8a970f56c9117a74f59084b4463ca714935ff6ded113484b78b3c77be275eb33",
        "217d4921f6661f4a30124a309edee92fa8ffb147a8034b7702fb929ae2ce70f7",
        "3faa3ad7819d472dcbcc752b08b3b9f3ce39b36b2bbac73268b84fc4dfb01e79",
        "598a3fa42274519bcae561a9c762616e6a21dc13f14ae4f5aca840da0e634a11",
        "87b51602f7604b456c474180247a0dc075c20355c67e22c530f83d85380c30aa",
        "f2d7358cf0db7aa32c9eb8b2e946b3733c42c0f121c9bd1ca0371913476da227",
        "e6ad1562402b6c8cb27db23ee9713134e3636f247b87f7a0daa1eb1e1b479ed9",
        "38d78c752e33e8fda905b925783c91ccdd96a3784d6418df858a23f6162b603c",
        "b2920cd344fad2424bc611ec1994e493f2fb878fad6b941a1136b49fae1ffb58",
        "5e49da2bbd90a67cfc8746d46383a48359325aee581f44620c2769f234260160",
    ];
    let mut actual = Vec::new();
    let mut actual_digests = Vec::new();
    for atom in atoms {
        let mut store = TermStore::new().unwrap();
        let term = store.atom(atom, budget()).unwrap();
        actual.push(hex(&store.canonical_bytes(term, budget()).unwrap()));
        actual_digests.push(hex(store.digest(term, budget()).unwrap().as_bytes()));
    }
    assert_eq!(actual, expected);
    assert_eq!(actual_digests, expected_digests);

    // A bound variable cannot be a stable root by itself, so freeze its atom
    // tag inside the smallest closed binder.
    let mut store = TermStore::new().unwrap();
    let variable = store.atom(Atom::BoundVariable(0), budget()).unwrap();
    let lambda = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Lambda,
                variable_count: 1,
                bounds: Vec::new(),
                body: variable,
            },
            budget(),
        )
        .unwrap();
    let bound_bytes = hex(&store.canonical_bytes(lambda, budget()).unwrap());
    let bound_digest = hex(store.digest(lambda, budget()).unwrap().as_bytes());
    assert_eq!(
        bound_bytes,
        "5245534f4c56454e542d5445524d000102010c001a0001000001"
    );
    assert_eq!(
        bound_digest,
        "4d670161936f197346118e5ae0cd469c3860e9258682a8993c47d1e18f81f96c"
    );
}

#[test]
fn every_node_and_enum_subtag_has_frozen_wire_bytes_and_digest() {
    fn record(label: &str, build: impl FnOnce(&mut TermStore) -> resolvent::TermId) -> String {
        let mut store = TermStore::new().unwrap();
        let root = build(&mut store);
        format!(
            "{label}|{}|{}\n",
            hex(&store.canonical_bytes(root, budget()).unwrap()),
            hex(store.digest(root, budget()).unwrap().as_bytes())
        )
    }
    fn binary(store: &mut TermStore) -> (resolvent::TermId, resolvent::TermId) {
        let zero = store.atom(Atom::integer("0").unwrap(), budget()).unwrap();
        let one = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
        (zero, one)
    }

    let mut actual = String::new();
    for (label, constant) in [
        ("constant.pi", SymbolicConstant::Pi),
        ("constant.e", SymbolicConstant::E),
        ("constant.i", SymbolicConstant::ImaginaryUnit),
        ("constant.infinity", SymbolicConstant::Infinity),
        (
            "constant.complex_infinity",
            SymbolicConstant::ComplexInfinity,
        ),
        ("constant.undefined", SymbolicConstant::Undefined),
    ] {
        actual.push_str(&record(label, |store| {
            store.atom(Atom::Constant(constant), budget()).unwrap()
        }));
    }
    actual.push_str(&record("node.apply", |store| {
        let (head, argument) = binary(store);
        apply(store, head, vec![argument])
    }));
    for (label, operator) in [
        ("relation.eq", RelationOperator::Equal),
        ("relation.ne", RelationOperator::NotEqual),
        ("relation.lt", RelationOperator::Less),
        ("relation.le", RelationOperator::LessEqual),
        ("relation.gt", RelationOperator::Greater),
        ("relation.ge", RelationOperator::GreaterEqual),
    ] {
        actual.push_str(&record(label, |store| {
            let (left, right) = binary(store);
            store
                .intern(
                    TermNode::Relation {
                        operator,
                        left,
                        right,
                    },
                    budget(),
                )
                .unwrap()
        }));
    }
    for (label, operator, arity) in [
        ("boolean.not", BooleanOperator::Not, 1),
        ("boolean.and", BooleanOperator::And, 2),
        ("boolean.or", BooleanOperator::Or, 2),
        ("boolean.xor", BooleanOperator::Xor, 2),
        ("boolean.implies", BooleanOperator::Implies, 2),
        ("boolean.equivalent", BooleanOperator::Equivalent, 2),
    ] {
        actual.push_str(&record(label, |store| {
            let (left, right) = binary(store);
            let arguments = if arity == 1 {
                vec![left]
            } else {
                vec![left, right]
            };
            store
                .intern(
                    TermNode::Boolean {
                        operator,
                        arguments,
                    },
                    budget(),
                )
                .unwrap()
        }));
    }
    actual.push_str(&record("node.condition", |store| {
        let expression = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
        let condition = store.atom(Atom::Boolean(true), budget()).unwrap();
        store
            .intern(
                TermNode::Condition {
                    expression,
                    condition,
                },
                budget(),
            )
            .unwrap()
    }));
    for with_otherwise in [false, true] {
        actual.push_str(&record(
            if with_otherwise {
                "piecewise.some"
            } else {
                "piecewise.none"
            },
            |store| {
                let (value, otherwise) = binary(store);
                let condition = store.atom(Atom::Boolean(true), budget()).unwrap();
                store
                    .intern(
                        TermNode::Piecewise {
                            cases: vec![PiecewiseCase { value, condition }],
                            otherwise: with_otherwise.then_some(otherwise),
                        },
                        budget(),
                    )
                    .unwrap()
            },
        ));
    }
    for label in ["collection.tuple", "collection.list", "collection.array"] {
        actual.push_str(&record(label, |store| {
            let (left, right) = binary(store);
            let kind = match label {
                "collection.tuple" => CollectionKind::Tuple,
                "collection.list" => CollectionKind::List,
                _ => CollectionKind::Array { shape: vec![2] },
            };
            store
                .intern(
                    TermNode::Collection {
                        kind,
                        elements: vec![left, right],
                    },
                    budget(),
                )
                .unwrap()
        }));
    }
    actual.push_str(&record("node.ordered_map", |store| {
        let (key, value) = binary(store);
        store
            .intern(
                TermNode::OrderedMap {
                    entries: vec![(key, value)],
                },
                budget(),
            )
            .unwrap()
    }));
    actual.push_str(&record("node.index", |store| {
        let (target, index) = binary(store);
        store
            .intern(
                TermNode::Index {
                    target,
                    indices: vec![index],
                },
                budget(),
            )
            .unwrap()
    }));
    for all_some in [false, true] {
        actual.push_str(&record(
            if all_some { "slice.some" } else { "slice.none" },
            |store| {
                let (target, value) = binary(store);
                store
                    .intern(
                        TermNode::Slice {
                            target,
                            start: all_some.then_some(value),
                            end: all_some.then_some(value),
                            step: all_some.then_some(value),
                        },
                        budget(),
                    )
                    .unwrap()
            },
        ));
    }
    for (label, kind, conditioned) in [
        ("rule.immediate.none", RuleKind::Immediate, false),
        ("rule.delayed.some", RuleKind::Delayed, true),
        ("rule.pattern.none", RuleKind::Pattern, false),
    ] {
        actual.push_str(&record(label, |store| {
            let (pattern, replacement) = binary(store);
            store
                .intern(
                    TermNode::Rule {
                        kind,
                        pattern,
                        replacement,
                        condition: conditioned.then_some(pattern),
                    },
                    budget(),
                )
                .unwrap()
        }));
    }
    for (label, kind, bound_count) in [
        ("binder.lambda", BinderKind::Lambda, 0),
        ("binder.sum", BinderKind::Sum, 2),
        ("binder.product", BinderKind::Product, 2),
        ("binder.integral", BinderKind::Integral, 2),
        ("binder.limit", BinderKind::Limit, 1),
        ("binder.local", BinderKind::Local, 1),
    ] {
        actual.push_str(&record(label, |store| {
            let (zero, one) = binary(store);
            let body = store.atom(Atom::BoundVariable(0), budget()).unwrap();
            let bounds = match bound_count {
                0 => vec![],
                1 => vec![zero],
                _ => vec![zero, one],
            };
            store
                .intern(
                    TermNode::Binder {
                        kind,
                        variable_count: 1,
                        bounds,
                        body,
                    },
                    budget(),
                )
                .unwrap()
        }));
    }
    actual.push_str(&record("node.held", |store| {
        let expression = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
        store
            .intern(TermNode::Held { expression }, budget())
            .unwrap()
    }));

    const EXPECTED: &str = r#"constant.pi|5245534f4c56454e542d5445524d000101010b0000|5e49da2bbd90a67cfc8746d46383a48359325aee581f44620c2769f234260160
constant.e|5245534f4c56454e542d5445524d000101010b0100|167608a8233c4fab98fbf3dcd2a04c1e0dfb186bdbc15e7a2f288e7b439dd2de
constant.i|5245534f4c56454e542d5445524d000101010b0200|5c6d3c9d4b5c6b4559c0c50227dfa8226fa1b8a286258d08faa4ff527649730c
constant.infinity|5245534f4c56454e542d5445524d000101010b0300|54b7a129167faa85feb0dfd0d652b10467a8c31e5afb2316bdc49144fcf42b71
constant.complex_infinity|5245534f4c56454e542d5445524d000101010b0400|018a8745aef48a0fea72173da0ae642fa3270a7a19938540c0b8e3ba17b1bf08
constant.undefined|5245534f4c56454e542d5445524d000101010b0500|deac56e66c7b4362bd73661697cede430adc5499c13fccc7c239deaaa6db526e
node.apply|5245534f4c56454e542d5445524d00010301010130010101311000010102|9ebdaaaae7bf41aa3efcdcbadd6074473812b101037b80eaabca546878386637
relation.eq|5245534f4c56454e542d5445524d00010301010130010101311100000102|aee32b4b3b6b153cba9f28f7099c82a00f072bb03a8745f6a50438f576f686d7
relation.ne|5245534f4c56454e542d5445524d00010301010130010101311101000102|c9121e87bc34e5925915abd508fca0ef0a5cc91fb55660ecc28e8fc9925ae832
relation.lt|5245534f4c56454e542d5445524d00010301010130010101311102000102|7de676195832cf3f4fd34c0f448c6f0b2c50268c99a884a154f83b53336ad082
relation.le|5245534f4c56454e542d5445524d00010301010130010101311103000102|f15e7016cf395ac7763a522ae804faf1e5fdabc08a1f4a0da1ccf9cf63bf12f4
relation.gt|5245534f4c56454e542d5445524d00010301010130010101311104000102|b13d16f9f6a519e57c75f3bd56f415532fffe8ad848ca8bbac8ce944fe2cdb28
relation.ge|5245534f4c56454e542d5445524d00010301010130010101311105000102|e54e917c035cd4468d1c3c895fee5133ce628159ccc97786e15b679ef6f9d386
boolean.not|5245534f4c56454e542d5445524d000102010101301200010001|a1849e761377817df05eb9e3028ad80dbacbe8d49b389e2db8e8c047ecd3f51f
boolean.and|5245534f4c56454e542d5445524d0001030101013001010131120102000102|3a1017f225eab909db27bfc0680d8b21dac364b674b963ff2dc695d0eb246255
boolean.or|5245534f4c56454e542d5445524d0001030101013001010131120202000102|d314de9a39831fe1b66916b5d5dbfaf5b5d98a344aa063d86f0a47a046980fb9
boolean.xor|5245534f4c56454e542d5445524d0001030101013001010131120302000102|7efb4970da83695d82a7ac7f83149f700ffa8f83e615053dce5fc461a6f7c72e
boolean.implies|5245534f4c56454e542d5445524d0001030101013001010131120402000102|d90c749ae32bfe37fe0b9716ac6e267504f713adf8a7d030093858cadbdcf647
boolean.equivalent|5245534f4c56454e542d5445524d0001030101013001010131120502000102|827b856fb77229d6cf42b8ddfc043066be7ff1f5e33e5f06931ac681e5b4914f
node.condition|5245534f4c56454e542d5445524d00010301010131010a0113000102|0560868816a905f2187336ab1b6749b48dbf6e2326427d45e1aa5f09a9f9eed8
piecewise.none|5245534f4c56454e542d5445524d00010301010130010a01140100010002|3d565b578e144380ba19494a9ab79146dc71a1d409ab3a95d66aa496b826e05d
piecewise.some|5245534f4c56454e542d5445524d00010401010130010a010101013114010001010203|122f92f712dbca735b8ccbceb518c8ad2358c6f7f4a22e1ee9a6f78bf2f0cb85
collection.tuple|5245534f4c56454e542d5445524d0001030101013001010131150002000102|ae6bcfdac3300293ae040a22f240d1ea74817b60fd1692cab9135cfe80fd81b7
collection.list|5245534f4c56454e542d5445524d0001030101013001010131150102000102|f140cc60bc5568d82b7eed07c7eff528c59d46186e4ea9236c9f0243e7f091a4
collection.array|5245534f4c56454e542d5445524d00010301010130010101311502010202000102|bb2d56122a6e3e34b0fac315e61c9497501954acd58408847a48acd193da3804
node.ordered_map|5245534f4c56454e542d5445524d00010301010130010101311601000102|f822fceab2a65473a41f9692d8fdb713d3a0ee155144d8fe78086b9eba54a0f0
node.index|5245534f4c56454e542d5445524d00010301010130010101311700010102|623d57f9e1ab28d3ee11282084793ec60470265aeba8cba1ff517dc49108aae7
slice.none|5245534f4c56454e542d5445524d00010201010130180000000001|2ab1e6ebc23adcc0b38b7ce78ae84ba9d44169a307c71970629ed65622c8f369
slice.some|5245534f4c56454e542d5445524d0001030101013001010131180001010101010102|60b1c60c7e456a5d2652827a0608a741b5ede64a6c68bcdec42234005ce57f4a
rule.immediate.none|5245534f4c56454e542d5445524d0001030101013001010131190000010002|10fe17cbe489163704007027c151cdab69c75dd1af21b87617832afae6ea731f
rule.delayed.some|5245534f4c56454e542d5445524d000103010101300101013119010001010002|658dc9e2989fe101c25308c2d461bef501a3e86407a995c1166659ad2d517755
rule.pattern.none|5245534f4c56454e542d5445524d0001030101013001010131190200010002|23a862828066d8b83dbc47802fc31200bc1762b2415d815af2764cdd30ecbf5e
binder.lambda|5245534f4c56454e542d5445524d000102010c001a0001000001|4d670161936f197346118e5ae0cd469c3860e9258682a8993c47d1e18f81f96c
binder.sum|5245534f4c56454e542d5445524d0001040101013001010131010c001a01010200010203|119371ceda9619d2e02a00cf3cd41a9b739892883c4cb4578e4729178a106603
binder.product|5245534f4c56454e542d5445524d0001040101013001010131010c001a02010200010203|a2e2a4e55b308eaef8a4502790e320db438a43dc9cce00dacc22bfb9c5366f03
binder.integral|5245534f4c56454e542d5445524d0001040101013001010131010c001a03010200010203|c4271080e57e16716fdea8d2a32b41298d7ce6a0ac6cccb77d0d777a84e8426a
binder.limit|5245534f4c56454e542d5445524d00010301010130010c001a040101000102|9e435c75acdedab24f97f950217904ceddbb33451d0c12544ab6d17331a962d9
binder.local|5245534f4c56454e542d5445524d00010301010130010c001a050101000102|aafcbb08a1407dcc889a0ac7d86fecd8bd2ad1ef2ed818240bf517152fcaff6f
node.held|5245534f4c56454e542d5445524d000102010101311b0001|6a80ddf7644597cbc63514d59e69fcb2f5d0967598b40d4fc08df063a23f39a4
"#;
    assert_eq!(actual, EXPECTED);
}

#[test]
fn decimals_preserve_exact_intent_without_machine_float_ingress() {
    assert_eq!(
        ExactDecimal::parse("0.1").unwrap(),
        ExactDecimal::parse("0.10").unwrap()
    );
    assert_eq!(
        ExactDecimal::parse("0.1").unwrap(),
        ExactDecimal::parse("1e-1").unwrap()
    );
    assert_eq!(ExactDecimal::parse("1000").unwrap().coefficient(), "1");
    assert_eq!(ExactDecimal::parse("1000").unwrap().scale(), -3);
    assert!(ExactDecimal::parse("NaN").is_err());
    assert!(Atom::integer("+1").is_err());
    assert!(Atom::integer("01").is_err());

    let mut store = TermStore::new().unwrap();
    let decimal = store
        .atom(
            Atom::ExactDecimal(ExactDecimal::parse("0.1").unwrap()),
            budget(),
        )
        .unwrap();
    let ieee = store
        .atom(Atom::ExactIeee754Bits(0x3fb9_9999_9999_999a), budget())
        .unwrap();
    let approximate = store
        .atom(Atom::MachineFloatBits(0x3fb9_9999_9999_999a), budget())
        .unwrap();
    assert_ne!(
        store.digest(decimal, budget()).unwrap(),
        store.digest(ieee, budget()).unwrap()
    );
    assert_ne!(
        store.digest(ieee, budget()).unwrap(),
        store.digest(approximate, budget()).unwrap()
    );
}

#[test]
fn structural_identity_preserves_order_nesting_and_non_cancellation() {
    let mut store = TermStore::new().unwrap();
    let add = symbol(&mut store, "System`", "Add");
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let zero = store.atom(Atom::integer("0").unwrap(), budget()).unwrap();
    let xy = apply(&mut store, add, vec![x, y]);
    let yx = apply(&mut store, add, vec![y, x]);
    let nested = apply(&mut store, add, vec![xy, zero]);
    let flat = apply(&mut store, add, vec![x, y, zero]);
    let repeated = apply(&mut store, add, vec![x, y]);

    assert_eq!(xy, repeated, "exact structure must hash-cons");
    assert_ne!(
        store.digest(xy, budget()).unwrap(),
        store.digest(yx, budget()).unwrap()
    );
    assert_ne!(
        store.digest(nested, budget()).unwrap(),
        store.digest(flat, budget()).unwrap()
    );
    assert_ne!(
        store.digest(xy, budget()).unwrap(),
        store.digest(zero, budget()).unwrap()
    );
}

#[test]
fn independent_stores_and_insertion_noise_have_identical_stable_identity() {
    fn build(noise_first: bool) -> (TermStore, resolvent::TermId) {
        let mut store = TermStore::new().unwrap();
        if noise_first {
            symbol(&mut store, "Noise`", "unused");
        }
        let mul = symbol(&mut store, "System`", "Mul");
        let a = symbol(&mut store, "Global`", "a");
        let b = symbol(&mut store, "Global`", "b");
        let root = apply(&mut store, mul, vec![a, b]);
        if !noise_first {
            symbol(&mut store, "Noise`", "unused");
        }
        (store, root)
    }
    let (first, first_root) = build(false);
    let (second, second_root) = build(true);
    assert_eq!(
        first.canonical_bytes(first_root, budget()).unwrap(),
        second.canonical_bytes(second_root, budget()).unwrap()
    );
    assert_eq!(
        first.digest(first_root, budget()).unwrap(),
        second.digest(second_root, budget()).unwrap()
    );
}

#[test]
fn reachable_subtree_insertion_permutations_preserve_fixed_ordered_root_identity() {
    fn build(reverse: bool) -> (TermStore, resolvent::TermId) {
        let mut store = TermStore::new().unwrap();
        let pair = symbol(&mut store, "System`", "Pair");
        let shared = symbol(&mut store, "Global`", "shared");
        let left_head = symbol(&mut store, "System`", "Left");
        let right_head = symbol(&mut store, "System`", "Right");
        let (left, right) = if reverse {
            let right = apply(&mut store, right_head, vec![shared]);
            let left = apply(&mut store, left_head, vec![shared]);
            (left, right)
        } else {
            let left = apply(&mut store, left_head, vec![shared]);
            let right = apply(&mut store, right_head, vec![shared]);
            (left, right)
        };
        let root = apply(&mut store, pair, vec![left, right]);
        (store, root)
    }

    let (forward, forward_root) = build(false);
    let (reverse, reverse_root) = build(true);
    assert_eq!(
        forward.canonical_bytes(forward_root, budget()).unwrap(),
        reverse.canonical_bytes(reverse_root, budget()).unwrap()
    );
    assert_eq!(
        forward.digest(forward_root, budget()).unwrap(),
        reverse.digest(reverse_root, budget()).unwrap()
    );
}

#[test]
fn construction_depth_and_exact_node_caps_apply_after_hash_cons_lookup() {
    let mut store = TermStore::new().unwrap();
    let atom = store
        .atom(
            Atom::Boolean(true),
            TermBudget {
                max_nodes: 1,
                max_depth: 1,
                ..budget()
            },
        )
        .unwrap();
    let at_cap = TermBudget {
        max_nodes: 1,
        max_depth: 1,
        ..budget()
    };
    assert_eq!(store.atom(Atom::Boolean(true), at_cap).unwrap(), atom);
    assert_eq!(
        store
            .intern(TermNode::Held { expression: atom }, at_cap)
            .unwrap_err(),
        TermError::BudgetExceeded {
            resource: "depth",
            limit: 1,
        }
    );
    assert_eq!(store.len(), 1);

    let mut source = TermStore::new().unwrap();
    let source_atom = source.atom(Atom::Boolean(true), budget()).unwrap();
    let imported = store.import(&source, source_atom, at_cap).unwrap();
    assert_eq!(imported, atom, "all-duplicate import must fit at the cap");
    assert_eq!(store.len(), 1);

    let source_root = source
        .intern(
            TermNode::Held {
                expression: source_atom,
            },
            budget(),
        )
        .unwrap();
    let imported_root = store
        .import(
            &source,
            source_root,
            TermBudget {
                max_nodes: 2,
                max_depth: 2,
                ..budget()
            },
        )
        .unwrap();
    assert_eq!(store.len(), 2, "only the non-duplicate node is charged");
    assert_eq!(
        store.node(imported_root).unwrap(),
        TermNode::Held { expression: atom }
    );
}

#[test]
fn de_bruijn_binders_support_outer_references_and_refuse_escaping_roots() {
    let mut store = TermStore::new().unwrap();
    let pair = symbol(&mut store, "System`", "Pair");
    let nearest = store.atom(Atom::BoundVariable(0), budget()).unwrap();
    let outer = store.atom(Atom::BoundVariable(1), budget()).unwrap();
    let body = apply(&mut store, pair, vec![outer, nearest]);
    let inner = store
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
    assert!(store.canonical_bytes(inner, budget()).is_err());
    let closed = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Lambda,
                variable_count: 1,
                bounds: vec![],
                body: inner,
            },
            budget(),
        )
        .unwrap();
    store.canonical_bytes(closed, budget()).unwrap();

    let one = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
    let invalid_bound = store
        .intern(
            TermNode::Binder {
                kind: BinderKind::Sum,
                variable_count: 1,
                bounds: vec![nearest, one],
                body: nearest,
            },
            budget(),
        )
        .unwrap();
    assert!(matches!(
        store.canonical_bytes(invalid_bound, budget()),
        Err(TermError::InvalidBinder(_))
    ));
    assert!(matches!(
        store.intern(
            TermNode::Binder {
                kind: BinderKind::Integral,
                variable_count: 1,
                bounds: vec![one],
                body: nearest,
            },
            budget()
        ),
        Err(TermError::InvalidBinder("wrong bound arity"))
    ));
}

#[test]
fn generic_node_vocabulary_round_trips_without_serde_or_local_ids() {
    let mut store = TermStore::new().unwrap();
    let f = symbol(&mut store, "Global`", "f");
    let x = symbol(&mut store, "Global`", "x");
    let y = symbol(&mut store, "Global`", "y");
    let truth = store.atom(Atom::Boolean(true), budget()).unwrap();
    let one = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
    let application = apply(&mut store, f, vec![x, y]);
    let relation = store
        .intern(
            TermNode::Relation {
                operator: RelationOperator::Less,
                left: x,
                right: y,
            },
            budget(),
        )
        .unwrap();
    let boolean = store
        .intern(
            TermNode::Boolean {
                operator: BooleanOperator::And,
                arguments: vec![truth, relation],
            },
            budget(),
        )
        .unwrap();
    let condition = store
        .intern(
            TermNode::Condition {
                expression: application,
                condition: boolean,
            },
            budget(),
        )
        .unwrap();
    let piecewise = store
        .intern(
            TermNode::Piecewise {
                cases: vec![PiecewiseCase {
                    value: application,
                    condition: relation,
                }],
                otherwise: Some(one),
            },
            budget(),
        )
        .unwrap();
    let array = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::Array { shape: vec![2] },
                elements: vec![x, y],
            },
            budget(),
        )
        .unwrap();
    let map = store
        .intern(
            TermNode::OrderedMap {
                entries: vec![(x, one), (y, application)],
            },
            budget(),
        )
        .unwrap();
    let index = store
        .intern(
            TermNode::Index {
                target: array,
                indices: vec![one],
            },
            budget(),
        )
        .unwrap();
    let slice = store
        .intern(
            TermNode::Slice {
                target: array,
                start: Some(one),
                end: None,
                step: Some(one),
            },
            budget(),
        )
        .unwrap();
    let rule = store
        .intern(
            TermNode::Rule {
                kind: RuleKind::Immediate,
                pattern: x,
                replacement: y,
                condition: Some(relation),
            },
            budget(),
        )
        .unwrap();
    let held = store
        .intern(TermNode::Held { expression: rule }, budget())
        .unwrap();
    let root = store
        .intern(
            TermNode::Collection {
                kind: CollectionKind::Tuple,
                elements: vec![condition, piecewise, map, index, slice, held],
            },
            budget(),
        )
        .unwrap();
    let bytes = store.canonical_bytes(root, budget()).unwrap();
    let digest = store.digest(root, budget()).unwrap();
    let (decoded, decoded_root) = decode_canonical_term(&bytes, budget()).unwrap();
    assert_eq!(
        decoded.canonical_bytes(decoded_root, budget()).unwrap(),
        bytes
    );
    assert_eq!(decoded.digest(decoded_root, budget()).unwrap(), digest);
}

#[test]
fn cross_store_handles_refuse_and_explicit_import_preserves_identity() {
    let mut source = TermStore::new().unwrap();
    let x = symbol(&mut source, "Global`", "x");
    let held = source
        .intern(TermNode::Held { expression: x }, budget())
        .unwrap();
    let mut target = TermStore::new().unwrap();
    let foreign = target.intern(TermNode::Held { expression: x }, budget());
    assert_eq!(foreign.unwrap_err(), TermError::ForeignTerm);
    let imported = target.import(&source, held, budget()).unwrap();
    assert_eq!(
        source.digest(held, budget()).unwrap(),
        target.digest(imported, budget()).unwrap()
    );
}

#[test]
fn deep_terms_use_iterative_walk_encoding_and_budget_refusal() {
    let mut store = TermStore::new().unwrap();
    let mut root = symbol(&mut store, "Global`", "x");
    for _ in 0..20_000 {
        root = store
            .intern(TermNode::Held { expression: root }, budget())
            .unwrap();
    }
    assert_eq!(store.topological(root, budget()).unwrap().len(), 20_001);
    store.digest(root, budget()).unwrap();
    assert!(matches!(
        store.topological(
            root,
            TermBudget {
                max_depth: 100,
                ..budget()
            }
        ),
        Err(TermError::BudgetExceeded {
            resource: "depth",
            limit: 100
        })
    ));
}

#[test]
fn decoder_fails_closed_for_malformed_noncanonical_and_over_budget_input() {
    let mut store = TermStore::new().unwrap();
    let one = store.atom(Atom::integer("1").unwrap(), budget()).unwrap();
    let bytes = store.canonical_bytes(one, budget()).unwrap();

    assert!(decode_canonical_term(&bytes[..bytes.len() - 1], budget()).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode_canonical_term(&trailing, budget()).is_err());
    assert!(matches!(
        decode_canonical_term(
            &bytes,
            TermBudget {
                max_nodes: 0,
                ..budget()
            }
        ),
        Err(TermError::BudgetExceeded {
            resource: "nodes",
            limit: 0
        })
    ));
    assert!(matches!(
        decode_canonical_term(
            &bytes,
            TermBudget {
                max_wire_bytes: bytes.len() - 1,
                ..budget()
            }
        ),
        Err(TermError::BudgetExceeded {
            resource: "wire bytes",
            ..
        })
    ));

    let header = b"RESOLVENT-TERM\0\x01";
    let mut overlong = header.to_vec();
    overlong.extend([0x81, 0x00]);
    assert!(matches!(
        decode_canonical_term(&overlong, budget()),
        Err(TermError::InvalidWire("non-canonical varint"))
    ));

    let mut unknown = header.to_vec();
    unknown.extend([1, 0xff, 0]);
    assert!(matches!(
        decode_canonical_term(&unknown, budget()),
        Err(TermError::InvalidWire("node tag"))
    ));

    let mut duplicate = header.to_vec();
    duplicate.push(2);
    duplicate.extend([0x01, 0x01, 1, b'1']);
    duplicate.extend([0x01, 0x01, 1, b'1']);
    duplicate.push(1);
    assert_eq!(
        decode_canonical_term(&duplicate, budget()).unwrap_err(),
        TermError::NonCanonicalWire
    );

    let mut overflowing_root = bytes.clone();
    overflowing_root.pop();
    overflowing_root.extend([0xff; 9]);
    overflowing_root.push(0x01);
    assert!(matches!(
        decode_canonical_term(&overflowing_root, budget()),
        Err(TermError::InvalidWire(_))
    ));

    let mut narrow_store = TermStore::new().unwrap();
    let element = narrow_store.atom(Atom::Boolean(true), budget()).unwrap();
    let narrow = TermBudget {
        max_children_per_node: 2,
        ..budget()
    };
    assert_eq!(
        narrow_store
            .intern(
                TermNode::Collection {
                    kind: CollectionKind::Array {
                        shape: vec![1, 1, 1],
                    },
                    elements: vec![element],
                },
                narrow,
            )
            .unwrap_err(),
        TermError::BudgetExceeded {
            resource: "array rank",
            limit: 2,
        }
    );
}

#[test]
fn store_reports_logical_accounting_and_supports_thread_safe_read_sharing() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TermStore>();

    let mut store = TermStore::new().unwrap();
    let x = symbol(&mut store, "Global`", "x");
    let repeated = symbol(&mut store, "Global`", "x");
    assert_eq!(x, repeated);
    let metrics = store.metrics();
    assert_eq!(metrics.terms, 1);
    assert_eq!(metrics.symbols, 1);
    assert_eq!(metrics.logical_bytes, 30);

    fn build(reverse: bool) -> (TermStore, resolvent::TermId) {
        let mut store = TermStore::new().unwrap();
        let (add, x) = if reverse {
            let x = symbol(&mut store, "Global`", "x");
            let add = symbol(&mut store, "System`", "Add");
            (add, x)
        } else {
            let add = symbol(&mut store, "System`", "Add");
            let x = symbol(&mut store, "Global`", "x");
            (add, x)
        };
        let root = apply(&mut store, add, vec![x, x]);
        (store, root)
    }
    let (forward, root) = build(false);
    let (reverse, _) = build(true);
    assert_eq!(forward.metrics(), reverse.metrics());
    assert_eq!(forward.metrics().logical_bytes, 95);

    let mut imported = TermStore::new().unwrap();
    imported.import(&forward, root, budget()).unwrap();
    assert_eq!(imported.metrics(), forward.metrics());
}
