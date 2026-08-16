//! Structural equation analysis over the common [`System`] IR.
//!
//! The module deliberately owns projections and passes, not a second source equation AST.
//! Incidence is derived from `System` + `ExprStore`; matching, SCC/BLT, tearing and DAE
//! structure operate on that projection.

pub mod dae;
pub mod scc;
pub mod schedule;

use crate::expr::{ExprNode, ExprStore};
use crate::id::SymbolId;
use crate::model::System;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub use schedule::{Block, BlockKind, Schedule, StructuralCompileError, compile_schedule, compile_schedule_without_tearing};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidenceSystem { pub variables: Vec<SymbolId>, pub rows: Vec<Vec<usize>> }

impl IncidenceSystem {
    pub fn from_system(system: &System, exprs: &ExprStore) -> Result<Self, StructuralError> {
        let mut columns = BTreeMap::new();
        for (index, symbol) in system.unknowns.iter().copied().enumerate() { columns.insert(symbol, index); }
        let mut rows = Vec::with_capacity(system.equations.len());
        for equation in &system.equations {
            let mut symbols = BTreeSet::new(); collect_symbols(equation.lhs, exprs, &mut symbols)?; collect_symbols(equation.rhs, exprs, &mut symbols)?;
            rows.push(symbols.into_iter().filter_map(|symbol| columns.get(&symbol).copied()).collect());
        }
        Ok(Self { variables: system.unknowns.clone(), rows })
    }
    pub fn n_equations(&self) -> usize { self.rows.len() }
    pub fn n_variables(&self) -> usize { self.variables.len() }
}

fn collect_symbols(root: crate::id::ExprId, exprs: &ExprStore, out: &mut BTreeSet<SymbolId>) -> Result<(), StructuralError> {
    let mut stack = vec![root]; let mut seen = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) { continue; }
        let node = exprs.get(id).ok_or(StructuralError::MissingExpression(id.0))?;
        match node {
            ExprNode::Literal(_) => {}
            ExprNode::Symbol(symbol) => { out.insert(*symbol); }
            ExprNode::Neg(x) => stack.push(*x),
            ExprNode::Add(xs) | ExprNode::Mul(xs) => stack.extend(xs.iter().copied()),
            ExprNode::Div { numerator, denominator } => { stack.push(*numerator); stack.push(*denominator); }
            ExprNode::PowI { base, .. } => stack.push(*base),
            ExprNode::Apply { args, .. } => stack.extend(args.iter().copied()),
            ExprNode::Derivative { expr, .. } => stack.push(*expr),
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matching { pub equation_to_variable: Vec<Option<usize>>, pub variable_to_equation: Vec<Option<usize>> }
impl Matching {
    pub fn cardinality(&self) -> usize { self.equation_to_variable.iter().filter(|v| v.is_some()).count() }
    pub fn is_perfect(&self) -> bool { self.equation_to_variable.len() == self.variable_to_equation.len() && self.cardinality() == self.equation_to_variable.len() }
    pub fn unmatched_equations(&self) -> Vec<usize> { self.equation_to_variable.iter().enumerate().filter_map(|(i,v)| v.is_none().then_some(i)).collect() }
    pub fn unmatched_variables(&self) -> Vec<usize> { self.variable_to_equation.iter().enumerate().filter_map(|(i,v)| v.is_none().then_some(i)).collect() }
}

pub fn maximum_matching(system: &IncidenceSystem) -> Matching {
    let n_u = system.n_equations(); let n_v = system.n_variables(); let mut pair_u = vec![None; n_u]; let mut pair_v = vec![None; n_v]; let mut dist = vec![usize::MAX; n_u];
    while bfs(&system.rows, &pair_u, &pair_v, &mut dist) { for u in 0..n_u { if pair_u[u].is_none() { dfs(u, &system.rows, &mut pair_u, &mut pair_v, &mut dist); } } }
    Matching { equation_to_variable: pair_u, variable_to_equation: pair_v }
}
fn bfs(rows: &[Vec<usize>], pair_u: &[Option<usize>], pair_v: &[Option<usize>], dist: &mut [usize]) -> bool {
    let mut queue = VecDeque::new(); for u in 0..pair_u.len() { if pair_u[u].is_none() { dist[u] = 0; queue.push_back(u); } else { dist[u] = usize::MAX; } }
    let mut nil = usize::MAX; while let Some(u) = queue.pop_front() { if dist[u] >= nil { continue; } for &v in &rows[u] { match pair_v[v] { None => nil = nil.min(dist[u] + 1), Some(next) if dist[next] == usize::MAX => { dist[next] = dist[u] + 1; queue.push_back(next); }, Some(_) => {} } } } nil != usize::MAX
}
fn dfs(u: usize, rows: &[Vec<usize>], pair_u: &mut [Option<usize>], pair_v: &mut [Option<usize>], dist: &mut [usize]) -> bool {
    let next_layer = dist[u].wrapping_add(1); for &v in &rows[u] { let advance = match pair_v[v] { None => true, Some(w) => dist[w] == next_layer && dfs(w, rows, pair_u, pair_v, dist) }; if advance { pair_v[v] = Some(u); pair_u[u] = Some(v); return true; } } dist[u] = usize::MAX; false
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StructuralError { #[error("system references expression id {0} that is absent from the expression store")] MissingExpression(u32) }
