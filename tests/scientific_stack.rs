use resolvent::*;

#[test]
fn expression_store_hash_conses_commutative_builders() {
    let mut ctx = Context::new();
    let x = ctx.declare_symbol(Symbol {
        name: "x".into(),
        role: SymbolRole::State,
        dimension: None,
    });
    let y = ctx.declare_symbol(Symbol {
        name: "y".into(),
        role: SymbolRole::State,
        dimension: None,
    });
    let x = ctx.exprs.symbol(x);
    let y = ctx.exprs.symbol(y);
    let xy = ctx.exprs.add([x, y]);
    let yx = ctx.exprs.add([y, x]);
    assert_eq!(xy, yx);
}

#[test]
fn scope_broadening_requires_named_obligation() {
    let source = ArtifactRef::of(ArtifactKind::System, &"restricted").unwrap();
    let target = ArtifactRef::of(ArtifactKind::System, &"global").unwrap();
    let mut refinement =
        RefinementRecord::new(source, target, RefinementRelation::LogicalConsequence);
    refinement.scope_transition = ScopeTransition::Broadened {
        obligation: ObligationId(7),
        reason: "restricted orbit family -> full orbit space".into(),
    };
    assert!(matches!(
        refinement.validate(),
        Err(RefinementError::MissingScopeObligation(ObligationId(7)))
    ));

    refinement.obligations.push(Obligation::open(
        ObligationId(7),
        "prove transport from restricted family to full orbit space",
    ));
    assert!(refinement.validate().is_ok());
    assert!(!refinement.is_promotion_ready());
}

#[test]
fn evidence_axes_do_not_collapse() {
    let items = vec![
        EvidenceItem {
            grade: EvidenceGrade::Formal(FormalGrade::KernelProved),
            claim: "the transformation is a theorem".into(),
            artifacts: vec![],
            limitations: vec![],
            metadata: Default::default(),
        },
        EvidenceItem {
            grade: EvidenceGrade::Empirical(EmpiricalGrade::NoData),
            claim: "no physical validation data are attached".into(),
            artifacts: vec![],
            limitations: vec![],
            metadata: Default::default(),
        },
    ];
    let profile = EvidenceProfile::from_items(&items);
    assert_eq!(profile.formal, Some(FormalGrade::KernelProved));
    assert_eq!(profile.empirical, Some(EmpiricalGrade::NoData));
    assert_eq!(profile.numerical, None);
}

#[test]
fn structural_projection_uses_common_system_ir() {
    let mut ctx = Context::new();
    let x = ctx.declare_symbol(Symbol {
        name: "x".into(),
        role: SymbolRole::Algebraic,
        dimension: None,
    });
    let y = ctx.declare_symbol(Symbol {
        name: "y".into(),
        role: SymbolRole::Algebraic,
        dimension: None,
    });
    let ex = ctx.exprs.symbol(x);
    let ey = ctx.exprs.symbol(y);
    let zero = ctx.exprs.literal(ScalarLiteral::integer(0));
    let sum = ctx.exprs.add([ex, ey]);
    let system = System {
        name: "square".into(),
        unknowns: vec![x, y],
        parameters: vec![],
        equations: vec![
            Equation {
                lhs: ex,
                rhs: zero,
                label: None,
            },
            Equation {
                lhs: sum,
                rhs: zero,
                label: None,
            },
        ],
        events: vec![],
        children: vec![],
        metadata: Default::default(),
    };
    let incidence = IncidenceSystem::from_system(&system, &ctx.exprs).unwrap();
    assert_eq!(incidence.rows, vec![vec![0], vec![0, 1]]);
    assert!(maximum_matching(&incidence).is_perfect());
}

#[test]
fn kernel_proved_reification_requires_theorem_and_axiom_whitelist() {
    let artifact = ArtifactRef::of(ArtifactKind::ScientificSpec, &"heat").unwrap();
    let declaration = LeanDeclaration {
        module: "Physics.Heat".into(),
        name: "heatSpec".into(),
        statement_digest: Digest::blake3(b"statement"),
        axioms: vec!["Classical.choice".into()],
        source_commit: None,
        toolchain: None,
    };
    let receipt = ReificationReceipt {
        schema_version: "resolvent-lean/0.1".into(),
        declaration,
        artifact,
        grade: FormalGrade::KernelProved,
        soundness_theorem: Some("heatSpec_sound".into()),
        assumptions: vec![],
    };
    let manifest = LeanExportManifest {
        schema_version: "resolvent-lean/0.1".into(),
        receipts: vec![receipt],
        checker_axiom_whitelist: vec!["Classical.choice".into()],
    };
    assert!(manifest.validate().is_ok());
}
