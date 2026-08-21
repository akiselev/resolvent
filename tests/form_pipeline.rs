use quantitas::UnitRegistry;
use resolvent::{
    FormArgumentRole, InputEvaluation, LocalInputRole, LocalIterationContract, RegionKind,
    SemanticMeasure, compile_semantics, compile_variational_form, factor_local_integral,
    lower_local_program, semantic_arena_digest,
};

const FORM_SOURCE: &str = r#"
module pipeline.test;
model Diffusion {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: trial scalar H1(order=2) on Omega;
  field v: test scalar H1(order=2) on Omega;
  field conductivity: coefficient scalar L2(order=1) on Omega;
  parameter reaction: Rate;
  form residual {
    cell(Omega): conductivity * dot(grad(u), grad(v));
    cell(Omega): reaction * u * v;
  }
}
"#;

#[test]
fn typed_identities_and_roles_survive_form_factorization() {
    let compilation = compile_semantics(FORM_SOURCE, &UnitRegistry::si_bootstrap()).unwrap();
    let form = compile_variational_form(&compilation.semantic, "Diffusion", "residual").unwrap();

    assert_eq!(
        form.source_semantic_digest.hex,
        semantic_arena_digest(&compilation.semantic)
    );
    assert_eq!(form.arguments[0].role, FormArgumentRole::Trial);
    assert_eq!(form.arguments[1].role, FormArgumentRole::Test);
    assert!(matches!(
        form.integrals[0].measure,
        SemanticMeasure::Cell { .. }
    ));

    let diffusion = factor_local_integral(&form, 0).unwrap();
    assert_eq!(diffusion.iteration, LocalIterationContract::QuadraturePoint);
    assert_eq!(diffusion.source_form_digest, form.artifact_digest);
    assert_eq!(diffusion.receipt.source_form_digest, form.artifact_digest);
    assert_eq!(diffusion.inputs[0].role, LocalInputRole::TrialBasis);
    assert_eq!(diffusion.inputs[1].role, LocalInputRole::TestBasis);
    assert_eq!(
        diffusion.inputs[2].role,
        LocalInputRole::PhysicalField(resolvent::scientific::FieldRole::Coefficient)
    );
    assert!(
        diffusion.inputs[..2]
            .iter()
            .all(|input| input.evaluations == [InputEvaluation::Gradient])
    );

    let reaction = factor_local_integral(&form, 1).unwrap();
    let lowered = lower_local_program(&reaction).unwrap();
    assert_eq!(
        lowered.receipt.source_program_digest,
        reaction.artifact_digest
    );
    assert_eq!(
        lowered.receipt.iteration,
        LocalIterationContract::QuadraturePoint
    );
    malleus::validate(lowered.kernel).unwrap();
}

#[test]
fn presentation_changes_do_not_change_form_or_program_digests() {
    let registry = UnitRegistry::si_bootstrap();
    let compact = compile_semantics(FORM_SOURCE, &registry).unwrap();
    let formatted = resolvent::format_scientific_module(&compact.source);
    let formatted = compile_semantics(&formatted, &registry).unwrap();
    let first = compile_variational_form(&compact.semantic, "Diffusion", "residual").unwrap();
    let second = compile_variational_form(&formatted.semantic, "Diffusion", "residual").unwrap();

    assert_eq!(first.artifact_digest, second.artifact_digest);
    assert_eq!(
        factor_local_integral(&first, 0).unwrap().artifact_digest,
        factor_local_integral(&second, 0).unwrap().artifact_digest
    );
}

#[test]
fn facet_measures_resolve_to_region_ids() {
    let compilation = compile_semantics(
        r#"
module facet.test;
model Flux {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: trial scalar H1(order=1) on Omega;
  field v: test scalar H1(order=1) on Omega;
  form boundary_residual { boundary(walls): u * v; }
}
"#,
        &UnitRegistry::si_bootstrap(),
    )
    .unwrap();
    let model = &compilation.semantic.models[0];
    let form =
        compile_variational_form(&compilation.semantic, "Flux", "boundary_residual").unwrap();
    let SemanticMeasure::ExteriorFacet { region } = form.integrals[0].measure else {
        panic!("boundary measure did not retain a typed exterior-facet region")
    };

    assert_eq!(model.regions[region.index()].name, "walls");
    assert_eq!(
        model.regions[region.index()].kind,
        RegionKind::ExteriorFacet
    );
}
