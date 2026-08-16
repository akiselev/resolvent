use crate::calculus::{CalculusError, differentiate, evaluate_f64};
use crate::expr::{ExprNode, ExprStore};
use crate::id::{ExprId, SymbolId};
use crate::units::Dimension;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GeneratedVerifyError {
    #[error(transparent)] Calculus(#[from] CalculusError),
    #[error("expression {0} is missing")]
    Missing(u32),
    #[error("dimension is not known for symbol {0}")]
    MissingDimension(u32),
    #[error("dimension mismatch in addition: {left} versus {right}")]
    AddDimension { left: Dimension, right: Dimension },
    #[error("function `{0}` has no dimension rule")]
    UnknownFunction(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManufacturedSolution {
    pub substitutions: BTreeMap<SymbolId, ExprId>,
    pub residual: ExprId,
    pub generated_forcing: ExprId,
}

/// Manufacture the forcing that makes the supplied exact fields satisfy `lhs = rhs`.
/// The returned forcing is `lhs[substitutions] - rhs[substitutions]`; callers attach it to
/// the source side with the sign convention of their form.
pub fn manufacture_forcing(
    store: &mut ExprStore,
    lhs: ExprId,
    rhs: ExprId,
    subs: &BTreeMap<SymbolId, ExprId>,
) -> Result<ManufacturedSolution, GeneratedVerifyError> {
    let l = substitute(store, lhs, subs)?;
    let r = substitute(store, rhs, subs)?;
    let nr = store.intern(ExprNode::Neg(r));
    let forcing = store.add([l, nr]);
    Ok(ManufacturedSolution {
        substitutions: subs.clone(),
        residual: forcing,
        generated_forcing: forcing,
    })
}

pub fn substitute(
    store: &mut ExprStore,
    root: ExprId,
    subs: &BTreeMap<SymbolId, ExprId>,
) -> Result<ExprId, GeneratedVerifyError> {
    let mut memo = BTreeMap::new();
    sub_inner(store, root, subs, &mut memo)
}

fn sub_inner(
    store: &mut ExprStore,
    root: ExprId,
    subs: &BTreeMap<SymbolId, ExprId>,
    memo: &mut BTreeMap<ExprId, ExprId>,
) -> Result<ExprId, GeneratedVerifyError> {
    if let Some(v) = memo.get(&root) {
        return Ok(*v);
    }
    let node = store
        .get(root)
        .cloned()
        .ok_or(GeneratedVerifyError::Missing(root.0))?;
    let out = match node {
        ExprNode::Literal(_) => root,
        ExprNode::Symbol(s) => subs.get(&s).copied().unwrap_or(root),
        ExprNode::Neg(x) => {
            let x = sub_inner(store, x, subs, memo)?;
            store.intern(ExprNode::Neg(x))
        }
        ExprNode::Add(xs) => {
            let xs = xs
                .into_iter()
                .map(|x| sub_inner(store, x, subs, memo))
                .collect::<Result<Vec<_>, _>>()?;
            store.add(xs)
        }
        ExprNode::Mul(xs) => {
            let xs = xs
                .into_iter()
                .map(|x| sub_inner(store, x, subs, memo))
                .collect::<Result<Vec<_>, _>>()?;
            store.mul(xs)
        }
        ExprNode::Div {
            numerator,
            denominator,
        } => {
            let numerator = sub_inner(store, numerator, subs, memo)?;
            let denominator = sub_inner(store, denominator, subs, memo)?;
            store.intern(ExprNode::Div {
                numerator,
                denominator,
            })
        }
        ExprNode::PowI { base, exponent } => {
            let base = sub_inner(store, base, subs, memo)?;
            store.intern(ExprNode::PowI { base, exponent })
        }
        ExprNode::Apply { function, args } => {
            let args = args
                .into_iter()
                .map(|x| sub_inner(store, x, subs, memo))
                .collect::<Result<Vec<_>, _>>()?;
            store.intern(ExprNode::Apply { function, args })
        }
        ExprNode::Derivative {
            expr,
            with_respect_to,
            order,
        } => {
            let replaced = sub_inner(store, expr, subs, memo)?;
            let mut d = replaced;
            for _ in 0..order {
                d = differentiate(store, d, with_respect_to)?;
            }
            d
        }
    };
    memo.insert(root, out);
    Ok(out)
}

pub fn infer_dimension(
    store: &ExprStore,
    root: ExprId,
    symbols: &BTreeMap<SymbolId, Dimension>,
) -> Result<Dimension, GeneratedVerifyError> {
    let mut memo = BTreeMap::new();
    dim_inner(store, root, symbols, &mut memo)
}

fn dim_inner(
    store: &ExprStore,
    root: ExprId,
    symbols: &BTreeMap<SymbolId, Dimension>,
    memo: &mut BTreeMap<ExprId, Dimension>,
) -> Result<Dimension, GeneratedVerifyError> {
    if let Some(v) = memo.get(&root) {
        return Ok(*v);
    }
    let node = store
        .get(root)
        .ok_or(GeneratedVerifyError::Missing(root.0))?;
    let d = match node {
        ExprNode::Literal(_) => Dimension::DIMENSIONLESS,
        ExprNode::Symbol(s) => *symbols
            .get(s)
            .ok_or(GeneratedVerifyError::MissingDimension(s.0))?,
        ExprNode::Neg(x) => dim_inner(store, *x, symbols, memo)?,
        ExprNode::Add(xs) => {
            let mut it = xs.iter();
            let first = it
                .next()
                .map(|x| dim_inner(store, *x, symbols, memo))
                .transpose()?
                .unwrap_or(Dimension::DIMENSIONLESS);
            for x in it {
                let other = dim_inner(store, *x, symbols, memo)?;
                if other != first {
                    return Err(GeneratedVerifyError::AddDimension {
                        left: first,
                        right: other,
                    });
                }
            }
            first
        }
        ExprNode::Mul(xs) => xs.iter().try_fold(Dimension::DIMENSIONLESS, |a, x| {
            Ok(a * dim_inner(store, *x, symbols, memo)?)
        })?,
        ExprNode::Div {
            numerator,
            denominator,
        } => {
            dim_inner(store, *numerator, symbols, memo)?
                / dim_inner(store, *denominator, symbols, memo)?
        }
        ExprNode::PowI { base, exponent } => {
            dim_inner(store, *base, symbols, memo)?.powi(*exponent as i8)
        }
        ExprNode::Derivative {
            expr,
            with_respect_to,
            ..
        } => {
            dim_inner(store, *expr, symbols, memo)?
                / symbols
                    .get(with_respect_to)
                    .copied()
                    .ok_or(GeneratedVerifyError::MissingDimension(with_respect_to.0))?
        }
        ExprNode::Apply { function, args } => match function.as_str() {
            "sin" | "cos" | "exp" | "log" => {
                let x = args
                    .first()
                    .map(|x| dim_inner(store, *x, symbols, memo))
                    .transpose()?
                    .unwrap_or(Dimension::DIMENSIONLESS);
                if x != Dimension::DIMENSIONLESS {
                    return Err(GeneratedVerifyError::UnknownFunction(format!(
                        "{function} requires dimensionless argument"
                    )));
                }
                Dimension::DIMENSIONLESS
            }
            "sqrt" => {
                let x = args
                    .first()
                    .map(|x| dim_inner(store, *x, symbols, memo))
                    .transpose()?
                    .unwrap_or(Dimension::DIMENSIONLESS);
                if x.0.iter().any(|e| e % 2 != 0) {
                    return Err(GeneratedVerifyError::UnknownFunction(
                        "sqrt requires even dimension exponents".into(),
                    ));
                }
                Dimension(x.0.map(|e| e / 2))
            }
            "grad" | "div" | "curl" => {
                args.first()
                    .map(|x| dim_inner(store, *x, symbols, memo))
                    .transpose()?
                    .unwrap_or(Dimension::DIMENSIONLESS)
                    / Dimension::LENGTH
            }
            "dot" => {
                if args.len() != 2 {
                    return Err(GeneratedVerifyError::UnknownFunction("dot arity".into()));
                }
                dim_inner(store, args[0], symbols, memo)?
                    * dim_inner(store, args[1], symbols, memo)?
            }
            _ => return Err(GeneratedVerifyError::UnknownFunction(function.clone())),
        },
    };
    memo.insert(root, d);
    Ok(d)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DerivativeGate {
    pub finite_difference: f64,
    pub analytic: f64,
    pub absolute_error: f64,
    pub relative_error: f64,
}

pub fn finite_difference_gate(
    store: &mut ExprStore,
    root: ExprId,
    wrt: SymbolId,
    values: &BTreeMap<SymbolId, f64>,
    step: f64,
) -> Result<DerivativeGate, GeneratedVerifyError> {
    let derivative = differentiate(store, root, wrt)?;
    let analytic = evaluate_f64(store, derivative, values)?;
    let x = *values
        .get(&wrt)
        .ok_or(CalculusError::MissingSymbol(wrt.0))?;
    let mut hi = values.clone();
    let mut lo = values.clone();
    hi.insert(wrt, x + step);
    lo.insert(wrt, x - step);
    let fd = (evaluate_f64(store, root, &hi)? - evaluate_f64(store, root, &lo)?)
        / (2.0 * step);
    let abs = (fd - analytic).abs();
    let rel = abs / analytic.abs().max(fd.abs()).max(1e-300);
    Ok(DerivativeGate {
        finite_difference: fd,
        analytic,
        absolute_error: abs,
        relative_error: rel,
    })
}

pub fn adjoint_dot_gate<F, G>(x: &[f64], y: &[f64], jvp: F, vjp: G) -> f64
where
    F: Fn(&[f64]) -> Vec<f64>,
    G: Fn(&[f64]) -> Vec<f64>,
{
    let jx = jvp(x);
    let jty = vjp(y);
    dot(&jx, y) - dot(x, &jty)
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConvergenceObservation {
    pub h: f64,
    pub error: f64,
}

pub fn observed_orders(obs: &[ConvergenceObservation]) -> Vec<f64> {
    obs.windows(2)
        .filter_map(|w| {
            (w[0].h > 0.0 && w[1].h > 0.0 && w[0].error > 0.0 && w[1].error > 0.0)
                .then(|| (w[0].error / w[1].error).ln() / (w[0].h / w[1].h).ln())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{ScalarLiteral, Symbol, SymbolRole};
    use crate::Context;

    #[test]
    fn manufactured_derivative_is_exact() {
        let mut c = Context::new();
        let t = c.declare_symbol(Symbol {
            name: "t".into(),
            role: SymbolRole::Independent,
            dimension: None,
        });
        let u = c.declare_symbol(Symbol {
            name: "u".into(),
            role: SymbolRole::State,
            dimension: None,
        });
        let et = c.exprs.symbol(t);
        let eu = c.exprs.symbol(u);
        let du = c.exprs.intern(ExprNode::Derivative {
            expr: eu,
            with_respect_to: t,
            order: 1,
        });
        let zero = c.exprs.literal(ScalarLiteral::integer(0));
        let manufactured = c.exprs.intern(ExprNode::PowI {
            base: et,
            exponent: 2,
        });
        let mut subs = BTreeMap::new();
        subs.insert(u, manufactured);
        let m = manufacture_forcing(&mut c.exprs, du, zero, &subs).unwrap();
        let mut vals = BTreeMap::new();
        vals.insert(t, 3.0);
        assert!((evaluate_f64(&c.exprs, m.generated_forcing, &vals).unwrap() - 6.0).abs() < 1e-12);
    }

    #[test]
    fn dot_gate_detects_transpose_identity() {
        let e = adjoint_dot_gate(
            &[1.0, 2.0],
            &[3.0, 4.0],
            |x| vec![2.0 * x[0], 5.0 * x[1]],
            |y| vec![2.0 * y[0], 5.0 * y[1]],
        );
        assert!(e.abs() < 1e-12);
    }

    #[test]
    fn order_is_computed() {
        let o = observed_orders(&[
            ConvergenceObservation { h: 0.5, error: 0.25 },
            ConvergenceObservation {
                h: 0.25,
                error: 0.0625,
            },
        ]);
        assert!((o[0] - 2.0).abs() < 1e-12);
    }
}
