use resolvent::quantities::{Dimension, QuantityKindId};
use resolvent::scientific::{
    DerivativeContract, FrameSemantics, OutOfValidityPolicy, PropertyDomain, PropertyEvidence,
    PropertyInput, PropertyLocality, PropertyModel, PropertyOutput, PropertySignature,
    TensorSymmetry, UncertaintyModel, ValueShapeV1,
};
use resolvent::{
    PropertyDefinition, ScientificPhysicsLock, freeze_scientific, parse_scientific_module,
    semantic_digest,
};
use std::collections::BTreeMap;

resolvent::include_scientific!(pub Embedded = "fixtures/scientific_macro.res");

#[test]
fn file_and_rust_macro_use_identical_scientific_semantics() {
    let direct = parse_scientific_module(Embedded::SOURCE).unwrap();
    let embedded = Embedded::parse().unwrap();
    assert_eq!(semantic_digest(&direct), semantic_digest(&embedded));
    assert_eq!(
        Embedded::semantic_digest().unwrap(),
        semantic_digest(&direct)
    );
    let lock = Embedded::freeze().unwrap();
    assert_eq!(lock.semantic_digest, semantic_digest(&direct));
}

fn property(dataset_digest: &str) -> PropertyDefinition {
    PropertyDefinition {
        signature: PropertySignature {
            id: "thermal.conductivity".into(),
            inputs: vec![PropertyInput {
                name: "T".into(),
                quantity_kind: QuantityKindId("ThermodynamicTemperature".into()),
                dimension: Dimension::TEMPERATURE,
                shape: ValueShapeV1::Scalar,
                physical_min: Some(0.0),
                physical_max: None,
                nominal: None,
            }],
            output: PropertyOutput {
                quantity_kind: QuantityKindId("ThermalConductivity".into()),
                dimension: Dimension::DIMENSIONLESS,
                shape: ValueShapeV1::Scalar,
                symmetry: TensorSymmetry::None,
                frame: FrameSemantics::Scalar,
            },
            locality: PropertyLocality::Pointwise,
            differentiability: DerivativeContract::Symbolic,
        },
        model: PropertyModel::Expression(resolvent::scientific::Expr::Name("T".into())),
        domain: PropertyDomain {
            physical_bounds: vec![],
            validity_bounds: vec![],
            phase_constraints: vec![],
            composition_constraints: vec![],
            assumptions: vec![],
            out_of_validity: OutOfValidityPolicy::Error,
        },
        evidence: PropertyEvidence {
            sources: vec!["doi:10.example/thermal".into()],
            dataset_digest: Some(dataset_digest.into()),
            fit_digest: Some("fit-sha256:abc".into()),
            uncertainty: Some(UncertaintyModel::StandardRelative(0.02)),
            notes: BTreeMap::from([("method".into(), "guarded fit".into())]),
        },
    }
}

#[test]
fn scientific_physics_lock_carries_property_provenance_and_digest_identity() {
    let module = Embedded::parse().unwrap();
    let a: ScientificPhysicsLock =
        freeze_scientific(Embedded::SOURCE, &module, &[property("data-a")]);
    let b = freeze_scientific(Embedded::SOURCE, &module, &[property("data-b")]);
    assert_eq!(a.semantic_digest, b.semantic_digest);
    assert_ne!(a.digest, b.digest);
    assert_eq!(
        a.property_evidence[0].dataset_digest.as_deref(),
        Some("data-a")
    );
    assert_eq!(
        a.property_evidence[0].sources,
        vec!["doi:10.example/thermal"]
    );
    assert!(a.property_evidence[0].uncertainty_digest.is_some());
}
