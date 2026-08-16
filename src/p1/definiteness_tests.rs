use super::*;
use crate::Context;
use crate::form::{
    Continuity, Field, FieldRole, FormExpr, FormProgram, FunctionSpace, Integral, Measure,
    ValueShape,
};
use crate::operator::OperatorProperty;
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
        name: "diffusion".into(),
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
            label: None,
        }],
        boundary_terms: Vec::new(),
        metadata: BTreeMap::new(),
    });
    (form, unknown, test)
}

fn properties(
    mesh: P1Mesh,
    diffusion: PiecewiseConstant,
    dirichlet: Vec<DirichletBoundary>,
) -> Vec<OperatorProperty> {
    let mut context = Context::new();
    let (form, unknown, test) = form(&mut context);
    let result = lower_p1(
        &mut context,
        &P1DiscretizationRequest {
            form,
            unknown,
            test,
            mesh,
            elliptic: ScalarEllipticInput {
                diffusion,
                dirichlet,
                ..ScalarEllipticInput::default()
            },
            mass: None,
        },
    )
    .unwrap();
    context.operator(result.operator).unwrap().properties.clone()
}

#[test]
fn negative_diffusion_gets_no_positive_definiteness_claim() {
    let mesh = P1Mesh {
        vertices: vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 1.0),
        ],
        cells: vec![Triangle {
            vertices: [0, 1, 2],
            region: 1,
        }],
        boundary_edges: vec![BoundaryEdge {
            vertices: [0, 1],
            tag: 10,
        }],
    };
    let properties = properties(
        mesh,
        PiecewiseConstant::uniform(-1.0),
        vec![DirichletBoundary {
            tag: 10,
            value: 0.0,
        }],
    );
    assert!(!properties.contains(&OperatorProperty::PositiveDefinite));
    assert!(!properties.contains(&OperatorProperty::PositiveSemidefinite));
}

#[test]
fn disconnected_unanchored_component_prevents_spd_claim() {
    let mesh = P1Mesh {
        vertices: vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 1.0),
            Point2::new(3.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(3.0, 1.0),
        ],
        cells: vec![
            Triangle {
                vertices: [0, 1, 2],
                region: 1,
            },
            Triangle {
                vertices: [3, 4, 5],
                region: 1,
            },
        ],
        boundary_edges: vec![BoundaryEdge {
            vertices: [0, 1],
            tag: 10,
        }],
    };
    let properties = properties(
        mesh,
        PiecewiseConstant::uniform(1.0),
        vec![DirichletBoundary {
            tag: 10,
            value: 0.0,
        }],
    );
    assert!(properties.contains(&OperatorProperty::PositiveSemidefinite));
    assert!(!properties.contains(&OperatorProperty::PositiveDefinite));
}
