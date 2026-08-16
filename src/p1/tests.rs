use super::*;
use crate::Context;
use crate::form::{
    Continuity, Field, FieldRole, FormExpr, FormProgram, FunctionSpace, Integral, Measure,
    ValueShape,
};
use crate::operator::OperatorProperty;
use crate::refinement::RefinementRelation;
use std::collections::BTreeMap;

fn close(actual: f64, expected: f64) {
    let scale = 1.0_f64.max(actual.abs()).max(expected.abs());
    assert!(
        (actual - expected).abs() <= 1.0e-12 * scale,
        "actual={actual:?}, expected={expected:?}"
    );
}

fn triangle_mesh() -> P1Mesh {
    P1Mesh {
        vertices: vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 1.0),
        ],
        cells: vec![Triangle {
            vertices: [0, 1, 2],
            region: 7,
        }],
        boundary_edges: vec![
            BoundaryEdge {
                vertices: [0, 1],
                tag: 10,
            },
            BoundaryEdge {
                vertices: [1, 2],
                tag: 20,
            },
            BoundaryEdge {
                vertices: [2, 0],
                tag: 30,
            },
        ],
    }
}

#[test]
fn stiffness_matches_closed_form_triangle() {
    let assembled = assemble_scalar_elliptic(
        &triangle_mesh(),
        &ScalarEllipticInput {
            diffusion: PiecewiseConstant::uniform(2.0),
            ..ScalarEllipticInput::default()
        },
    )
    .unwrap();
    let dense = assembled.stiffness_full.to_dense();
    let expected = [[2.0, -1.0, -1.0], [-1.0, 1.0, 0.0], [-1.0, 0.0, 1.0]];
    for (actual_row, expected_row) in dense.iter().zip(expected) {
        for (&actual, expected) in actual_row.iter().zip(expected_row) {
            close(actual, expected);
        }
    }
}

#[test]
fn source_neumann_and_dirichlet_lift_share_one_rhs() {
    let mut source = PiecewiseSource::default();
    source.per_region.insert(7, 6.0);
    let mut neumann = BoundaryFlux::default();
    neumann.per_tag.insert(20, 2.0);
    let assembled = assemble_scalar_elliptic(
        &triangle_mesh(),
        &ScalarEllipticInput {
            diffusion: PiecewiseConstant::uniform(2.0),
            source,
            neumann,
            dirichlet: vec![DirichletBoundary {
                tag: 10,
                value: 4.0,
            }],
        },
    )
    .unwrap();

    assert_eq!(assembled.dof_map.n_free(), 1);
    close(assembled.stiffness_free.to_dense()[0][0], 1.0);
    close(assembled.dirichlet_lift[0], 4.0);
    close(assembled.source_free[0], 1.0);
    close(assembled.neumann_free[0], 2.0_f64.sqrt());
    close(assembled.rhs[0], 5.0 + 2.0_f64.sqrt());
}

#[test]
fn consistent_and_lumped_mass_preserve_total_capacity() {
    let mesh = triangle_mesh();
    let dof = DofMap::from_dirichlet(&mesh, &[]).unwrap();
    let consistent = assemble_mass(
        &mesh,
        &dof,
        &MassInput {
            capacity: PiecewiseConstant::uniform(3.0),
            lumping: MassLumping::Consistent,
        },
    )
    .unwrap();
    let lumped = assemble_mass(
        &mesh,
        &dof,
        &MassInput {
            capacity: PiecewiseConstant::uniform(3.0),
            lumping: MassLumping::Lumped,
        },
    )
    .unwrap();

    let total_consistent: f64 = consistent.mass_full.values.iter().sum();
    let total_lumped: f64 = lumped.mass_full.values.iter().sum();
    close(total_consistent, 1.5);
    close(total_lumped, 1.5);
    let dense = lumped.mass_full.to_dense();
    close(dense[0][0], 0.5);
    close(dense[1][1], 0.5);
    close(dense[2][2], 0.5);
}

