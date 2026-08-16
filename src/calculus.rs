use crate::expr::{ExprNode, ExprStore, ScalarLiteral};
use crate::id::{ExprId, SymbolId};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CalculusError {
    #[error("missing expression {0}")]
    Missing(u32),
    #[error("missing value for symbol {0}")]
    MissingSymbol(u32),
    #[error("unsupported reference function `{0}`")]
    UnsupportedFunction(String),
    #[error("named constant `{0}` has no reference value")]
    UnknownConstant(String),
    #[error("invalid exact literal `{0}`")]
    InvalidLiteral(String),
}

/// Exact symbolic derivative over the generic scalar DAG. Spatial semantic operators are
/// represented as named applications and use linearity where valid (`grad`, `div`, `curl`,
/// `dot`); unknown functions are left as explicit `d:<name>` applications rather than
/// silently assuming a derivative.
pub fn differentiate(store: &mut ExprStore, root: ExprId, wrt: SymbolId) -> Result<ExprId, CalculusError> {
    let mut memo = BTreeMap::new();
    differentiate_inner(store, root, wrt, &mut memo)
}

fn differentiate_inner(store: &mut ExprStore, root: ExprId, wrt: SymbolId, memo: &mut BTreeMap<ExprId, ExprId>) -> Result<ExprId, CalculusError> {
    if let Some(v) = memo.get(&root) { return Ok(*v); }
    let node = store.get(root).cloned().ok_or(CalculusError::Missing(root.0))?;
    let zero = || ScalarLiteral::integer(0); let one = || ScalarLiteral::integer(1);
    let out = match node {
        ExprNode::Literal(_) => store.literal(zero()),
        ExprNode::Symbol(s) => store.literal(if s == wrt { one() } else { zero() }),
        ExprNode::Neg(x) => { let dx = differentiate_inner(store, x, wrt, memo)?; store.intern(ExprNode::Neg(dx)) }
        ExprNode::Add(xs) => { let dx = xs.into_iter().map(|x| differentiate_inner(store,x,wrt,memo)).collect::<Result<Vec<_>,_>>()?; store.add(dx) }
        ExprNode::Mul(xs) => {
            if xs.is_empty() { store.literal(zero()) } else {
                let mut terms = Vec::with_capacity(xs.len());
                for i in 0..xs.len() {
                    let mut factors = xs.clone(); factors[i] = differentiate_inner(store, xs[i], wrt, memo)?; terms.push(store.mul(factors));
                }
                store.add(terms)
            }
        }
        ExprNode::Div{numerator,denominator} => {
            let dn = differentiate_inner(store,numerator,wrt,memo)?; let dd = differentiate_inner(store,denominator,wrt,memo)?;
            let a = store.mul([dn,denominator]); let b = store.mul([numerator,dd]); let nb = store.intern(ExprNode::Neg(b)); let num = store.add([a,nb]);
            let den = store.intern(ExprNode::PowI{base:denominator,exponent:2}); store.intern(ExprNode::Div{numerator:num,denominator:den})
        }
        ExprNode::PowI{base,exponent} => {
            if exponent == 0 { store.literal(zero()) } else { let db=differentiate_inner(store,base,wrt,memo)?; let c=store.literal(ScalarLiteral::integer(exponent as i64)); let p=store.intern(ExprNode::PowI{base,exponent:exponent-1}); store.mul([c,p,db]) }
        }
        ExprNode::Derivative{expr,with_respect_to,order} => {
            let d = differentiate_inner(store,expr,wrt,memo)?; store.intern(ExprNode::Derivative{expr:d,with_respect_to,order})
        }
        ExprNode::Apply{function,args} => differentiate_apply(store, function, args, wrt, memo)?,
    };
    memo.insert(root,out); Ok(out)
}

fn differentiate_apply(store:&mut ExprStore,function:String,args:Vec<ExprId>,wrt:SymbolId,memo:&mut BTreeMap<ExprId,ExprId>)->Result<ExprId,CalculusError>{
    if matches!(function.as_str(),"grad"|"div"|"curl") && args.len()==1 { let d=differentiate_inner(store,args[0],wrt,memo)?; return Ok(store.intern(ExprNode::Apply{function,args:vec![d]})); }
    if function=="dot" && args.len()==2 { let da=differentiate_inner(store,args[0],wrt,memo)?; let db=differentiate_inner(store,args[1],wrt,memo)?; let a=store.intern(ExprNode::Apply{function:"dot".into(),args:vec![da,args[1]]}); let b=store.intern(ExprNode::Apply{function:"dot".into(),args:vec![args[0],db]}); return Ok(store.add([a,b])); }
    if args.len()==1 && matches!(function.as_str(),"sin"|"cos"|"exp"|"log"|"sqrt") {
        let x=args[0]; let dx=differentiate_inner(store,x,wrt,memo)?;
        let local=match function.as_str(){
            "sin"=>store.intern(ExprNode::Apply{function:"cos".into(),args:vec![x]}),
            "cos"=>{let sin=store.intern(ExprNode::Apply{function:"sin".into(),args:vec![x]});store.intern(ExprNode::Neg(sin))},
            "exp"=>store.intern(ExprNode::Apply{function:"exp".into(),args:vec![x]}),
            "log"=>{let one=store.literal(ScalarLiteral::integer(1));store.intern(ExprNode::Div{numerator:one,denominator:x})},
            "sqrt"=>{let two=store.literal(ScalarLiteral::integer(2));let sq=store.intern(ExprNode::Apply{function:"sqrt".into(),args:vec![x]});let den=store.mul([two,sq]);let one=store.literal(ScalarLiteral::integer(1));store.intern(ExprNode::Div{numerator:one,denominator:den})},
            _=>unreachable!(),
        };
        return Ok(store.mul([local,dx]));
    }
    let mut dargs=Vec::with_capacity(args.len()); for a in &args { dargs.push(differentiate_inner(store,*a,wrt,memo)?); }
    Ok(store.intern(ExprNode::Apply{function:format!("d:{function}"),args:dargs}))
}

