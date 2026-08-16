use super::P1Error;
use super::assembly::{
    EvolutionAssembly, EvolutionClass, MassAssembly, MassInput, ScalarEllipticAssembly,
    ScalarEllipticInput, assemble_mass, assemble_scalar_elliptic,
};
use super::mesh::P1Mesh;
use crate::context::Context;
use crate::discrete::{BasisEvaluation, DiscreteOp, DiscreteProgram, RestrictionDirection};
use crate::form::{Continuity, Field, FieldRole, FormExpr, FormProgram, ValueShape};
use crate::id::{DiscreteProgramId, FieldId, FormId, OperatorId, RefinementId};
use crate::operator::{
    DerivativeCapability, OperatorBlock, OperatorBlockKind, OperatorProgram, OperatorProperty,
    SparsityContract,
};
use crate::refinement::{ArtifactKind, RefinementProvenance, RefinementRecord, RefinementRelation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P1DiscretizationRequest {
    pub form: FormId,
    pub unknown: FieldId,
    pub test: FieldId,
    pub mesh: P1Mesh,
    pub elliptic: ScalarEllipticInput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<MassInput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P1LoweringResult {
    pub form: FormId,
    pub stiffness_program: DiscreteProgramId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_program: Option<DiscreteProgramId>,
    pub operator: OperatorId,
    pub refinement: RefinementId,
    pub elliptic: ScalarEllipticAssembly,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<MassAssembly>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution: Option<EvolutionAssembly>,
}

/// Compile the first concrete continuum vertical. The source form must contain scalar P1 H1
/// unknown/test fields and at least one volume term coupling gradients of those fields.
/// Numerical coefficient/source/boundary data is the discretization binding for that semantic
/// form and is covered by the target refinement digest.
pub fn lower_p1(
    context: &mut Context,
    request: &P1DiscretizationRequest,
) -> Result<P1LoweringResult, P1Error> {
    let form = context
        .form(request.form)
        .cloned()
        .ok_or(P1Error::MissingForm(request.form.0))?;
    validate_form(&form, request.unknown, request.test)?;
    request.mesh.validate()?;

    let source_ref = context.rooted_artifact_ref(ArtifactKind::Form, &form)?;
    let elliptic = assemble_scalar_elliptic(&request.mesh, &request.elliptic)?;

    let stiffness_program = context.insert_discrete(stiffness_program(request));
    let mass = request
        .mass
        .as_ref()
        .map(|input| assemble_mass(&request.mesh, &elliptic.dof_map, input))
        .transpose()?;
    let mass_program = if mass.is_some() {
        Some(context.insert_discrete(mass_program(request)))
    } else {
        None
    };

    let evolution = mass.as_ref().map(|mass| EvolutionAssembly {
        stiffness: elliptic.stiffness_free.clone(),
        mass: mass.mass_free.clone(),
        rhs: elliptic.rhs.clone(),
        class: EvolutionClass::Ode,
    });

    let operator_program = operator_program(request, stiffness_program, mass_program, &elliptic);
    let operator = context.insert_operator(operator_program.clone());

    #[derive(Serialize)]
    struct NumericalRoot<'a> {
        operator: &'a OperatorProgram,
        elliptic: &'a ScalarEllipticAssembly,
        mass: &'a Option<MassAssembly>,
        evolution: &'a Option<EvolutionAssembly>,
    }

    let target_ref = context.rooted_artifact_ref(
        ArtifactKind::OperatorProgram,
        &NumericalRoot {
            operator: &operator_program,
            elliptic: &elliptic,
            mass: &mass,
            evolution: &evolution,
        },
    )?;

    let mut refinement = RefinementRecord::new(
        source_ref,
        target_ref,
        RefinementRelation::Discretization {
            scheme: "conforming continuous Galerkin P1 triangle FEM".into(),
            declared_order: Some(1),
        },
    );
    refinement.assumptions = vec![
        "piecewise-affine scalar H1 field on conforming 2-D triangles".into(),
        "piecewise-constant cell coefficient/capacity data".into(),
        "Dirichlet values are time-constant in the transient vertical".into(),
    ];
    refinement.provenance = RefinementProvenance {
        producer: Some("resolvent::p1".into()),
        producer_version: Some(env!("CARGO_PKG_VERSION").into()),
        parameters: BTreeMap::from([
            (
                "quadrature".into(),
                "exact P1 analytic element integrals".into(),
            ),
            ("assembly".into(), "stable-order deterministic CSR".into()),
        ]),
        ..RefinementProvenance::default()
    };
    let refinement = context.record_refinement(refinement);

    Ok(P1LoweringResult {
        form: request.form,
        stiffness_program,
        mass_program,
        operator,
        refinement,
        elliptic,
        mass,
        evolution,
    })
}

fn validate_form(form: &FormProgram, unknown: FieldId, test: FieldId) -> Result<(), P1Error> {
    let unknown_field = find_field(form, unknown)?;
    let test_field = find_field(form, test)?;
    validate_p1_scalar_h1(unknown_field, false)?;
    validate_p1_scalar_h1(test_field, true)?;

    let has_diffusion_term = form.residual_terms.iter().any(|integral| {
        contains_gradient_of(&integral.integrand, unknown)
            && contains_gradient_of(&integral.integrand, test)
            && matches!(integral.measure, crate::form::Measure::Volume { .. })
    });
    if !has_diffusion_term {
        return Err(P1Error::UnsupportedForm(
            "expected a volume residual term coupling gradients of the unknown and test fields"
                .into(),
        ));
    }
    Ok(())
}

fn find_field(form: &FormProgram, id: FieldId) -> Result<&Field, P1Error> {
    form.fields
        .iter()
        .find(|field| field.id == id)
        .ok_or(P1Error::MissingField(id.0))
}

fn validate_p1_scalar_h1(field: &Field, require_test: bool) -> Result<(), P1Error> {
    if !matches!(field.space.value_shape, ValueShape::Scalar)
        || field.space.order != 1
        || !matches!(field.space.continuity, Continuity::H1)
        || !field.space.family.eq_ignore_ascii_case("lagrange")
    {
        return Err(P1Error::UnsupportedFieldSpace(field.name.clone()));
    }
    if require_test && !matches!(field.role, FieldRole::Test) {
        return Err(P1Error::UnsupportedFieldRole {
            field: field.name.clone(),
            expected: "test".into(),
        });
    }
    if !require_test && !matches!(field.role, FieldRole::Unknown | FieldRole::Trial) {
        return Err(P1Error::UnsupportedFieldRole {
            field: field.name.clone(),
            expected: "unknown or trial".into(),
        });
    }
    Ok(())
}

fn contains_gradient_of(expression: &FormExpr, field: FieldId) -> bool {
    match expression {
        FormExpr::Gradient(inner) => contains_field(inner, field),
        FormExpr::Neg(inner)
        | FormExpr::Divergence(inner)
        | FormExpr::Curl(inner)
        | FormExpr::TimeDerivative(inner)
        | FormExpr::Trace(inner) => contains_gradient_of(inner, field),
        FormExpr::Add(items) | FormExpr::Product(items) => {
            items.iter().any(|item| contains_gradient_of(item, field))
        }
        FormExpr::Inner { left, right } | FormExpr::Contract { left, right } => {
            contains_gradient_of(left, field) || contains_gradient_of(right, field)
        }
        FormExpr::Custom { args, .. } => args.iter().any(|arg| contains_gradient_of(arg, field)),
        FormExpr::Scalar(_) | FormExpr::Field(_) => false,
    }
}

fn contains_field(expression: &FormExpr, field: FieldId) -> bool {
    match expression {
        FormExpr::Field(candidate) => *candidate == field,
        FormExpr::Neg(inner)
        | FormExpr::Gradient(inner)
        | FormExpr::Divergence(inner)
        | FormExpr::Curl(inner)
        | FormExpr::TimeDerivative(inner)
        | FormExpr::Trace(inner) => contains_field(inner, field),
        FormExpr::Add(items) | FormExpr::Product(items) => {
            items.iter().any(|item| contains_field(item, field))
        }
        FormExpr::Inner { left, right } | FormExpr::Contract { left, right } => {
            contains_field(left, field) || contains_field(right, field)
        }
        FormExpr::Custom { args, .. } => args.iter().any(|arg| contains_field(arg, field)),
        FormExpr::Scalar(_) => false,
    }
}

fn stiffness_program(request: &P1DiscretizationRequest) -> DiscreteProgram {
    let mut program = DiscreteProgram {
        name: "p1_scalar_diffusion".into(),
        instructions: Vec::new(),
        outputs: Vec::new(),
        metadata: BTreeMap::from([
            ("scheme".into(), "continuous_galerkin_p1".into()),
            ("topology".into(), "triangle".into()),
            ("assembly".into(), "deterministic_csr".into()),
        ]),
    };
    let field = program.push(DiscreteOp::FieldInput {
        field: request.unknown,
    });
    let element = program.push(DiscreteOp::Restrict {
        input: field,
        field: request.unknown,
        direction: RestrictionDirection::Gather,
    });
    let gradient = program.push(DiscreteOp::Basis {
        input: element,
        field: request.unknown,
        evaluation: BasisEvaluation::Gradient,
        transpose: false,
    });
    let flux = program.push(DiscreteOp::Custom {
        operator: "piecewise_constant_scalar_diffusion".into(),
        inputs: vec![gradient],
        metadata: BTreeMap::new(),
    });
    let weighted = program.push(DiscreteOp::QuadratureWeight {
        input: flux,
        rule: "analytic_p1_triangle".into(),
    });
    let tested = program.push(DiscreteOp::Basis {
        input: weighted,
        field: request.test,
        evaluation: BasisEvaluation::Gradient,
        transpose: true,
    });
    let assembled = program.push(DiscreteOp::Restrict {
        input: tested,
        field: request.test,
        direction: RestrictionDirection::ScatterAdd,
    });
    program.outputs.push(assembled);
    program
}

fn mass_program(request: &P1DiscretizationRequest) -> DiscreteProgram {
    let mut program = DiscreteProgram {
        name: "p1_scalar_mass".into(),
        instructions: Vec::new(),
        outputs: Vec::new(),
        metadata: BTreeMap::from([
            ("scheme".into(), "continuous_galerkin_p1".into()),
            ("topology".into(), "triangle".into()),
        ]),
    };
    let field = program.push(DiscreteOp::FieldInput {
        field: request.unknown,
    });
    let element = program.push(DiscreteOp::Restrict {
        input: field,
        field: request.unknown,
        direction: RestrictionDirection::Gather,
    });
    let value = program.push(DiscreteOp::Basis {
        input: element,
        field: request.unknown,
        evaluation: BasisEvaluation::Value,
        transpose: false,
    });
    let capacity = program.push(DiscreteOp::Custom {
        operator: "piecewise_constant_capacity".into(),
        inputs: vec![value],
        metadata: BTreeMap::new(),
    });
    let weighted = program.push(DiscreteOp::QuadratureWeight {
        input: capacity,
        rule: "analytic_p1_triangle_mass".into(),
    });
    let tested = program.push(DiscreteOp::Basis {
        input: weighted,
        field: request.test,
        evaluation: BasisEvaluation::Value,
        transpose: true,
    });
    let assembled = program.push(DiscreteOp::Restrict {
        input: tested,
        field: request.test,
        direction: RestrictionDirection::ScatterAdd,
    });
    program.outputs.push(assembled);
    program
}

fn operator_program(
    request: &P1DiscretizationRequest,
    stiffness: DiscreteProgramId,
    mass: Option<DiscreteProgramId>,
    assembly: &ScalarEllipticAssembly,
) -> OperatorProgram {
    let mut blocks = vec![OperatorBlock {
        name: "stiffness".into(),
        kind: OperatorBlockKind::Stiffness,
        program: stiffness,
        row_variables: vec![format!("field:{}", request.test.0)],
        column_variables: vec![format!("field:{}", request.unknown.0)],
    }];
    if let Some(mass) = mass {
        blocks.push(OperatorBlock {
            name: "mass".into(),
            kind: OperatorBlockKind::Mass,
            program: mass,
            row_variables: vec![format!("field:{}", request.test.0)],
            column_variables: vec![format!("field:{}", request.unknown.0)],
        });
    }

    let mut properties = vec![
        OperatorProperty::Symmetric,
        OperatorProperty::UnitsConsistent,
    ];
    properties.extend(diffusion_definiteness(request, assembly));

    OperatorProgram {
        name: "p1_scalar_elliptic".into(),
        blocks,
        derivatives: vec![
            DerivativeCapability::AnalyticJacobian,
            DerivativeCapability::Jvp,
            DerivativeCapability::Vjp,
        ],
        properties,
        sparsity: Some(SparsityContract {
            rows: assembly.dof_map.n_free(),
            cols: assembly.dof_map.n_free(),
            block_pattern: Vec::new(),
            note: Some(
                "mesh-fixed P1 vertex adjacency; concrete CSR is in the numerical artifact".into(),
            ),
        }),
        metadata: BTreeMap::from([
            ("reference_backend".into(), "resolvent::p1".into()),
            (
                "rhs_terms".into(),
                "dirichlet_lift+volume_source+neumann_flux".into(),
            ),
        ]),
    }
}

fn diffusion_definiteness(
    request: &P1DiscretizationRequest,
    assembly: &ScalarEllipticAssembly,
) -> Vec<OperatorProperty> {
    let coefficients: Vec<_> = request
        .mesh
        .cells
        .iter()
        .map(|cell| request.elliptic.diffusion.value(cell.region))
        .collect();

    if coefficients
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Vec::new();
    }

    if coefficients.iter().any(|value| *value == 0.0) {
        return vec![OperatorProperty::PositiveSemidefinite];
    }

    if every_mesh_component_is_constrained(&request.mesh, assembly) {
        vec![OperatorProperty::PositiveDefinite]
    } else {
        vec![OperatorProperty::PositiveSemidefinite]
    }
}

