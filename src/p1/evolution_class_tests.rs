use super::*;
use crate::Context;
use crate::form::{
    Continuity, Field, FieldRole, FormExpr, FormProgram, FunctionSpace, Integral, Measure,
    ValueShape,
};
use std::collections::BTreeMap;

fn form(context: &mut Context) -> (crate::FormId, crate::FieldId, crate::FieldId) {
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
        name: "transient_diffusion".into(),
        fields: vec![
            Field {
                id: unknown,
                name: "u".into(),
                role: FieldRole::Unknown,
                space: space.clone(),
                dimension: None,
                metadata: BTreeMap::new(),
            },
            Field {
                id: test,
                name: "v".into(),
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

fn lower(mesh: P1Mesh, capacity: PiecewiseConstant) -> EvolutionClass {
    let mut context = Context::new();
    let (form, unknown, test) = form(&mut context);
    lower_p1(
        &mut context,
        &P1DiscretizationRequest {
            form,
            unknown,
            test,
            mesh,
            elliptic: ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(1.0),
                ..ScalarEllipticInput::default()
            },
            mass: Some(MassInput {
                capacity,
                lumping: MassLumping::Consistent,
            }),
        },
    )
    .unwrap()
    .evolution
    .unwrap()
    .class
}

fn one_triangle(extra_vertex: bool) -> P1Mesh {
    let mut vertices = vec![
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(0.0, 1.0),
    ];
    if extra_vertex {
        vertices.push(Point2::new(2.0, 2.0));
    }
    P1Mesh {
        vertices,
        cells: vec![Triangle {
            vertices: [0, 1, 2],
            region: 1,
        }],
        boundary_edges: Vec::new(),
    }
}

#[test]
fn strictly_positive_capacity_on_all_free_dofs_is_an_ode() {
    assert_eq!(
        lower(one_triangle(false), PiecewiseConstant::uniform(2.0)),
        EvolutionClass::Ode
    );
}

#[test]
fn zero_capacity_is_not_guessed_to_be_an_ode_or_indexed_dae() {
    assert_eq!(
        lower(one_triangle(false), PiecewiseConstant::uniform(0.0)),
        EvolutionClass::Unclassified
    );
}

#[test]
fn isolated_free_vertex_makes_mass_classification_unresolved() {
    assert_eq!(
        lower(one_triangle(true), PiecewiseConstant::uniform(2.0)),
        EvolutionClass::Unclassified
    );
}
