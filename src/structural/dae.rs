use crate::expr::{ExprNode, ExprStore};
use crate::id::{ExprId, SymbolId};
use crate::model::System;
use crate::structural::{IncidenceSystem, Matching, StructuralError, maximum_matching};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DerivativeVariable { pub base: SymbolId, pub with_respect_to: SymbolId, pub order: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasGroup { pub representative: SymbolId, pub members: Vec<SymbolId> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PantelidesStep {
    pub equation: usize,
    pub differentiate_times: u8,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DummyDerivative {
    pub derivative: DerivativeVariable,
    pub replacement_name: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralDaeAnalysis {
    pub incidence: IncidenceSystem,
    pub matching: Matching,
    pub derivative_variables: Vec<DerivativeVariable>,
    pub aliases: Vec<AliasGroup>,
    pub pantelides: Vec<PantelidesStep>,
    pub dummy_derivatives: Vec<DummyDerivative>,
    pub estimated_structural_index: u8,
}

pub fn analyze_dae(system: &System, exprs: &ExprStore) -> Result<StructuralDaeAnalysis, StructuralError> {
    let incidence = IncidenceSystem::from_system(system, exprs)?;
    let matching = maximum_matching(&incidence);
    let derivative_variables = collect_derivative_variables(system, exprs)?;
    let aliases = alias_groups(system, exprs)?;
    let pantelides = pantelides_plan(system, exprs, &incidence, &matching, &derivative_variables)?;
    let estimated_structural_index = pantelides.iter().map(|s| s.differentiate_times).max().unwrap_or(0).saturating_add(if derivative_variables.is_empty() { 0 } else { 1 });
    let dummy_derivatives = select_dummy_derivatives(&derivative_variables, &pantelides);
    Ok(StructuralDaeAnalysis { incidence, matching, derivative_variables, aliases, pantelides, dummy_derivatives, estimated_structural_index })
}

pub fn collect_derivative_variables(system: &System, exprs: &ExprStore) -> Result<Vec<DerivativeVariable>, StructuralError> {
    let mut found = BTreeSet::new();
    for equation in &system.equations {
        collect_derivatives(equation.lhs, exprs, &mut found)?;
        collect_derivatives(equation.rhs, exprs, &mut found)?;
    }
    Ok(found.into_iter().collect())
}

fn collect_derivatives(root: ExprId, exprs: &ExprStore, out: &mut BTreeSet<DerivativeVariable>) -> Result<(), StructuralError> {
    let mut stack = vec![root]; let mut seen = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) { continue; }
        let node = exprs.get(id).ok_or(StructuralError::MissingExpression(id.0))?;
        match node {
            ExprNode::Derivative { expr, with_respect_to, order } => {
                if let Some(ExprNode::Symbol(base)) = exprs.get(*expr) { out.insert(DerivativeVariable { base: *base, with_respect_to: *with_respect_to, order: *order }); }
                stack.push(*expr);
            }
            ExprNode::Neg(x) => stack.push(*x),
            ExprNode::Add(xs) | ExprNode::Mul(xs) => stack.extend(xs.iter().copied()),
            ExprNode::Div { numerator, denominator } => { stack.push(*numerator); stack.push(*denominator); }
            ExprNode::PowI { base, .. } => stack.push(*base),
            ExprNode::Apply { args, .. } => stack.extend(args.iter().copied()),
            ExprNode::Literal(_) | ExprNode::Symbol(_) => {}
        }
    }
    Ok(())
}

pub fn alias_groups(system: &System, exprs: &ExprStore) -> Result<Vec<AliasGroup>, StructuralError> {
    let mut parent: BTreeMap<SymbolId, SymbolId> = system.unknowns.iter().copied().map(|s| (s, s)).collect();
    for equation in &system.equations {
        let lhs = direct_symbol(equation.lhs, exprs)?; let rhs = direct_symbol(equation.rhs, exprs)?;
        if let (Some(a), Some(b)) = (lhs, rhs) { union(&mut parent, a, b); }
    }
    let mut groups: BTreeMap<SymbolId, Vec<SymbolId>> = BTreeMap::new();
    for symbol in system.unknowns.iter().copied() { let root = find(&mut parent, symbol); groups.entry(root).or_default().push(symbol); }
    Ok(groups.into_iter().filter(|(_, members)| members.len() > 1).map(|(representative, mut members)| { members.sort(); AliasGroup { representative, members } }).collect())
}

fn direct_symbol(id: ExprId, exprs: &ExprStore) -> Result<Option<SymbolId>, StructuralError> { Ok(match exprs.get(id).ok_or(StructuralError::MissingExpression(id.0))? { ExprNode::Symbol(s) => Some(*s), _ => None }) }
fn find(parent: &mut BTreeMap<SymbolId, SymbolId>, x: SymbolId) -> SymbolId { let p = parent.get(&x).copied().unwrap_or(x); if p == x { x } else { let root = find(parent, p); parent.insert(x, root); root } }
fn union(parent: &mut BTreeMap<SymbolId, SymbolId>, a: SymbolId, b: SymbolId) { let ra = find(parent, a); let rb = find(parent, b); if ra != rb { let (lo, hi) = if ra < rb { (ra, rb) } else { (rb,ra) }; parent.insert(hi, lo); } }

/// Conservative Pantelides planning over the semantic derivative graph. It does not silently
/// mutate the equation system: each requested differentiation is explicit and can be replayed
/// by the symbolic calculus layer, preserving a refinement receipt.
pub fn pantelides_plan(system: &System, exprs: &ExprStore, incidence: &IncidenceSystem, matching: &Matching, derivatives: &[DerivativeVariable]) -> Result<Vec<PantelidesStep>, StructuralError> {
    if matching.is_perfect() || derivatives.is_empty() { return Ok(Vec::new()); }
    let mut derivative_bases = BTreeSet::new();
    for derivative in derivatives { derivative_bases.insert(derivative.base); }
    let variable_columns: BTreeMap<SymbolId, usize> = incidence.variables.iter().copied().enumerate().map(|(i,s)| (s,i)).collect();
    let unmatched: BTreeSet<usize> = matching.unmatched_variables().into_iter().collect();
    let mut steps = Vec::new();
    for (equation_index, row) in incidence.rows.iter().enumerate() {
        let touches_unmatched_differential = row.iter().any(|column| unmatched.contains(column) && derivative_bases.contains(&incidence.variables[*column]));
        if touches_unmatched_differential {
            steps.push(PantelidesStep { equation: equation_index, differentiate_times: 1, reason: "equation touches an unmatched variable that also appears differentiated; expose one additional derivative incidence".into() });
        }
    }
    if steps.is_empty() && !matching.unmatched_equations().is_empty() {
        for equation in matching.unmatched_equations() {
            let touches_differential = incidence.rows[equation].iter().any(|column| derivative_bases.contains(&incidence.variables[*column]));
            if touches_differential { steps.push(PantelidesStep { equation, differentiate_times: 1, reason: "unmatched equation is incident to a differential variable".into() }); }
        }
    }
    let _ = variable_columns;
    let _ = system;
    let _ = exprs;
    Ok(steps)
}

pub fn select_dummy_derivatives(derivatives: &[DerivativeVariable], pantelides: &[PantelidesStep]) -> Vec<DummyDerivative> {
    if pantelides.is_empty() { return Vec::new(); }
    let max_order = derivatives.iter().map(|d| d.order).max().unwrap_or(0);
    derivatives.iter().filter(|d| d.order == max_order && max_order > 0).enumerate().map(|(index, derivative)| DummyDerivative { derivative: derivative.clone(), replacement_name: format!("__dummy_d{index}"), reason: "highest-order derivative participates in an index-reduction differentiation; expose an independently solvable dummy derivative candidate".into() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{ExprNode, Symbol, SymbolRole};
    use crate::model::Equation;

    #[test]
    fn finds_aliases_and_derivatives() {
        let mut symbols = crate::expr::SymbolTable::default();
        let x = symbols.declare(Symbol { name: "x".into(), role: SymbolRole::State, dimension: None });
        let y = symbols.declare(Symbol { name: "y".into(), role: SymbolRole::Algebraic, dimension: None });
        let t = symbols.declare(Symbol { name: "t".into(), role: SymbolRole::Independent, dimension: None });
        let mut exprs = ExprStore::new(); let ex = exprs.symbol(x); let ey = exprs.symbol(y); let dx = exprs.intern(ExprNode::Derivative { expr: ex, with_respect_to: t, order: 1 });
        let system = System { name: "dae".into(), unknowns: vec![x,y], parameters: vec![], equations: vec![Equation { lhs: ex, rhs: ey, label: None }, Equation { lhs: dx, rhs: ey, label: None }], events: vec![], children: vec![], metadata: BTreeMap::new() };
        assert_eq!(alias_groups(&system, &exprs).unwrap().len(), 1);
        assert_eq!(collect_derivative_variables(&system, &exprs).unwrap().len(), 1);
    }
}
