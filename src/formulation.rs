//! Variational-form artifacts derived from the canonical scientific model.

use crate::scientific::{Expr, FieldRole, Measure, ScientificModel, SpaceSpec, ValueShape};
use crate::source::SourceSpan;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// A field that is actually referenced by a compiled form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariationalField {
    pub name: String,
    pub role: FieldRole,
    pub shape: ValueShape,
    pub space: SpaceSpec,
    pub domain: String,
}

/// One integral in a compiled variational form. The expression is the canonical Resolvent
/// expression; there is no parallel form-expression tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalIntegral {
    pub measure: Measure,
    pub integrand: Expr,
    pub span: SourceSpan,
}

/// The semantic output of form compilation before tensor and kernel lowering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalForm {
    pub model: String,
    pub name: String,
    pub fields: Vec<VariationalField>,
    pub referenced_symbols: Vec<String>,
    pub integrals: Vec<VariationalIntegral>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum FormCompileError {
    #[error("model `{model}` has no form named `{form}`")]
    MissingForm { model: String, form: String },
    #[error("model `{model}` declares form `{form}` more than once")]
    DuplicateForm { model: String, form: String },
    #[error("form `{form}` contains no integrals")]
    EmptyForm { form: String },
    #[error("form `{form}` integrates over unknown cell domain `{domain}`")]
    UnknownCellDomain { form: String, domain: String },
}

/// Compile one authored form without introducing an alternate expression IR.
///
/// Strong-equation-to-form derivation is intentionally a later pass. Until it exists, callers
/// must select an authored form rather than silently receiving a guessed weak statement.
pub fn compile_variational_form(
    model: &ScientificModel,
    form_name: &str,
) -> Result<VariationalForm, FormCompileError> {
    let mut matches = model.forms.iter().filter(|form| form.name == form_name);
    let form = matches
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
    if form.integrals.is_empty() {
        return Err(FormCompileError::EmptyForm {
            form: form_name.to_owned(),
        });
    }

    let domains: BTreeSet<_> = model
        .domains
        .iter()
        .map(|domain| domain.name.as_str())
        .collect();
    for integral in &form.integrals {
        if let Measure::Cell(domain) = &integral.measure
            && !domains.contains(domain.as_str())
        {
            return Err(FormCompileError::UnknownCellDomain {
                form: form_name.to_owned(),
                domain: domain.clone(),
            });
        }
    }

    let mut referenced = BTreeSet::new();
    for integral in &form.integrals {
        integral.integrand.names(&mut referenced);
    }
    let field_names: BTreeSet<_> = model
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    let fields = model
        .fields
        .iter()
        .filter(|field| referenced.contains(&field.name))
        .map(|field| VariationalField {
            name: field.name.clone(),
            role: field.role.clone(),
            shape: field.shape.clone(),
            space: field.space.clone(),
            domain: field.domain.clone(),
        })
        .collect();
    let referenced_symbols = referenced
        .into_iter()
        .filter(|name| !field_names.contains(name.as_str()))
        .collect();
    let integrals = form
        .integrals
        .iter()
        .map(|integral| VariationalIntegral {
            measure: integral.measure.clone(),
            integrand: integral.integrand.clone(),
            span: integral.span,
        })
        .collect();

    Ok(VariationalForm {
        model: model.name.clone(),
        name: form.name.clone(),
        fields,
        referenced_symbols,
        integrals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_scientific_module;

    #[test]
    fn authored_form_compiles_from_the_canonical_expression_tree() {
        let module = parse_scientific_module(
            r#"
module form.test;
model Poisson {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: unknown scalar H1(order=1) on Omega;
  field v: test scalar H1(order=1) on Omega;
  parameter alpha: Diffusivity;
  form residual { cell(Omega): alpha * u * v; }
}
"#,
        )
        .unwrap();
        let form = compile_variational_form(&module.models[0], "residual").unwrap();
        assert_eq!(form.fields.len(), 2);
        assert_eq!(form.referenced_symbols, ["alpha"]);
        assert_eq!(form.integrals.len(), 1);
    }
}
