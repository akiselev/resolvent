use crate::discrete::{BasisEvaluation, DiscreteOp, DiscreteProgram, RestrictionDirection};
use crate::field::FieldRole;
use crate::form::{FormExpr, Integral, Measure};
use crate::id::{DiscreteProgramId, ExprId, FormId, OperatorId};
use crate::operator::{
    DerivativeCapability, OperatorBlock, OperatorBlockKind, OperatorProgram, OperatorProperty,
};
use crate::refinement::{ArtifactKind, RefinementRecord, RefinementRelation};
use crate::{Context, declare_refinement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormCompileOptions {
    pub scheme: String,
    pub declared_order: Option<u8>,
    #[serde(default)]
    pub matrix_free: bool,
}
impl Default for FormCompileOptions {
    fn default() -> Self {
        Self {
            scheme: "galerkin_h1".into(),
            declared_order: Some(1),
            matrix_free: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognizedFormTerm {
    Diffusion { coefficient: Option<ExprId> },
    Mass { coefficient: Option<ExprId> },
    Source { source: ExprId },
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledForm {
    pub discrete: DiscreteProgramId,
    pub operator: OperatorId,
    pub form_to_discrete: RefinementRecord,
    pub discrete_to_operator: RefinementRecord,
    pub recognized: Vec<RecognizedFormTerm>,
}

#[derive(Debug, Error)]
pub enum FormCompileError {
    #[error("form id {0} is absent from context")]
    MissingForm(u32),
    #[error("form must contain one unknown/state field and one test field")]
    FieldRoles,
    #[error("could not declare refinement: {0}")]
    Refinement(String),
}

/// Compile the common scalar H1 patterns into libCEED-style explicit stages. Unknown form
/// terms remain represented as Custom instructions instead of being silently discarded.
pub fn compile_form(
    ctx: &mut Context,
    form_id: FormId,
    options: FormCompileOptions,
) -> Result<CompiledForm, FormCompileError> {
    let form = ctx
        .form(form_id)
        .cloned()
        .ok_or(FormCompileError::MissingForm(form_id.0))?;
    let unknown = form
        .fields
        .iter()
        .find(|f| {
            matches!(
                f.role,
                FieldRole::Unknown | FieldRole::State | FieldRole::Trial
            )
        })
        .ok_or(FormCompileError::FieldRoles)?
        .id;
    let test = form
        .fields
        .iter()
        .find(|f| f.role == FieldRole::Test)
        .ok_or(FormCompileError::FieldRoles)?
        .id;
    let mut program = DiscreteProgram {
        name: format!("{}::discrete", form.name),
        instructions: vec![],
        outputs: vec![],
        metadata: BTreeMap::from([("scheme".into(), options.scheme.clone())]),
    };
    let u = program.push(DiscreteOp::FieldInput { field: unknown });
    let gathered = program.push(DiscreteOp::Restrict {
        input: u,
        field: unknown,
        direction: RestrictionDirection::Gather,
    });
    let mut recognized = vec![];
    let mut contributions = vec![];
    for term in form.residual_terms.iter().chain(form.boundary_terms.iter()) {
        let kind = recognize(term, unknown, test);
        let contribution = match &kind {
            RecognizedFormTerm::Diffusion { coefficient } => {
                let grad = program.push(DiscreteOp::Basis {
                    input: gathered,
                    field: unknown,
                    evaluation: BasisEvaluation::Gradient,
                    transpose: false,
                });
                let point = program.push(DiscreteOp::Pointwise {
                    inputs: vec![grad],
                    expressions: coefficient.iter().copied().collect(),
                });
                let weighted = program.push(DiscreteOp::QuadratureWeight {
                    input: point,
                    rule: "auto".into(),
                });
                program.push(DiscreteOp::Basis {
                    input: weighted,
                    field: test,
                    evaluation: BasisEvaluation::Gradient,
                    transpose: true,
                })
            }
            RecognizedFormTerm::Mass { coefficient } => {
                let value = program.push(DiscreteOp::Basis {
                    input: gathered,
                    field: unknown,
                    evaluation: BasisEvaluation::Value,
                    transpose: false,
                });
                let point = program.push(DiscreteOp::Pointwise {
                    inputs: vec![value],
                    expressions: coefficient.iter().copied().collect(),
                });
                let weighted = program.push(DiscreteOp::QuadratureWeight {
                    input: point,
                    rule: "auto".into(),
                });
                program.push(DiscreteOp::Basis {
                    input: weighted,
                    field: test,
                    evaluation: BasisEvaluation::Value,
                    transpose: true,
                })
            }
            RecognizedFormTerm::Source { source } => program.push(DiscreteOp::Custom {
                operator: "source_integral".into(),
                inputs: vec![],
                metadata: BTreeMap::from([
                    ("expr".into(), source.0.to_string()),
                    ("measure".into(), measure_name(&term.measure)),
                ]),
            }),
            RecognizedFormTerm::Custom => program.push(DiscreteOp::Custom {
                operator: "unlowered_form_term".into(),
                inputs: vec![],
                metadata: BTreeMap::from([(
                    "label".into(),
                    term.label.clone().unwrap_or_default(),
                )]),
            }),
        };
        recognized.push(kind);
        contributions.push(contribution);
    }
    let sum = if contributions.len() == 1 {
        contributions[0]
    } else {
        program.push(DiscreteOp::Sum {
            inputs: contributions,
        })
    };
    let output = program.push(DiscreteOp::Restrict {
        input: sum,
        field: test,
        direction: RestrictionDirection::ScatterAdd,
    });
    program.outputs.push(output);
    let discrete_ref = declare_refinement(
        ArtifactKind::Form,
        &form,
        ArtifactKind::DiscreteProgram,
        &program,
        RefinementRelation::Discretization {
            scheme: options.scheme.clone(),
            declared_order: options.declared_order,
        },
    )
    .map_err(|e| FormCompileError::Refinement(e.to_string()))?;
    let discrete = ctx.insert_discrete(program.clone());
    let operator = OperatorProgram {
        name: format!("{}::operator", form.name),
        blocks: vec![OperatorBlock {
            name: "residual".into(),
            kind: OperatorBlockKind::Residual,
            program: discrete,
            row_variables: vec![test.0.to_string()],
            column_variables: vec![unknown.0.to_string()],
        }],
        derivatives: vec![
            DerivativeCapability::Jvp,
            DerivativeCapability::Vjp,
            DerivativeCapability::ParameterDerivative,
        ],
        properties: derive_properties(&recognized),
        sparsity: None,
        metadata: BTreeMap::from([(
            "realization".into(),
            if options.matrix_free {
                "matrix_free_capable"
            } else {
                "assembled"
            }
            .into(),
        )]),
    };
    let operator_ref = declare_refinement(
        ArtifactKind::DiscreteProgram,
        &program,
        ArtifactKind::OperatorProgram,
        &operator,
        RefinementRelation::AlgebraicImplementation,
    )
    .map_err(|e| FormCompileError::Refinement(e.to_string()))?;
    let operator_id = ctx.insert_operator(operator);
    ctx.record_refinement(discrete_ref.clone());
    ctx.record_refinement(operator_ref.clone());
    Ok(CompiledForm {
        discrete,
        operator: operator_id,
        form_to_discrete: discrete_ref,
        discrete_to_operator: operator_ref,
        recognized,
    })
}

fn derive_properties(terms: &[RecognizedFormTerm]) -> Vec<OperatorProperty> {
    let mut p = vec![OperatorProperty::UnitsConsistent];
    if terms.iter().all(|t| {
        matches!(
            t,
            RecognizedFormTerm::Diffusion { .. }
                | RecognizedFormTerm::Mass { .. }
                | RecognizedFormTerm::Source { .. }
        )
    }) {
        p.push(OperatorProperty::Symmetric)
    }
    p
}
fn measure_name(m: &Measure) -> String {
    match m {
        Measure::Volume { domain } => format!("volume:{domain}"),
        Measure::Boundary { boundary } => format!("boundary:{boundary}"),
        Measure::Interface { interface } => format!("interface:{interface}"),
        Measure::Point { set } => format!("point:{set}"),
    }
}
fn recognize(
    term: &Integral,
    unknown: crate::id::FieldId,
    test: crate::id::FieldId,
) -> RecognizedFormTerm {
    if let Some(c) = diffusion(&term.integrand, unknown, test) {
        return RecognizedFormTerm::Diffusion { coefficient: c };
    }
    if let Some(c) = mass(&term.integrand, unknown, test) {
        return RecognizedFormTerm::Mass { coefficient: c };
    }
    if let FormExpr::Product(xs) = &term.integrand {
        if xs.len() == 2 {
            if let (FormExpr::Scalar(s), FormExpr::Field(f)) = (&xs[0], &xs[1]) {
                if *f == test {
                    return RecognizedFormTerm::Source { source: *s };
                }
            }
        }
    }
    RecognizedFormTerm::Custom
}
fn diffusion(e: &FormExpr, u: crate::id::FieldId, v: crate::id::FieldId) -> Option<Option<ExprId>> {
    match e {
        FormExpr::Inner { left, right } if grad_field(left, u) && grad_field(right, v) => {
            Some(None)
        }
        FormExpr::Product(xs) if xs.len() == 2 => match (&xs[0], &xs[1]) {
            (FormExpr::Scalar(c), FormExpr::Inner { left, right })
                if grad_field(left, u) && grad_field(right, v) =>
            {
                Some(Some(*c))
            }
            _ => None,
        },
        _ => None,
    }
}
fn mass(e: &FormExpr, u: crate::id::FieldId, v: crate::id::FieldId) -> Option<Option<ExprId>> {
    match e {
        FormExpr::Product(xs) if xs.len() == 2 && is_field(&xs[0], u) && is_field(&xs[1], v) => {
            Some(None)
        }
        FormExpr::Product(xs) if xs.len() == 3 => {
            let mut coeff = None;
            let mut has_u = false;
            let mut has_v = false;
            for x in xs {
                match x {
                    FormExpr::Scalar(c) => coeff = Some(*c),
                    FormExpr::Field(f) if *f == u => has_u = true,
                    FormExpr::Field(f) if *f == v => has_v = true,
                    _ => {}
                }
            }
            if has_u && has_v { Some(coeff) } else { None }
        }
        _ => None,
    }
}
fn grad_field(e: &FormExpr, f: crate::id::FieldId) -> bool {
    matches!(e,FormExpr::Gradient(x) if is_field(x,f))
}
fn is_field(e: &FormExpr, f: crate::id::FieldId) -> bool {
    matches!(e,FormExpr::Field(x) if *x==f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{Field, FieldRole, FunctionSpace};
    use crate::form::{FormExpr, FormProgram, Integral, Measure};
    #[test]
    fn diffusion_becomes_explicit_discrete_stages() {
        let mut c = Context::new();
        let u = c.allocate_field_id();
        let v = c.allocate_field_id();
        let coeff = c.exprs.literal(crate::ScalarLiteral::integer(2));
        let fields = vec![
            Field {
                id: u,
                name: "u".into(),
                role: FieldRole::State,
                space: FunctionSpace::h1_lagrange(1, "Omega"),
                dimension: None,
                metadata: BTreeMap::new(),
            },
            Field {
                id: v,
                name: "v".into(),
                role: FieldRole::Test,
                space: FunctionSpace::h1_lagrange(1, "Omega"),
                dimension: None,
                metadata: BTreeMap::new(),
            },
        ];
        let form = FormProgram {
            name: "poisson".into(),
            fields,
            residual_terms: vec![Integral {
                integrand: FormExpr::Product(vec![
                    FormExpr::Scalar(coeff),
                    FormExpr::Inner {
                        left: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(u)))),
                        right: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(v)))),
                    },
                ]),
                measure: Measure::Volume {
                    domain: "Omega".into(),
                },
                label: None,
            }],
            boundary_terms: vec![],
            essential_boundaries: vec![],
            natural_boundaries: vec![],
            robin_boundaries: vec![],
            metadata: BTreeMap::new(),
        };
        let id = c.insert_form(form);
        let out = compile_form(&mut c, id, Default::default()).unwrap();
        assert!(matches!(
            out.recognized[0],
            RecognizedFormTerm::Diffusion {
                coefficient: Some(_)
            }
        ));
        assert!(c.discrete(out.discrete).unwrap().instructions.len() >= 7);
        assert!(
            c.operator(out.operator)
                .unwrap()
                .derivatives
                .contains(&DerivativeCapability::Vjp)
        );
    }
}
