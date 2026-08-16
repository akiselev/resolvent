use crate::form::{Field, FieldRole, FormExpr, FormProgram, Integral};
use crate::id::{ExprId, FieldId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GateauxError {
    #[error("custom form operator `{0}` has no declared derivative rule")]
    OpaqueCustom(String),
}

/// Gateaux variation of a form expression with respect to `target`. Scalar-expression
/// coefficients can participate through the supplied `scalar_variation` callback, keeping
/// the generic Expr dialect and the variational dialect separate while allowing nonlinear
/// coefficient rules to bridge them deliberately.
pub fn variation(
    expr: &FormExpr,
    target: FieldId,
    direction: FieldId,
    scalar_variation: &impl Fn(ExprId) -> Option<ExprId>,
) -> Result<Option<FormExpr>, GateauxError> {
    Ok(match expr {
        FormExpr::Scalar(id) => scalar_variation(*id).map(FormExpr::Scalar),
        FormExpr::Field(field) => (*field == target).then_some(FormExpr::Field(direction)),
        FormExpr::Neg(x) => variation(x,target,direction,scalar_variation)?.map(|x| FormExpr::Neg(Box::new(x))),
        FormExpr::Add(xs) => {
            let terms = xs.iter().filter_map(|x| variation(x,target,direction,scalar_variation).transpose()).collect::<Result<Vec<_>,_>>()?;
            (!terms.is_empty()).then_some(FormExpr::Add(terms))
        }
        FormExpr::Product(xs) => {
            let mut sum = Vec::new();
            for i in 0..xs.len() { if let Some(dx)=variation(&xs[i],target,direction,scalar_variation)? { let mut factors=xs.clone(); factors[i]=dx; sum.push(FormExpr::Product(factors)); } }
            (!sum.is_empty()).then_some(FormExpr::Add(sum))
        }
        FormExpr::Gradient(x) => variation(x,target,direction,scalar_variation)?.map(|x|FormExpr::Gradient(Box::new(x))),
        FormExpr::Divergence(x) => variation(x,target,direction,scalar_variation)?.map(|x|FormExpr::Divergence(Box::new(x))),
        FormExpr::Curl(x) => variation(x,target,direction,scalar_variation)?.map(|x|FormExpr::Curl(Box::new(x))),
        FormExpr::TimeDerivative(x) => variation(x,target,direction,scalar_variation)?.map(|x|FormExpr::TimeDerivative(Box::new(x))),
        FormExpr::Trace(x) => variation(x,target,direction,scalar_variation)?.map(|x|FormExpr::Trace(Box::new(x))),
        FormExpr::Inner { left, right } => product_binary(left,right,target,direction,scalar_variation,|a,b|FormExpr::Inner{left:Box::new(a),right:Box::new(b)})?,
        FormExpr::Contract { left, right } => product_binary(left,right,target,direction,scalar_variation,|a,b|FormExpr::Contract{left:Box::new(a),right:Box::new(b)})?,
        FormExpr::Custom { operator, .. } => return Err(GateauxError::OpaqueCustom(operator.clone())),
    })
}

fn product_binary(
    left:&FormExpr,right:&FormExpr,target:FieldId,direction:FieldId,scalar_variation:&impl Fn(ExprId)->Option<ExprId>,
    build:impl Fn(FormExpr,FormExpr)->FormExpr,
)->Result<Option<FormExpr>,GateauxError>{
    let dl=variation(left,target,direction,scalar_variation)?; let dr=variation(right,target,direction,scalar_variation)?; let mut sum=Vec::new();
    if let Some(dl)=dl { sum.push(build(dl,right.clone())); } if let Some(dr)=dr { sum.push(build(left.clone(),dr)); }
    Ok(match sum.len(){0=>None,1=>sum.pop(),_=>Some(FormExpr::Add(sum))})
}

pub fn linearize_form(
    form:&FormProgram,target:FieldId,mut direction_field:Field,scalar_variation:&impl Fn(ExprId)->Option<ExprId>
)->Result<FormProgram,GateauxError>{
    direction_field.role=FieldRole::Trial; let direction=direction_field.id; let mut residual_terms=Vec::new(); let mut boundary_terms=Vec::new();
    for term in &form.residual_terms { if let Some(integrand)=variation(&term.integrand,target,direction,scalar_variation)? { residual_terms.push(Integral{integrand,measure:term.measure.clone(),label:term.label.clone()}); } }
    for term in &form.boundary_terms { if let Some(integrand)=variation(&term.integrand,target,direction,scalar_variation)? { boundary_terms.push(Integral{integrand,measure:term.measure.clone(),label:term.label.clone()}); } }
    let mut fields=form.fields.clone(); if !fields.iter().any(|f|f.id==direction){fields.push(direction_field);}
    Ok(FormProgram{name:format!("{}::linearized",form.name),fields,residual_terms,boundary_terms,metadata:form.metadata.clone()})
}
