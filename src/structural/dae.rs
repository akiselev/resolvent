//! Alias and differential-algebraic structure derived from scientific expressions.

use crate::scientific::{Expr, ScientificModel};
use crate::structural::{IncidenceSystem, StructuralError, collect_names, maximum_matching};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DerivativeVariable {
    pub field: String,
    pub order: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasClass {
    pub representative: String,
    pub members: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasAnalysis {
    pub classes: Vec<AliasClass>,
    pub eliminated_equations: Vec<usize>,
}

/// Detect direct aliases such as `x = y`. This pass reports candidates; it does not rewrite
/// the authored equations or hide the transformation from later evidence.
pub fn analyze_aliases(model: &ScientificModel) -> AliasAnalysis {
    let variables: Vec<_> = model
        .fields
        .iter()
        .filter(|field| {
            matches!(
                field.role,
                crate::scientific::FieldRole::State | crate::scientific::FieldRole::Unknown
            )
        })
        .map(|field| field.name.clone())
        .collect();
    let mut parent: BTreeMap<String, String> = variables
        .iter()
        .map(|name| (name.clone(), name.clone()))
        .collect();
    let mut eliminated_equations = Vec::new();
    for (index, equation) in model.equations.iter().enumerate() {
        if let (Some(left), Some(right)) = (as_name(&equation.lhs), as_name(&equation.rhs))
            && parent.contains_key(left)
            && parent.contains_key(right)
        {
            union(&mut parent, left, right);
            eliminated_equations.push(index);
        }
    }

    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in variables {
        let representative = find(&mut parent, &variable);
        groups.entry(representative).or_default().push(variable);
    }
    AliasAnalysis {
        classes: groups
            .into_iter()
            .filter_map(|(representative, members)| {
                (members.len() > 1).then_some(AliasClass {
                    representative,
                    members,
                })
            })
            .collect(),
        eliminated_equations,
    }
}

fn as_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Name(name) => Some(name),
        _ => None,
    }
}

fn find(parent: &mut BTreeMap<String, String>, value: &str) -> String {
    let next = parent
        .get(value)
        .cloned()
        .unwrap_or_else(|| value.to_owned());
    if next == value {
        next
    } else {
        let representative = find(parent, &next);
        parent.insert(value.to_owned(), representative.clone());
        representative
    }
}