#[test]
fn evolution_residual_and_shifted_matrix_are_consistent() {
    let mesh = triangle_mesh();
    let elliptic = assemble_scalar_elliptic(
        &mesh,
        &ScalarEllipticInput {
            diffusion: PiecewiseConstant::uniform(2.0),
            ..ScalarEllipticInput::default()
        },
    )
    .unwrap();
    let mass = assemble_mass(
        &mesh,
        &elliptic.dof_map,
        &MassInput {
            capacity: PiecewiseConstant::uniform(3.0),
            lumping: MassLumping::Consistent,
        },
    )
    .unwrap();
    let evolution = EvolutionAssembly {
        stiffness: elliptic.stiffness_free,
        mass: mass.mass_free,
        rhs: elliptic.rhs,
        class: EvolutionClass::Ode,
    };
    let state = [1.0, 2.0, 3.0];
    let rate = [0.5, -0.5, 1.0];
    let residual = evolution.residual(&state, &rate).unwrap();
    let stiffness = evolution.static_jvp(&state).unwrap();
    let mass = evolution.mass_jvp(&rate).unwrap();
    for ((&actual, stiffness), mass) in residual.iter().zip(stiffness).zip(mass) {
        close(actual, stiffness + mass);
    }

    let shifted = evolution.iteration_matrix(10.0, 1.0).unwrap();
    let direction = [0.25, 0.5, -0.25];
    let applied = shifted.apply(&direction).unwrap();
    let expected_mass = evolution.mass_jvp(&direction).unwrap();
    let expected_stiffness = evolution.static_jvp(&direction).unwrap();
    for ((&actual, mass), stiffness) in applied.iter().zip(expected_mass).zip(expected_stiffness) {
        close(actual, 10.0 * mass + stiffness);
    }
}

fn insert_diffusion_form(context: &mut Context) -> (crate::FormId, crate::FieldId, crate::FieldId) {
    let unknown = context.allocate_field_id();
    let test = context.allocate_field_id();
    let space = FunctionSpace {
        family: "Lagrange".into(),
        order: 1,
        continuity: Continuity::H1,
        value_shape: ValueShape::Scalar,
        domain: Some("omega".into()),
    };
    let form = context.insert_form(FormProgram {
        name: "heat".into(),
        fields: vec![
            Field {
                id: unknown,
                name: "temperature".into(),
                role: FieldRole::Unknown,
                space: space.clone(),
                dimension: Some("K".into()),
                metadata: BTreeMap::new(),
            },
            Field {
                id: test,
                name: "test_temperature".into(),
                role: FieldRole::Test,
                space,
                dimension: None,
                metadata: BTreeMap::new(),
            },
        ],
        residual_terms: vec![Integral {
            integrand: FormExpr::Inner {
                left: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(test)))),
                right: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(unknown)))),
            },
            measure: Measure::Volume {
                domain: "omega".into(),
            },
            label: Some("diffusion".into()),
        }],
        boundary_terms: Vec::new(),
        metadata: BTreeMap::new(),
    });
    (form, unknown, test)
}

#[test]
fn compiler_emits_discrete_operator_and_refinement_artifacts() {
    let mut context = Context::new();
    let (form, unknown, test) = insert_diffusion_form(&mut context);

    let result = lower_p1(
        &mut context,
        &P1DiscretizationRequest {
            form,
            unknown,
            test,
            mesh: triangle_mesh(),
            elliptic: ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(4.0),
                dirichlet: vec![DirichletBoundary {
                    tag: 10,
                    value: 0.0,
                }],
                ..ScalarEllipticInput::default()
            },
            mass: Some(MassInput {
                capacity: PiecewiseConstant::uniform(2.0),
                lumping: MassLumping::Consistent,
            }),
        },
    )
    .unwrap();

    assert!(context.discrete(result.stiffness_program).is_some());
    assert!(context.discrete(result.mass_program.unwrap()).is_some());
    let operator = context.operator(result.operator).unwrap();
    assert!(
        operator
            .properties
            .contains(&OperatorProperty::PositiveDefinite)
    );
    let refinement = context.refinement(result.refinement).unwrap();
    assert!(matches!(
        refinement.relation,
        RefinementRelation::Discretization {
            declared_order: Some(1),
            ..
        }
    ));
    assert_eq!(result.evolution.unwrap().class, EvolutionClass::Ode);
}

#[test]
fn pure_neumann_diffusion_is_not_labeled_positive_definite() {
    let mut context = Context::new();
    let (form, unknown, test) = insert_diffusion_form(&mut context);
    let result = lower_p1(
        &mut context,
        &P1DiscretizationRequest {
            form,
            unknown,
            test,
            mesh: triangle_mesh(),
            elliptic: ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(1.0),
                ..ScalarEllipticInput::default()
            },
            mass: None,
        },
    )
    .unwrap();
    let operator = context.operator(result.operator).unwrap();
    assert!(
        operator
            .properties
            .contains(&OperatorProperty::PositiveSemidefinite)
    );
    assert!(
        !operator
            .properties
            .contains(&OperatorProperty::PositiveDefinite)
    );
}