fn every_mesh_component_is_constrained(mesh: &P1Mesh, assembly: &ScalarEllipticAssembly) -> bool {
    if mesh.vertices.is_empty() {
        return false;
    }

    let mut parent: Vec<usize> = (0..mesh.vertices.len()).collect();

    fn root(parent: &mut [usize], mut vertex: usize) -> usize {
        while parent[vertex] != vertex {
            let grandparent = parent[parent[vertex]];
            parent[vertex] = grandparent;
            vertex = grandparent;
        }
        vertex
    }

    fn union(parent: &mut [usize], a: usize, b: usize) {
        let root_a = root(parent, a);
        let root_b = root(parent, b);
        if root_a != root_b {
            parent[root_b] = root_a;
        }
    }

    for cell in &mesh.cells {
        union(&mut parent, cell.vertices[0], cell.vertices[1]);
        union(&mut parent, cell.vertices[1], cell.vertices[2]);
    }

    let mut component_has_constraint = BTreeMap::<usize, bool>::new();
    for vertex in 0..mesh.vertices.len() {
        let component = root(&mut parent, vertex);
        component_has_constraint.entry(component).or_insert(false);
    }
    for &(vertex, _) in assembly.dof_map.constrained() {
        let component = root(&mut parent, vertex);
        component_has_constraint.insert(component, true);
    }

    component_has_constraint.values().all(|anchored| *anchored)
}
