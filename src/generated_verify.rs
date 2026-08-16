use crate::calculus::{CalculusError, differentiate, substitute};
use crate::expr::{ExprNode, ExprStore};
use crate::id::{ExprId, SymbolId};
use crate::model::Equation;
use crate::reference::SparseMatrix;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DerivativeCheck {
    pub passed: bool,
    pub absolute_error: f64,
    pub relative_error: f64,
    pub epsilon: f64,
}

pub fn check_jvp(
    residual: impl Fn(&[f64]) -> Vec<f64>,
    jvp: impl Fn(&[f64], &[f64]) -> Vec<f64>,
    state: &[f64],
    direction: &[f64],
    epsilon: f64,
    tolerance: f64,
) -> DerivativeCheck {
    let plus: Vec<f64> = state
        .iter()
        .zip(direction)
        .map(|(u, d)| u + epsilon * d)
        .collect();
    let minus: Vec<f64> = state
        .iter()
        .zip(direction)
        .map(|(u, d)| u - epsilon * d)
        .collect();
    let rp = residual(&plus);
    let rm = residual(&minus);
    let analytic = jvp(state, direction);
    let mut abs = 0.0f64;
    let mut scale = 0.0f64;
    for ((p, m), a) in rp.iter().zip(&rm).zip(&analytic) {
        let fd = (p - m) / (2.0 * epsilon);
        abs = abs.max((fd - a).abs());
        scale = scale.max(fd.abs().max(a.abs()));
    }
    let rel = if scale == 0.0 { abs } else { abs / scale };
    DerivativeCheck {
        passed: abs <= tolerance || rel <= tolerance,
        absolute_error: abs,
        relative_error: rel,
        epsilon,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdjointCheck {
    pub passed: bool,
    pub forward_dot: f64,
    pub transpose_dot: f64,
    pub absolute_error: f64,
}
pub fn check_transpose(
    matrix: &SparseMatrix,
    x: &[f64],
    y: &[f64],
    tolerance: f64,
) -> Result<AdjointCheck, crate::reference::ReferenceError> {
    let ax = matrix.apply(x)?;
    let aty = matrix.transpose_apply(y)?;
    let lhs = dot(&ax, y);
    let rhs = dot(x, &aty);
    let error = (lhs - rhs).abs();
    Ok(AdjointCheck {
        passed: error <= tolerance,
        forward_dot: lhs,
        transpose_dot: rhs,
        absolute_error: error,
    })
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceEstimate {
    pub coarse_error: f64,
    pub fine_error: f64,
    pub refinement_ratio: f64,
    pub observed_order: Option<f64>,
}
pub fn convergence_order(coarse: f64, fine: f64, ratio: f64) -> ConvergenceEstimate {
    let order = if coarse > 0.0 && fine > 0.0 && ratio > 1.0 {
        Some((coarse / fine).ln() / ratio.ln())
    } else {
        None
    };
    ConvergenceEstimate {
        coarse_error: coarse,
        fine_error: fine,
        refinement_ratio: ratio,
        observed_order: order,
    }
}

/// Substitute manufactured fields/parameters and resolve semantic derivatives by exact
/// differentiation in the expression DAG. Spatial differential operators represented as
/// named Apply nodes remain explicit for the form compiler; time/coordinate derivatives are
/// eliminated here.
pub fn manufacture_equation(
    store: &mut ExprStore,
    equation: &Equation,
    replacements: &BTreeMap<SymbolId, ExprId>,
) -> Result<Equation, CalculusError> {
    let lhs = substitute(store, equation.lhs, replacements)?;
    let rhs = substitute(store, equation.rhs, replacements)?;
    let lhs = resolve_derivatives(store, lhs)?;
    let rhs = resolve_derivatives(store, rhs)?;
    Ok(Equation {
        lhs,
        rhs,
        label: equation.label.clone(),
    })
}

pub fn resolve_derivatives(store: &mut ExprStore, root: ExprId) -> Result<ExprId, CalculusError> {
    let node = store
        .get(root)
        .cloned()
        .ok_or(CalculusError::MissingExpression(root.0))?;
    Ok(match node {
        ExprNode::Derivative {
            expr,
            with_respect_to,
            order,
        } => {
            let mut current = resolve_derivatives(store, expr)?;
            for _ in 0..order {
                current = differentiate(store, current, with_respect_to)?;
            }
            current
        }
        ExprNode::Neg(x) => {
            let x = resolve_derivatives(store, x)?;
            store.intern(ExprNode::Neg(x))
        }
        ExprNode::Add(xs) => {
            let xs = xs
                .into_iter()
                .map(|x| resolve_derivatives(store, x))
                .collect::<Result<Vec<_>, _>>()?;
            store.add(xs)
        }
        ExprNode::Mul(xs) => {
            let xs = xs
                .into_iter()
                .map(|x| resolve_derivatives(store, x))
                .collect::<Result<Vec<_>, _>>()?;
            store.mul(xs)
        }
        ExprNode::Div {
            numerator,
            denominator,
        } => {
            let n = resolve_derivatives(store, numerator)?;
            let d = resolve_derivatives(store, denominator)?;
            store.intern(ExprNode::Div {
                numerator: n,
                denominator: d,
            })
        }
        ExprNode::PowI { base, exponent } => {
            let base = resolve_derivatives(store, base)?;
            store.intern(ExprNode::PowI { base, exponent })
        }
        ExprNode::Apply { function, args } => {
            let args = args
                .into_iter()
                .map(|x| resolve_derivatives(store, x))
                .collect::<Result<Vec<_>, _>>()?;
            store.intern(ExprNode::Apply { function, args })
        }
        ExprNode::Literal(_) | ExprNode::Symbol(_) => root,
    })
}
