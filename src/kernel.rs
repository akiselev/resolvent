//! Direct lowering from Resolvent form expressions into Malleus-owned kernel IR.

use crate::formulation::VariationalForm;
use crate::scientific::{BinaryOp as ResBinaryOp, Expr, Measure, UnaryOp as ResUnaryOp};
use crate::source::SourceSpan;
use malleus::{
    AccessMode, BinaryOp, IndexingMap, IterationDomain, KernelOperand, KernelRegion, NumericPolicy,
    OperandId, ScalarExpr, Statement, StructuredKernel, UnaryOp,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Realization-neutral local work selected from one form integral.
///
/// This artifact identifies local inputs and output while retaining the canonical Resolvent
/// expression. It contains no mesh, basis table, quadrature points, global indices, or schedule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalFormProgram {
    pub name: String,
    pub measure: Measure,
    pub inputs: Vec<String>,
    pub output: String,
    pub expression: Expr,
    pub source: SourceSpan,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum KernelLoweringError {
    #[error("form `{form}` has no integral at index {index}")]
    MissingIntegral { form: String, index: usize },
    #[error("kernel expression references unbound symbol `{0}`")]
    UnboundSymbol(String),
    #[error("expression `{0}` requires tensor/form lowering that is not implemented yet")]
    UnsupportedExpression(String),
}

/// Select realization-neutral local work from a form integral.
pub fn factor_local_integral(
    form: &VariationalForm,
    integral_index: usize,
) -> Result<LocalFormProgram, KernelLoweringError> {
    let integral =
        form.integrals
            .get(integral_index)
            .ok_or_else(|| KernelLoweringError::MissingIntegral {
                form: form.name.clone(),
                index: integral_index,
            })?;
    let mut names = BTreeSet::new();
    integral.integrand.names(&mut names);

    Ok(LocalFormProgram {
        name: format!("{}::{}::integral_{integral_index}", form.model, form.name),
        measure: integral.measure.clone(),
        inputs: names.into_iter().collect(),
        output: "output".into(),
        expression: integral.integrand.clone(),
        source: integral.span,
    })
}

/// Lower realization-neutral scalar work into Malleus's structured kernel IR.
///
/// Differential and tensor operators are rejected explicitly. They will be admitted by the
/// tensor/form factorization passes rather than encoded as magic scalar opcodes.
pub fn lower_local_program(
    program: &LocalFormProgram,
) -> Result<StructuredKernel, KernelLoweringError> {
    let mut operands = Vec::with_capacity(program.inputs.len() + 1);
    let mut bindings = BTreeMap::new();
    for name in &program.inputs {
        let id = OperandId::new(operands.len());
        operands.push(KernelOperand::scalar(name.clone(), AccessMode::Read));
        bindings.insert(name.clone(), id);
    }
    let output = OperandId::new(operands.len());
    operands.push(KernelOperand::scalar(
        program.output.clone(),
        AccessMode::Write,
    ));
    let indexing_maps = (0..operands.len())
        .map(|index| IndexingMap::scalar(OperandId::new(index)))
        .collect();
    let value = lower_expr(&program.expression, &bindings)?;

    Ok(StructuredKernel {
        name: program.name.clone(),
        iteration_domain: IterationDomain::default(),
        iterators: Vec::new(),
        operands,
        indexing_maps,
        body: KernelRegion {
            statements: vec![Statement::Store {
                operand: output,
                value,
            }],
        },
        numeric_policy: NumericPolicy::default(),
    })
}

fn lower_expr(
    expr: &Expr,
    bindings: &BTreeMap<String, OperandId>,
) -> Result<ScalarExpr, KernelLoweringError> {
    Ok(match expr {
        Expr::Number { value, .. } => ScalarExpr::Constant(*value),
        Expr::Name { name, .. } => ScalarExpr::Load(
            *bindings
                .get(name)
                .ok_or_else(|| KernelLoweringError::UnboundSymbol(name.clone()))?,
        ),
        Expr::Unary {
            op: ResUnaryOp::Neg,
            arg,
            ..
        } => ScalarExpr::unary(UnaryOp::Neg, lower_expr(arg, bindings)?),
        Expr::Binary { op, lhs, rhs, .. } => {
            let op = match op {
                ResBinaryOp::Add => BinaryOp::Add,
                ResBinaryOp::Sub => BinaryOp::Sub,
                ResBinaryOp::Mul => BinaryOp::Mul,
                ResBinaryOp::Div => BinaryOp::Div,
                ResBinaryOp::Pow => BinaryOp::Pow,
                ResBinaryOp::Eq
                | ResBinaryOp::Lt
                | ResBinaryOp::Le
                | ResBinaryOp::Gt
                | ResBinaryOp::Ge => {
                    return Err(KernelLoweringError::UnsupportedExpression(
                        "comparison in a scalar integrand".into(),
                    ));
                }
            };
            ScalarExpr::binary(op, lower_expr(lhs, bindings)?, lower_expr(rhs, bindings)?)
        }
        Expr::Call { function, args, .. } => lower_call(function, args, bindings)?,
        Expr::String { .. } => {
            return Err(KernelLoweringError::UnsupportedExpression(
                "string literal".into(),
            ));
        }
        Expr::Index { .. } => {
            return Err(KernelLoweringError::UnsupportedExpression(
                "indexed tensor expression".into(),
            ));
        }
        Expr::Vector { .. } => {
            return Err(KernelLoweringError::UnsupportedExpression(
                "vector expression".into(),
            ));
        }
    })
}

fn lower_call(
    function: &str,
    args: &[Expr],
    bindings: &BTreeMap<String, OperandId>,
) -> Result<ScalarExpr, KernelLoweringError> {
    let unary = match function {
        "abs" => Some(UnaryOp::Abs),
        "sqrt" => Some(UnaryOp::Sqrt),
        "exp" => Some(UnaryOp::Exp),
        "log" | "ln" => Some(UnaryOp::Ln),
        "sin" => Some(UnaryOp::Sin),
        "cos" => Some(UnaryOp::Cos),
        "tan" => Some(UnaryOp::Tan),
        "floor" => Some(UnaryOp::Floor),
        "ceil" => Some(UnaryOp::Ceil),
        _ => None,
    };
    if let (Some(op), [arg]) = (unary, args) {
        return Ok(ScalarExpr::unary(op, lower_expr(arg, bindings)?));
    }
    let binary = match function {
        "min" => Some(BinaryOp::Min),
        "max" => Some(BinaryOp::Max),
        "atan2" => Some(BinaryOp::Atan2),
        _ => None,
    };
    if let (Some(op), [left, right]) = (binary, args) {
        return Ok(ScalarExpr::binary(
            op,
            lower_expr(left, bindings)?,
            lower_expr(right, bindings)?,
        ));
    }
    Err(KernelLoweringError::UnsupportedExpression(format!(
        "call to `{function}`"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile_variational_form, parse_scientific_module};

    #[test]
    fn scalar_form_integrand_lowers_to_valid_malleus_ir() {
        let module = parse_scientific_module(
            r#"
module kernel.test;
model Reaction {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: unknown scalar H1(order=1) on Omega;
  field v: test scalar H1(order=1) on Omega;
  parameter alpha: Rate;
  form residual { cell(Omega): alpha * u * v; }
}
"#,
        )
        .unwrap();
        let form = compile_variational_form(&module.models[0], "residual").unwrap();
        let program = factor_local_integral(&form, 0).unwrap();
        let kernel = lower_local_program(&program).unwrap();
        assert_eq!(kernel.operands.len(), 4);
        malleus::validate(kernel).unwrap();
    }

    #[test]
    fn differential_operator_is_not_smuggled_through_as_an_opcode() {
        let module = parse_scientific_module(
            r#"
module kernel.test;
model Diffusion {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field u: unknown scalar H1(order=1) on Omega;
  field v: test scalar H1(order=1) on Omega;
  form residual { cell(Omega): dot(grad(u), grad(v)); }
}
"#,
        )
        .unwrap();
        let form = compile_variational_form(&module.models[0], "residual").unwrap();
        assert!(matches!(
            lower_local_program(&factor_local_integral(&form, 0).unwrap()),
            Err(KernelLoweringError::UnsupportedExpression(_))
        ));
    }
}