fn union(parent: &mut BTreeMap<String, String>, left: &str, right: &str) {
    let left = find(parent, left);
    let right = find(parent, right);
    if left != right {
        let (first, second) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        parent.insert(second, first);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationDerivativeProfile {
    pub equation: usize,
    pub derivatives: Vec<DerivativeVariable>,
}

pub fn derivative_profile(model: &ScientificModel) -> Vec<EquationDerivativeProfile> {
    model
        .equations
        .iter()
        .enumerate()
        .map(|(equation, declaration)| {
            let mut derivatives = BTreeSet::new();
            collect_derivatives(&declaration.lhs, &mut derivatives);
            collect_derivatives(&declaration.rhs, &mut derivatives);
            EquationDerivativeProfile {
                equation,
                derivatives: derivatives.into_iter().collect(),
            }
        })
        .collect()
}

fn collect_derivatives(expr: &Expr, out: &mut BTreeSet<DerivativeVariable>) {
    if let Some((field, order)) = derivative_target(expr) {
        out.insert(DerivativeVariable {
            field: field.to_owned(),
            order,
        });
        return;
    }
    match expr {
        Expr::Unary { arg, .. } => collect_derivatives(arg, out),
        Expr::Binary { lhs, rhs, .. } => {
            collect_derivatives(lhs, out);
            collect_derivatives(rhs, out);
        }
        Expr::Call { args, .. } | Expr::Vector(args) => {
            for arg in args {
                collect_derivatives(arg, out);
            }
        }
        Expr::Index { value, indices } => {
            collect_derivatives(value, out);
            for index in indices {
                collect_derivatives(index, out);
            }
        }
        Expr::Number { .. } | Expr::String(_) | Expr::Name(_) => {}
    }
}

fn derivative_target(mut expr: &Expr) -> Option<(&str, u8)> {
    let mut order = 0_u8;
    while let Expr::Call { function, args } = expr {
        if function != "dt" || args.len() != 1 {
            break;
        }
        order = order.saturating_add(1);
        expr = &args[0];
    }
    match (order, expr) {
        (0, _) => None,
        (_, Expr::Name(field)) => Some((field, order)),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentiationStep {
    pub equation: usize,
    pub new_order: u8,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexReductionPlan {
    pub structural_index_lower_bound: u8,
    pub steps: Vec<DifferentiationStep>,
    pub unmatched_equations: Vec<usize>,
    pub unmatched_variables: Vec<usize>,
    pub consistent_initialization_required: bool,
}

/// Plan deterministic Pantelides-style differentiation without mutating the source model.
pub fn pantelides_plan(
    model: &ScientificModel,
    max_order: u8,
) -> Result<IndexReductionPlan, StructuralError> {
    let incidence = IncidenceSystem::from_model(model)?;
    let matching = maximum_matching(&incidence);
    let profiles = derivative_profile(model);
    let unmatched_equations = matching.unmatched_equations();
    let unmatched_variables = matching.unmatched_variables();
    let mut steps = Vec::new();
    let mut index = if unmatched_equations.is_empty() { 1 } else { 2 };

    for equation in &unmatched_equations {
        let existing = profiles
            .get(*equation)
            .and_then(|profile| profile.derivatives.iter().map(|item| item.order).max())
            .unwrap_or(0);
        let target = existing.saturating_add(1).min(max_order.max(1));
        index = index.max(target.saturating_add(1));
        steps.push(DifferentiationStep {
            equation: *equation,
            new_order: target,
            reason: "equation is structurally unmatched; expose derivative incidence before causalization"
                .into(),
        });
    }

    // A square system can still hide an algebraic constraint on a differentiated state.
    if unmatched_equations.is_empty() {
        let differentiated: BTreeSet<_> = profiles
            .iter()
            .flat_map(|profile| profile.derivatives.iter().map(|item| item.field.as_str()))
            .collect();
        for (equation, declaration) in model.equations.iter().enumerate() {
            let mut names = BTreeSet::new();
            collect_names(&declaration.lhs, &mut names);
            collect_names(&declaration.rhs, &mut names);
            if profiles[equation].derivatives.is_empty()
                && names.iter().any(|name| differentiated.contains(name))
            {
                steps.push(DifferentiationStep {
                    equation,
                    new_order: 1,
                    reason: "algebraic constraint closes a differentiated state; candidate hidden constraint"
                        .into(),
                });
                index = index.max(2);
            }
        }
    }

    steps.sort_by_key(|step| (step.equation, step.new_order));
    steps.dedup_by_key(|step| (step.equation, step.new_order));
    Ok(IndexReductionPlan {
        structural_index_lower_bound: index,
        consistent_initialization_required: !steps.is_empty(),
        steps,
        unmatched_equations,
        unmatched_variables,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_scientific_module;

    fn model(body: &str) -> ScientificModel {
        let source = format!(
            "module structural.test;\nmodel Test {{\n  domain D {{ dimension = 1; coordinates = cartesian; }}\n{body}\n}}\n"
        );
        parse_scientific_module(&source).unwrap().models.remove(0)
    }

    #[test]
    fn detects_alias_class() {
        let model = model(
            "  field x: unknown scalar H1(order=1) on D;\n  field y: unknown scalar H1(order=1) on D;\n  equation alias on D { x = y; }",
        );
        let analysis = analyze_aliases(&model);
        assert_eq!(analysis.classes.len(), 1);
        assert_eq!(analysis.classes[0].members, ["x", "y"]);
    }

    #[test]
    fn hidden_constraint_requests_differentiation() {
        let model = model(
            "  field x: state scalar H1(order=1) on D;\n  field y: state scalar H1(order=1) on D;\n  equation dynamic on D { dt(x) = y; }\n  equation constraint on D { x = 0; }",
        );
        let plan = pantelides_plan(&model, 3).unwrap();
        assert!(plan.steps.iter().any(|step| step.equation == 1));
        assert!(plan.consistent_initialization_required);
    }
}