pub fn evaluate_f64(store:&ExprStore,root:ExprId,values:&BTreeMap<SymbolId,f64>)->Result<f64,CalculusError>{
    let mut memo=BTreeMap::new(); eval_inner(store,root,values,&mut memo)
}
fn eval_inner(store:&ExprStore,root:ExprId,values:&BTreeMap<SymbolId,f64>,memo:&mut BTreeMap<ExprId,f64>)->Result<f64,CalculusError>{
    if let Some(v)=memo.get(&root){return Ok(*v)}; let node=store.get(root).ok_or(CalculusError::Missing(root.0))?;
    let v=match node{
        ExprNode::Literal(l)=>literal_f64(l)?, ExprNode::Symbol(s)=>*values.get(s).ok_or(CalculusError::MissingSymbol(s.0))?,
        ExprNode::Neg(x)=>-eval_inner(store,*x,values,memo)?, ExprNode::Add(xs)=>xs.iter().try_fold(0.0,|a,x|Ok(a+eval_inner(store,*x,values,memo)?))?,
        ExprNode::Mul(xs)=>xs.iter().try_fold(1.0,|a,x|Ok(a*eval_inner(store,*x,values,memo)?))?,
        ExprNode::Div{numerator,denominator}=>eval_inner(store,*numerator,values,memo)?/eval_inner(store,*denominator,values,memo)?,
        ExprNode::PowI{base,exponent}=>eval_inner(store,*base,values,memo)?.powi(*exponent),
        ExprNode::Derivative{..}=>return Err(CalculusError::UnsupportedFunction("semantic derivative requires a supplied derivative value before scalar evaluation".into())),
        ExprNode::Apply{function,args}=>{let a=args.iter().map(|x|eval_inner(store,*x,values,memo)).collect::<Result<Vec<_>,_>>()?;match(function.as_str(),a.as_slice()){("sin",[x])=>x.sin(),("cos",[x])=>x.cos(),("exp",[x])=>x.exp(),("log",[x])=>x.ln(),("sqrt",[x])=>x.sqrt(),("abs",[x])=>x.abs(),("pow",[x,y])=>x.powf(*y),_=>return Err(CalculusError::UnsupportedFunction(function.clone()))}}
    }; memo.insert(root,v);Ok(v)
}
fn literal_f64(l:&ScalarLiteral)->Result<f64,CalculusError>{match l{ScalarLiteral::Integer(s)=>s.parse().map_err(|_|CalculusError::InvalidLiteral(s.clone())),ScalarLiteral::Rational{numerator,denominator}=>{let n:nobreak::F64=numerator.parse().map_err(|_|CalculusError::InvalidLiteral(numerator.clone()))?;let d:nobreak::F64=denominator.parse().map_err(|_|CalculusError::InvalidLiteral(denominator.clone()))?;Ok(n.0/d.0)},ScalarLiteral::FloatBits(b)=>Ok(f64::from_bits(*b)),ScalarLiteral::NamedConstant(s)=>match s.as_str(){"pi"|"π"=>Ok(std::f64::consts::PI),"e"=>Ok(std::f64::consts::E),_=>Err(CalculusError::UnknownConstant(s.clone()))}}}

// Tiny parsing wrapper avoids type-inference ambiguities without adding a numeric dependency.
mod nobreak { #[derive(Clone,Copy)] pub struct F64(pub f64); impl std::str::FromStr for F64 { type Err=std::num::ParseFloatError; fn from_str(s:&str)->Result<Self,Self::Err>{Ok(Self(s.parse()?))} } }

#[cfg(test)] mod tests{use super::*;use crate::expr::{Symbol,SymbolRole};#[test]fn derivative_matches_quadratic(){let mut s=ExprStore::new();let x=SymbolId(0);let ex=s.symbol(x);let sq=s.mul([ex,ex]);let d=differentiate(&mut s,sq,x).unwrap();let mut v=BTreeMap::new();v.insert(x,3.0);assert!((evaluate_f64(&s,d,&v).unwrap()-6.0).abs()<1e-12);}#[test]fn sin_chain_rule(){let mut s=ExprStore::new();let x=SymbolId(0);let ex=s.symbol(x);let y=s.intern(ExprNode::Apply{function:"sin".into(),args:vec![ex]});let d=differentiate(&mut s,y,x).unwrap();let mut v=BTreeMap::new();v.insert(x,0.0);assert!((evaluate_f64(&s,d,&v).unwrap()-1.0).abs()<1e-12);}}
