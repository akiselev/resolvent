//! Typed variational-form artifacts projected from the canonical semantic arena.

use crate::id::{Digest, span_independent_digest};
use crate::scientific::{FieldRole, SpaceSpec};
use crate::semantic::{
    DeclarationId, DomainId, ExprId, SemanticDeclarationKind, SemanticExpr, SemanticExprKind,
    SemanticMeasure, SemanticModel, SemanticModule, SemanticRole, SemanticType, SymbolId,
    semantic_arena_digest,
};
use crate::source::SourceSpan;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const VARIATIONAL_FORM_SCHEMA: &str = "resolvent-variational-form/1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormArgumentRole {
    Test,
    Trial,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormArgument {
    pub symbol: SymbolId,
    pub role: FormArgumentRole,
    pub ty: SemanticType,
    pub domain: DomainId,
    pub space: SpaceSpec,
    pub source_span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormCaptureRole {
    PhysicalField(FieldRole),
    Parameter,
    Constant,
    Source,
    Property,
    ConstitutiveLaw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormCapture {
    pub symbol: SymbolId,
    pub role: FormCaptureRole,
    pub ty: SemanticType,
    pub domain: Option<DomainId>,
    pub space: Option<SpaceSpec>,
    pub source_span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalIntegral {
    pub measure: SemanticMeasure,
    pub integrand: ExprId,
    pub source_span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormTransformation {
    Authored,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormReceipt {
    pub source_declaration: DeclarationId,
    pub source_span: SourceSpan,
    pub transformations: Vec<FormTransformation>,
    pub assumptions: Vec<String>,
    pub boundary_terms: Vec<ExprId>,
}

/// A typed form retains canonical semantic expressions and resolved identities. Display names are
/// intentionally absent from arguments, captures, measures, and integrands.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalForm {
    pub schema: String,
    pub model: String,
    pub name: String,
    pub source_semantic_digest: Digest,
    pub artifact_digest: Digest,
    pub declaration: DeclarationId,
    pub arguments: Vec<FormArgument>,
    pub captures: Vec<FormCapture>,
    pub expressions: Vec<SemanticExpr>,
    pub integrals: Vec<VariationalIntegral>,
    pub receipt: FormReceipt,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum FormCompileError {
    #[error("semantic module has no model named `{0}`")]
    MissingModel(String),
    #[error("model `{model}` has no form named `{form}`")]
    MissingForm { model: String, form: String },
    #[error("model `{model}` declares form `{form}` more than once")]
    DuplicateForm { model: String, form: String },
    #[error("form `{form}` contains no integrals")]
    EmptyForm { form: String },
    #[error("form `{form}` references symbol {symbol} with invalid role {role:?}")]
    InvalidReferenceRole {
        form: String,
        symbol: SymbolId,
        role: SemanticRole,
    },
    #[error("form argument {symbol} has no resolved domain")]
    ArgumentWithoutDomain { symbol: SymbolId },
    #[error("form argument {symbol} has no function-space declaration")]
    ArgumentWithoutSpace { symbol: SymbolId },
    #[error("semantic expression id {0} is outside the canonical arena")]
    InvalidExpression(ExprId),
    #[error("semantic symbol id {0} is outside the canonical arena")]
    InvalidSymbol(SymbolId),
}

/// Compile one authored form exclusively from typed semantic declarations and identities.
pub fn compile_variational_form(
    module: &SemanticModule,
    model_name: &str,
    form_name: &str,
) -> Result<VariationalForm, FormCompileError> {
    let model = module
        .models
        .iter()
        .find(|model| model.name == model_name)
        .ok_or_else(|| FormCompileError::MissingModel(model_name.to_owned()))?;
    let mut matches = model.declarations.iter().filter(|declaration| {
        declaration.name == form_name
            && matches!(declaration.kind, SemanticDeclarationKind::Form { .. })
    });
    let declaration = matches
        .next()
        .ok_or_else(|| FormCompileError::MissingForm {
            model: model.name.clone(),
            form: form_name.to_owned(),
        })?;
    if matches.next().is_some() {
        return Err(FormCompileError::DuplicateForm {
            model: model.name.clone(),
            form: form_name.to_owned(),
        });
    }
    let SemanticDeclarationKind::Form { integrals } = &declaration.kind else {
        unreachable!("form declaration was selected above")
    };
    if integrals.is_empty() {
        return Err(FormCompileError::EmptyForm {
            form: form_name.to_owned(),
        });
    }

    let mut referenced = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for integral in integrals {
        collect_symbols(model, integral.integrand, &mut visited, &mut referenced)?;
    }

    let mut arguments = vec![];
    let mut captures = vec![];
    for symbol_id in referenced {
        let symbol = model
            .symbols
            .get(symbol_id.index())
            .ok_or(FormCompileError::InvalidSymbol(symbol_id))?;
        match &symbol.ty.role {
            SemanticRole::PhysicalField(FieldRole::Test | FieldRole::Trial) => {
                let role = match &symbol.ty.role {
                    SemanticRole::PhysicalField(FieldRole::Test) => FormArgumentRole::Test,
                    SemanticRole::PhysicalField(FieldRole::Trial) => FormArgumentRole::Trial,
                    _ => unreachable!(),
                };
                arguments.push(FormArgument {
                    symbol: symbol.id,
                    role,
                    ty: symbol.ty.clone(),
                    domain: symbol
                        .domain
                        .ok_or(FormCompileError::ArgumentWithoutDomain { symbol: symbol.id })?,
                    space: symbol
                        .space
                        .clone()
                        .ok_or(FormCompileError::ArgumentWithoutSpace { symbol: symbol.id })?,
                    source_span: symbol.span,
                });
            }
            role => {
                let role =
                    capture_role(role).ok_or_else(|| FormCompileError::InvalidReferenceRole {
                        form: form_name.to_owned(),
                        symbol: symbol.id,
                        role: role.clone(),
                    })?;
                captures.push(FormCapture {
                    symbol: symbol.id,
                    role,
                    ty: symbol.ty.clone(),
                    domain: symbol.domain,
                    space: symbol.space.clone(),
                    source_span: symbol.span,
                });
            }
        }
    }

    let source_semantic_digest = Digest {
        algorithm: "blake3".into(),
        hex: semantic_arena_digest(module),
    };
    let receipt = FormReceipt {
        source_declaration: declaration.id,
        source_span: declaration.span,
        transformations: vec![FormTransformation::Authored],
        assumptions: vec![],
        boundary_terms: vec![],
    };
    let integrals = integrals
        .iter()
        .map(|integral| VariationalIntegral {
            measure: integral.measure.clone(),
            integrand: integral.integrand,
            source_span: integral.span,
        })
        .collect::<Vec<_>>();
    let artifact_digest = span_independent_digest(&FormDigestPayload {
        schema: VARIATIONAL_FORM_SCHEMA,
        model: &model.name,
        name: form_name,
        source_semantic_digest: &source_semantic_digest,
        declaration: declaration.id,
        arguments: &arguments,
        captures: &captures,
        expressions: &model.expressions,
        integrals: &integrals,
        receipt: &receipt,
    });
    Ok(VariationalForm {
        schema: VARIATIONAL_FORM_SCHEMA.into(),
        model: model.name.clone(),
        name: form_name.to_owned(),
        source_semantic_digest,
        artifact_digest,
        declaration: declaration.id,
        arguments,
        captures,
        expressions: model.expressions.clone(),
        integrals,
        receipt,
    })
}

#[derive(Serialize)]
struct FormDigestPayload<'a> {
    schema: &'static str,
    model: &'a str,
    name: &'a str,
    source_semantic_digest: &'a Digest,
    declaration: DeclarationId,
    arguments: &'a [FormArgument],
    captures: &'a [FormCapture],
    expressions: &'a [SemanticExpr],
    integrals: &'a [VariationalIntegral],
    receipt: &'a FormReceipt,
}

fn collect_symbols(
    model: &SemanticModel,
    expression: ExprId,
    visited: &mut BTreeSet<ExprId>,
    symbols: &mut BTreeSet<SymbolId>,
) -> Result<(), FormCompileError> {
    if !visited.insert(expression) {
        return Ok(());
    }
    let expression = model
        .expressions
        .get(expression.index())
        .ok_or(FormCompileError::InvalidExpression(expression))?;
    match &expression.kind {
        SemanticExprKind::Symbol { symbol } => {
            symbols.insert(*symbol);
        }
        SemanticExprKind::Unary { arg, .. } => {
            collect_symbols(model, *arg, visited, symbols)?;
        }
        SemanticExprKind::Binary { lhs, rhs, .. } => {
            collect_symbols(model, *lhs, visited, symbols)?;
            collect_symbols(model, *rhs, visited, symbols)?;
        }
        SemanticExprKind::Call { args, .. } | SemanticExprKind::Vector { elements: args } => {
            for argument in args {
                collect_symbols(model, *argument, visited, symbols)?;
            }
        }
        SemanticExprKind::Index { value, indices } => {
            collect_symbols(model, *value, visited, symbols)?;
            for index in indices {
                collect_symbols(model, *index, visited, symbols)?;
            }
        }
        SemanticExprKind::Number { .. } | SemanticExprKind::String { .. } => {}
    }
    Ok(())
}

fn capture_role(role: &SemanticRole) -> Option<FormCaptureRole> {
    match role {
        SemanticRole::PhysicalField(role) => Some(FormCaptureRole::PhysicalField(role.clone())),
        SemanticRole::Parameter => Some(FormCaptureRole::Parameter),
        SemanticRole::Constant => Some(FormCaptureRole::Constant),
        SemanticRole::Source => Some(FormCaptureRole::Source),
        SemanticRole::Property => Some(FormCaptureRole::Property),
        SemanticRole::ConstitutiveLaw => Some(FormCaptureRole::ConstitutiveLaw),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_semantics;
    use quantitas::UnitRegistry;

    #[test]
    fn authored_form_uses_typed_arena_identities_and_roles() {
        let compilation = compile_semantics(
            r#"
module form.test;
model Poisson {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: trial scalar H1(order=1) on Omega;
  field v: test scalar H1(order=1) on Omega;
  parameter alpha: Diffusivity;
  form residual { cell(Omega): alpha * u * v; }
}
"#,
            &UnitRegistry::si_bootstrap(),
        )
        .unwrap();
        let form = compile_variational_form(&compilation.semantic, "Poisson", "residual").unwrap();
        assert_eq!(form.arguments.len(), 2);
        assert_eq!(form.arguments[0].role, FormArgumentRole::Trial);
        assert_eq!(form.arguments[1].role, FormArgumentRole::Test);
        assert_eq!(form.captures.len(), 1);
        assert_eq!(form.captures[0].role, FormCaptureRole::Parameter);
        assert_eq!(form.integrals.len(), 1);
        assert_eq!(form.receipt.transformations, [FormTransformation::Authored]);
        assert_eq!(form.artifact_digest.algorithm, "blake3");
    }
}
