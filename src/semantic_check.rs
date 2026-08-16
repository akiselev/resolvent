use crate::author::{DeclKind, ParsedModel};
use crate::diagnostic::{Diagnostic, SourceSpan};
use crate::latex::{MathExpr, MathEquation};
use crate::units::{Dimension, UnitExpr};
use std::collections::BTreeMap;

pub fn check_model(model:&ParsedModel)->Vec<Diagnostic>{
    let mut dims=BTreeMap::<String,Dimension>::new(); let mut functions=BTreeMap::<String,Dimension>::new();
    for decl in &model.declarations { if let Some(UnitExpr{dimension,..})=&decl.unit { match decl.kind { DeclKind::Coefficient|DeclKind::Source=>{functions.insert(decl.name.clone(),*dimension);},_=>{dims.insert(decl.name.clone(),*dimension);} } } }
    dims.insert("t".into(),Dimension::TIME);dims.insert("x".into(),Dimension::LENGTH);dims.insert("y".into(),Dimension::LENGTH);dims.insert("z".into(),Dimension::LENGTH);
    let mut out=Vec::new();for equation in &model.equations{check_equation(&equation.parsed,equation.span,&dims,&functions,&mut out);}for boundary in &model.boundaries{for equation in &boundary.equations{check_equation(&equation.parsed,equation.span,&dims,&functions,&mut out);}}out
}

fn check_equation(eq:&MathEquation,span:SourceSpan,dims:&BTreeMap<String,Dimension>,functions:&BTreeMap<String,Dimension>,out:&mut Vec<Diagnostic>){match(infer(&eq.lhs,dims,functions),infer(&eq.rhs,dims,functions)){(Ok(a),Ok(b))if a!=b=>out.push(Diagnostic::error("RSL-U014","units",format!("equation dimensions disagree: left is `{a}`, right is `{b}`")).at(span,"dimension mismatch in this equation")),(Err(m),_)|(_,Err(m))=>out.push(Diagnostic::error("RSL-U013","units",m).at(span,"while checking this equation")),_=>{}}}

fn infer(expr:&MathExpr,dims:&BTreeMap<String,Dimension>,functions:&BTreeMap<String,Dimension>)->Result<Dimension,String>{Ok(match expr{
    MathExpr::Number(_)=>Dimension::DIMENSIONLESS,
    MathExpr::Name(name)=>dims.get(name).copied().or_else(||functions.get(name).copied()).unwrap_or(Dimension::DIMENSIONLESS),
    MathExpr::Neg(x)=>infer(x,dims,functions)?,
    MathExpr::Add(xs)=>{let mut it=xs.iter();let first=it.next().map(|x|infer(x,dims,functions)).transpose()?.unwrap_or(Dimension::DIMENSIONLESS);for x in it{let d=infer(x,dims,functions)?;if d!=first{return Err(format!("additive terms have dimensions `{first}` and `{d}`"));}}first},
    MathExpr::Mul(xs)=>xs.iter().try_fold(Dimension::DIMENSIONLESS,|a,x|Ok::<_,String>(a.mul(infer(x,dims,functions)?)))?,
    MathExpr::Div{numerator,denominator}=>infer(numerator,dims,functions)?.div(infer(denominator,dims,functions)?),
    MathExpr::Pow{base,exponent}=>infer(base,dims,functions)?.powi((*exponent).try_into().map_err(|_|"unit exponent outside i8 range".to_string())?),
    MathExpr::Call{function,args}=>{if let Some(d)=functions.get(function){*d}else{match function.as_str(){"sin"|"cos"|"tan"|"exp"|"log"=>{if let Some(arg)=args.first(){let d=infer(arg,dims,functions)?;if d!=Dimension::DIMENSIONLESS{return Err(format!("{function} requires a dimensionless argument, found `{d}`"));}}Dimension::DIMENSIONLESS},"sqrt"=>return Err("sqrt dimension inference requires even dimension exponents and is not implicit yet".into()),_=>Dimension::DIMENSIONLESS}}},
    MathExpr::Derivative{expr,with_respect_to,order}=>{let wrt=dims.get(with_respect_to).copied().ok_or_else(||format!("unknown derivative coordinate `{with_respect_to}`"))?;infer(expr,dims,functions)?.div(wrt.powi((*order).try_into().unwrap_or(i8::MAX)))},
    MathExpr::Gradient(x)|MathExpr::Divergence(x)|MathExpr::Curl(x)=>infer(x,dims,functions)?.div(Dimension::LENGTH),
    MathExpr::Inner{left,right}=>infer(left,dims,functions)?.mul(infer(right,dims,functions)?),
})}
