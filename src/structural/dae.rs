use crate::expr::{ExprNode, ExprStore};
use crate::id::{ExprId, SymbolId};
use crate::model::System;
use crate::structural::{IncidenceSystem, StructuralError, maximum_matching};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DerivativeVariable { pub symbol: SymbolId, pub order: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasClass { pub representative: SymbolId, pub members: Vec<SymbolId> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasAnalysis { pub classes: Vec<AliasClass>, pub eliminated_equations: Vec<usize> }

/// Detect direct aliases `x = y` in the semantic System. The compiler reports classes first;
/// a later rewrite may substitute them while preserving a refinement receipt.
pub fn analyze_aliases(system:&System,exprs:&ExprStore)->AliasAnalysis{
    let mut parent:BTreeMap<SymbolId,SymbolId>=system.unknowns.iter().copied().map(|s|(s,s)).collect(); let mut eliminated=vec![];
    for(i,e)in system.equations.iter().enumerate(){if let(Some(a),Some(b))=(as_symbol(exprs,e.lhs),as_symbol(exprs,e.rhs)){if parent.contains_key(&a)&&parent.contains_key(&b){union(&mut parent,a,b);eliminated.push(i)}}}
    let mut groups:BTreeMap<SymbolId,Vec<SymbolId>>=BTreeMap::new();for s in system.unknowns.iter().copied(){let r=find(&mut parent,s);groups.entry(r).or_default().push(s)}
    AliasAnalysis{classes:groups.into_iter().filter_map(|(representative,members)|(members.len()>1).then_some(AliasClass{representative,members})).collect(),eliminated_equations:eliminated}
}
fn as_symbol(exprs:&ExprStore,id:ExprId)->Option<SymbolId>{match exprs.get(id)?{ExprNode::Symbol(s)=>Some(*s),_=>None}}
fn find(parent:&mut BTreeMap<SymbolId,SymbolId>,x:SymbolId)->SymbolId{let p=*parent.get(&x).unwrap_or(&x);if p==x{x}else{let r=find(parent,p);parent.insert(x,r);r}}
fn union(parent:&mut BTreeMap<SymbolId,SymbolId>,a:SymbolId,b:SymbolId){let ra=find(parent,a);let rb=find(parent,b);if ra!=rb{let(lo,hi)=if ra<rb{(ra,rb)}else{(rb,ra)};parent.insert(hi,lo);}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationDerivativeProfile { pub equation: usize, pub derivatives: Vec<DerivativeVariable> }

pub fn derivative_profile(system:&System,exprs:&ExprStore,time:SymbolId)->Result<Vec<EquationDerivativeProfile>,StructuralError>{
    system.equations.iter().enumerate().map(|(i,e)|{let mut set=BTreeSet::new();collect_derivatives(e.lhs,exprs,time,&mut set)?;collect_derivatives(e.rhs,exprs,time,&mut set)?;Ok(EquationDerivativeProfile{equation:i,derivatives:set.into_iter().collect()})}).collect()
}
fn collect_derivatives(root:ExprId,exprs:&ExprStore,time:SymbolId,out:&mut BTreeSet<DerivativeVariable>)->Result<(),StructuralError>{let mut stack=vec![root];let mut seen=BTreeSet::new();while let Some(id)=stack.pop(){if !seen.insert(id){continue}let node=exprs.get(id).ok_or(StructuralError::MissingExpression(id.0))?;match node{ExprNode::Literal(_)|ExprNode::Symbol(_)=>{},ExprNode::Neg(x)=>stack.push(*x),ExprNode::Add(xs)|ExprNode::Mul(xs)=>stack.extend(xs.iter().copied()),ExprNode::Div{numerator,denominator}=>{stack.push(*numerator);stack.push(*denominator)},ExprNode::PowI{base,..}=>stack.push(*base),ExprNode::Apply{args,..}=>stack.extend(args.iter().copied()),ExprNode::Derivative{expr,with_respect_to,order}=>{if *with_respect_to==time{if let Some(s)=as_symbol(exprs,*expr){out.insert(DerivativeVariable{symbol:s,order:*order});}}stack.push(*expr)}}}Ok(())}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentiationStep { pub equation: usize, pub new_order: u8, pub reason: String }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexReductionPlan { pub structural_index_lower_bound: u8, pub steps: Vec<DifferentiationStep>, pub unmatched_equations: Vec<usize>, pub unmatched_variables: Vec<usize>, pub consistent_initialization_required: bool }

/// A deterministic Pantelides-style planning pass. It does not silently mutate the source
/// System: it identifies structurally unmatched equations and the differentiation order that
/// must be introduced. A later semantic pass materializes the differentiated equations and
/// records the exact refinement relation.
pub fn pantelides_plan(system:&System,exprs:&ExprStore,time:SymbolId,max_order:u8)->Result<IndexReductionPlan,StructuralError>{
    let incidence=IncidenceSystem::from_system(system,exprs)?;let matching=maximum_matching(&incidence);let profiles=derivative_profile(system,exprs,time)?;
    let unmatched_eq=matching.unmatched_equations();let unmatched_var=matching.unmatched_variables();let mut steps=vec![];let mut index=if unmatched_eq.is_empty(){1}else{2};
    for eq in &unmatched_eq{let existing=profiles.get(*eq).and_then(|p|p.derivatives.iter().map(|d|d.order).max()).unwrap_or(0);let target=(existing+1).min(max_order.max(1));index=index.max(target.saturating_add(1));steps.push(DifferentiationStep{equation:*eq,new_order:target,reason:"equation is structurally unmatched; expose derivative incidence before causalization".into()});}
    // Square systems can still be higher-index if an algebraic equation constrains a state
    // that appears differentiated elsewhere. Mark those constraints for one derivative.
    if unmatched_eq.is_empty(){let differentiated:BTreeSet<_>=profiles.iter().flat_map(|p|p.derivatives.iter().map(|d|d.symbol)).collect();for(i,e)in system.equations.iter().enumerate(){let mut syms=BTreeSet::new();super::collect_symbols_for_dae(e.lhs,exprs,&mut syms)?;super::collect_symbols_for_dae(e.rhs,exprs,&mut syms)?;if profiles[i].derivatives.is_empty()&&syms.iter().any(|s|differentiated.contains(s)){steps.push(DifferentiationStep{equation:i,new_order:1,reason:"algebraic constraint closes a differentiated state; candidate hidden constraint".into()});index=index.max(2);}}}
    steps.sort_by_key(|s|(s.equation,s.new_order));steps.dedup_by_key(|s|(s.equation,s.new_order));
    Ok(IndexReductionPlan{structural_index_lower_bound:index,consistent_initialization_required:!steps.is_empty(),steps,unmatched_equations:unmatched_eq,unmatched_variables:unmatched_var})
}

#[cfg(test)]mod tests{use super::*;use crate::expr::{ScalarLiteral,Symbol,SymbolRole};use crate::model::Equation;use crate::Context;#[test]fn detects_alias_class(){let mut c=Context::new();let x=c.declare_symbol(Symbol{name:"x".into(),role:SymbolRole::Algebraic,dimension:None});let y=c.declare_symbol(Symbol{name:"y".into(),role:SymbolRole::Algebraic,dimension:None});let ex=c.exprs.symbol(x);let ey=c.exprs.symbol(y);let s=System{name:"a".into(),unknowns:vec![x,y],parameters:vec![],equations:vec![Equation{lhs:ex,rhs:ey,label:None}],events:vec![],children:vec![],metadata:Default::default()};let a=analyze_aliases(&s,&c.exprs);assert_eq!(a.classes.len(),1);}#[test]fn hidden_constraint_requests_differentiation(){let mut c=Context::new();let t=c.declare_symbol(Symbol{name:"t".into(),role:SymbolRole::Independent,dimension:None});let x=c.declare_symbol(Symbol{name:"x".into(),role:SymbolRole::State,dimension:None});let y=c.declare_symbol(Symbol{name:"y".into(),role:SymbolRole::State,dimension:None});let ex=c.exprs.symbol(x);let ey=c.exprs.symbol(y);let dx=c.exprs.intern(ExprNode::Derivative{expr:ex,with_respect_to:t,order:1});let zero=c.exprs.literal(ScalarLiteral::integer(0));let s=System{name:"dae".into(),unknowns:vec![x,y],parameters:vec![],equations:vec![Equation{lhs:dx,rhs:ey,label:None},Equation{lhs:ex,rhs:zero,label:None}],events:vec![],children:vec![],metadata:Default::default()};let p=pantelides_plan(&s,&c.exprs,t,3).unwrap();assert!(p.steps.iter().any(|s|s.equation==1));assert!(p.consistent_initialization_required);}}
