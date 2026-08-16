//! Generic weak-form lowering for the scientific-v1 semantic IR.
//!
//! This pass recognizes mathematical structure, not named physics.  In particular there is
//! no heat/electrical special case: scalar H1 equations composed from `dt`, `grad`, `div`,
//! pointwise expressions, properties, sources, and constitutive aliases lower to the same
//! small weak-term vocabulary.

use crate::scientific::{BinaryOp, Expr, ScientificModel, UnaryOp};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeakResidualBlock {
    pub name: String,
    pub domain: Option<String>,
    pub primary_field: String,
    pub terms: Vec<WeakTerm>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WeakTerm {
    /// `coefficient * dt(field) * test`
    Mass {
        field: String,
        coefficient: Expr,
    },
    /// `coefficient * grad(field) dot grad(test)`
    Diffusion {
        field: String,
        coefficient: Expr,
    },
    /// `expression * test`, after moving the equation to residual form.
    Pointwise { expression: Expr },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeakOperatorProgram {
    pub model: String,
    pub blocks: Vec<WeakResidualBlock>,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum WeakLoweringError {
    #[error("equation `{0}` was not found")]
    MissingEquation(String),
    #[error("cyclic semantic alias while expanding `{0}`")]
    AliasCycle(String),
    #[error("equation `{0}` does not contain a primary differential field")]
    MissingPrimaryField(String),
    #[error("unsupported differential expression in equation `{equation}`: {expression:?}")]
    UnsupportedDifferential {
        equation: String,
        expression: Expr,
    },
}

pub fn lower_scalar_h1_model(model: &ScientificModel) -> Result<WeakOperatorProgram, WeakLoweringError> {
    let blocks = model
        .equations
        .iter()
        .map(|equation| lower_scalar_h1_equation(model, &equation.name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(WeakOperatorProgram {
        model: model.name.clone(),
        blocks,
    })
}

pub fn lower_scalar_h1_equation(
    model: &ScientificModel,
    equation_name: &str,
) -> Result<WeakResidualBlock, WeakLoweringError> {
    let equation = model
        .equations
        .iter()
        .find(|equation| equation.name == equation_name)
        .ok_or_else(|| WeakLoweringError::MissingEquation(equation_name.into()))?;

    let aliases = semantic_aliases(model);
    let lhs = expand_aliases(&equation.lhs, &aliases, &mut BTreeSet::new())?;
    let rhs = expand_aliases(&equation.rhs, &aliases, &mut BTreeSet::new())?;
    let residual = Expr::Binary {
        op: BinaryOp::Sub,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    };

    let mut additive = Vec::new();
    flatten_additive(&residual, 1.0, &mut additive);
    let mut terms = Vec::new();
    let mut primary_candidates = BTreeSet::new();

    for (sign, expression) in additive {
        if let Some((field, coefficient)) = match_mass_term(&expression) {
            primary_candidates.insert(field.clone());
            terms.push(WeakTerm::Mass {
                field,
                coefficient: scaled(sign, coefficient),
            });
            continue;
        }
        if let Some((field, coefficient)) = match_div_grad_term(&expression) {
            primary_candidates.insert(field.clone());
            // Integration by parts contributes a minus sign:
            // integral div(c grad u) v = boundary - integral c grad u . grad v.
            terms.push(WeakTerm::Diffusion {
                field,
                coefficient: scaled(-sign, coefficient),
            });
            continue;
        }
        if contains_differential_operator(&expression) {
            return Err(WeakLoweringError::UnsupportedDifferential {
                equation: equation.name.clone(),
                expression,
            });
        }
        terms.push(WeakTerm::Pointwise {
            expression: scaled(sign, expression),
        });
    }

    let primary_field = select_primary_field(model, &primary_candidates)
        .ok_or_else(|| WeakLoweringError::MissingPrimaryField(equation.name.clone()))?;

    Ok(WeakResidualBlock {
        name: equation.name.clone(),
        domain: equation.domain.clone(),
        primary_field,
        terms,
    })
}

fn semantic_aliases(model: &ScientificModel) -> BTreeMap<String, Expr> {
    let mut aliases = BTreeMap::new();
    for property in &model.properties {
        aliases.insert(property.name.clone(), property.value.clone());
    }
    for law in &model.constitutive_laws {
        aliases.insert(law.name.clone(), law.law.clone());
    }
    for value in model
        .parameters
        .iter()
        .chain(model.constants.iter())
        .chain(model.sources.iter())
    {
        if let Some(expr) = &value.value {
            aliases.insert(value.name.clone(), expr.clone());
        }
    }
    aliases
}

fn expand_aliases(
    expression: &Expr,
    aliases: &BTreeMap<String, Expr>,
    stack: &mut BTreeSet<String>,
) -> Result<Expr, WeakLoweringError> {
    Ok(match expression {
        Expr::Name(name) if aliases.contains_key(name) => {
            if !stack.insert(name.clone()) {
                return Err(WeakLoweringError::AliasCycle(name.clone()));
            }
            let expanded = expand_aliases(&aliases[name], aliases, stack)?;
            stack.remove(name);
            expanded
        }
        Expr::Unary { op, arg } => Expr::Unary {
            op: *op,
            arg: Box::new(expand_aliases(arg, aliases, stack)?),
        },
        Expr::Binary { op, lhs, rhs } => Expr::Binary {
            op: *op,
            lhs: Box::new(expand_aliases(lhs, aliases, stack)?),
            rhs: Box::new(expand_aliases(rhs, aliases, stack)?),
        },
        Expr::Call { function, args } => Expr::Call {
            function: function.clone(),
            args: args
                .iter()
                .map(|arg| expand_aliases(arg, aliases, stack))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Expr::Index { value, indices } => Expr::Index {
            value: Box::new(expand_aliases(value, aliases, stack)?),
            indices: indices
                .iter()
                .map(|index| expand_aliases(index, aliases, stack))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Expr::Vector(values) => Expr::Vector(
            values
                .iter()
                .map(|value| expand_aliases(value, aliases, stack))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        other => other.clone(),
    })
}

fn flatten_additive(expression: &Expr, sign: f64, out: &mut Vec<(f64, Expr)>) {
    match expression {
        Expr::Binary {
            op: BinaryOp::Add,
            lhs,
            rhs,
        } => {
            flatten_additive(lhs, sign, out);
            flatten_additive(rhs, sign, out);
        }
        Expr::Binary {
            op: BinaryOp::Sub,
            lhs,
            rhs,
        } => {
            flatten_additive(lhs, sign, out);
            flatten_additive(rhs, -sign, out);
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            arg,
        } => flatten_additive(arg, -sign, out),
        other => out.push((sign, other.clone())),
    }
}

fn flatten_product(expression: &Expr, sign: &mut f64, factors: &mut Vec<Expr>) {
    match expression {
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } => {
            flatten_product(lhs, sign, factors);
            flatten_product(rhs, sign, factors);
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            arg,
        } => {
            *sign = -*sign;
            flatten_product(arg, sign, factors);
        }
        other => factors.push(other.clone()),
    }
}

fn match_mass_term(expression: &Expr) -> Option<(String, Expr)> {
    let mut sign = 1.0;
    let mut factors = Vec::new();
    flatten_product(expression, &mut sign, &mut factors);
    let mut dt_index = None;
    let mut field = None;
    for (index, factor) in factors.iter().enumerate() {
        if let Expr::Call { function, args } = factor
            && function == "dt"
            && let [Expr::Name(name)] = args.as_slice()
        {
            if dt_index.is_some() {
                return None;
            }
            dt_index = Some(index);
            field = Some(name.clone());
        }
    }
    let index = dt_index?;
    factors.remove(index);
    Some((field?, scaled(sign, product(factors))))
}

fn match_div_grad_term(expression: &Expr) -> Option<(String, Expr)> {
    let Expr::Call { function, args } = expression else {
        return None;
    };
    if function != "div" || args.len() != 1 {
        return None;
    }
    let mut sign = 1.0;
    let mut factors = Vec::new();
    flatten_product(&args[0], &mut sign, &mut factors);
    let mut grad_index = None;
    let mut field = None;
    for (index, factor) in factors.iter().enumerate() {
        if let Expr::Call { function, args } = factor
            && function == "grad"
            && let [Expr::Name(name)] = args.as_slice()
        {
            if grad_index.is_some() {
                return None;
            }
            grad_index = Some(index);
            field = Some(name.clone());
        }
    }
    let index = grad_index?;
    factors.remove(index);
    Some((field?, scaled(sign, product(factors))))
}

fn product(mut factors: Vec<Expr>) -> Expr {
    if factors.is_empty() {
        return number(1.0);
    }
    let first = factors.remove(0);
    factors.into_iter().fold(first, |lhs, rhs| Expr::Binary {
        op: BinaryOp::Mul,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn scaled(scale: f64, expression: Expr) -> Expr {
    if scale == 1.0 {
        expression
    } else if scale == -1.0 {
        Expr::Unary {
            op: UnaryOp::Neg,
            arg: Box::new(expression),
        }
    } else {
        Expr::Binary {
            op: BinaryOp::Mul,
            lhs: Box::new(number(scale)),
            rhs: Box::new(expression),
        }
    }
}

fn number(value: f64) -> Expr {
    Expr::Number { value, unit: None }
}

fn contains_differential_operator(expression: &Expr) -> bool {
    match expression {
        Expr::Call { function, args } => {
            matches!(function.as_str(), "dt" | "grad" | "div" | "curl" | "sym_grad")
                || args.iter().any(contains_differential_operator)
        }
        Expr::Unary { arg, .. } => contains_differential_operator(arg),
        Expr::Binary { lhs, rhs, .. } => {
            contains_differential_operator(lhs) || contains_differential_operator(rhs)
        }
        Expr::Index { value, indices } => {
            contains_differential_operator(value)
                || indices.iter().any(contains_differential_operator)
        }
        Expr::Vector(values) => values.iter().any(contains_differential_operator),
        Expr::Number { .. } | Expr::String(_) | Expr::Name(_) => false,
    }
}

fn select_primary_field(model: &ScientificModel, candidates: &BTreeSet<String>) -> Option<String> {
    model
        .fields
        .iter()
        .find(|field| candidates.contains(&field.name))
        .map(|field| field.name.clone())
        .or_else(|| candidates.iter().next().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scientific::parse_scientific_module;

    #[test]
    fn nonlinear_heat_lowers_without_named_physics_special_case() {
        let source = r#"
module test.heat;
model Heat {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field T: state scalar H1(order=1) on Omega { time_role = differential; };
  property rho = density(T);
  property cp = specific_heat(T);
  property k = thermal_conductivity(T);
  source Q: VolumetricHeatSource;
  equation energy on Omega { rho * cp * dt(T) - div(k * grad(T)) = Q; }
}
"#;
        let module = parse_scientific_module(source).unwrap();
        let block = lower_scalar_h1_equation(&module.models[0], "energy").unwrap();
        assert_eq!(block.primary_field, "T");
        assert_eq!(block.terms.len(), 3);
        assert!(matches!(block.terms[0], WeakTerm::Mass { .. }));
        assert!(matches!(block.terms[1], WeakTerm::Diffusion { .. }));
        assert!(matches!(block.terms[2], WeakTerm::Pointwise { .. }));
    }

    #[test]
    fn electrothermal_aliases_lower_into_two_generic_blocks() {
        let source = r#"
module test.et;
model ET {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field V: unknown scalar H1(order=1) on Omega;
  field T: state scalar H1(order=1) on Omega { time_role = differential; };
  property sigma = electrical_conductivity(T);
  property rho = density(T);
  property cp = specific_heat(T);
  property k = thermal_conductivity(T);
  constitutive current = -sigma * grad(V);
  source joule = sigma * dot(grad(V), grad(V));
  equation electrical on Omega { div(current) = 0; }
  equation thermal on Omega { rho * cp * dt(T) - div(k * grad(T)) = joule; }
}
"#;
        let module = parse_scientific_module(source).unwrap();
        let program = lower_scalar_h1_model(&module.models[0]).unwrap();
        assert_eq!(program.blocks.len(), 2);
        assert_eq!(program.blocks[0].primary_field, "V");
        assert_eq!(program.blocks[1].primary_field, "T");
        assert!(program.blocks[1]
            .terms
            .iter()
            .any(|term| matches!(term, WeakTerm::Pointwise { .. })));
    }
}
