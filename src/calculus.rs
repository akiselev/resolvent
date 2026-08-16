use crate::expr::{ExprNode, ExprStore, ScalarLiteral};
use crate::id::{ExprId, SymbolId};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CalculusError {
    #[error("expression id {0} is absent from the store")]
    MissingExpression(u32),
    #[error("cannot evaluate expression: {0}")]
    Evaluation(String),
}

pub fn differentiate(store: &mut ExprStore, root: ExprId, wrt: SymbolId) -> Result<ExprId, CalculusError> {
    let mut memo = BTreeMap::new();
    diff(store, root, wrt, &mut memo)
}

fn diff(store: &mut ExprStore, id: ExprId, wrt: SymbolId, memo: &mut BTreeMap<ExprId, ExprId>) -> Result<ExprId, CalculusError> {
    if let Some(result) = memo.get(&id) { return Ok(*result); }
    let node = store.get(id).cloned().ok_or(CalculusError::MissingExpression(id.0))?;
    let zero = || ScalarLiteral::integer(0);
    let one = || ScalarLiteral::integer(1);
    let result = match node {
        ExprNode::Literal(_) => store.literal(zero()),
        ExprNode::Symbol(symbol) => store.literal(if symbol == wrt { one() } else { zero() }),
        ExprNode::Neg(x) => { let dx = diff(store, x, wrt, memo)?; store.intern(ExprNode::Neg(dx)) }
        ExprNode::Add(xs) => {
            let terms = xs.into_iter().map(|x| diff(store, x, wrt, memo)).collect::<Result<Vec<_>, _>>()?;
            store.add(terms)
        }
        ExprNode::Mul(xs) => {
            let mut sum = Vec::new();
            for i in 0..xs.len() {
                let mut factors = xs.clone();
                factors[i] = diff(store, xs[i], wrt, memo)?;
                sum.push(store.mul(factors));
            }
            store.add(sum)
        }
        ExprNode::Div { numerator, denominator } => {
            let dn = diff(store, numerator, wrt, memo)?;
            let dd = diff(store, denominator, wrt, memo)?;
            let left = store.mul([dn, denominator]);
            let right = store.mul([numerator, dd]);
            let neg_right = store.intern(ExprNode::Neg(right));
            let top = store.add([left, neg_right]);
            let bottom = store.intern(ExprNode::PowI { base: denominator, exponent: 2 });
            store.intern(ExprNode::Div { numerator: top, denominator: bottom })
        }
        ExprNode::PowI { base, exponent } => {
            if exponent == 0 { store.literal(zero()) } else {
                let coefficient = store.literal(ScalarLiteral::integer(exponent as i64));
                let power = store.intern(ExprNode::PowI { base, exponent: exponent - 1 });
                let dbase = diff(store, base, wrt, memo)?;
                store.mul([coefficient, power, dbase])
            }
        }
        ExprNode::Apply { function, args } => {
            if args.len() != 1 {
                store.intern(ExprNode::Derivative { expr: id, with_respect_to: wrt, order: 1 })
            } else {
                let x = args[0];
                let dx = diff(store, x, wrt, memo)?;
                let outer = match function.as_str() {
                    "sin" => store.intern(ExprNode::Apply { function: "cos".into(), args: vec![x] }),
                    "cos" => {
                        let sin = store.intern(ExprNode::Apply { function: "sin".into(), args: vec![x] });
                        store.intern(ExprNode::Neg(sin))
                    }
                    "exp" => store.intern(ExprNode::Apply { function, args: vec![x] }),
                    "log" => {
                        let one = store.literal(one());
                        store.intern(ExprNode::Div { numerator: one, denominator: x })
                    }
                    _ => store.intern(ExprNode::Derivative { expr: id, with_respect_to: wrt, order: 1 }),
                };
                store.mul([outer, dx])
            }
        }
        ExprNode::Derivative { expr, with_respect_to, order } => {
            if with_respect_to == wrt {
                store.intern(ExprNode::Derivative { expr, with_respect_to, order: order.saturating_add(1) })
            } else {
                store.intern(ExprNode::Derivative { expr: id, with_respect_to: wrt, order: 1 })
            }
        }
    };
    memo.insert(id, result);
    Ok(result)
}

pub fn substitute(store: &mut ExprStore, root: ExprId, replacements: &BTreeMap<SymbolId, ExprId>) -> Result<ExprId, CalculusError> {
    let mut memo = BTreeMap::new();
    subst(store, root, replacements, &mut memo)
}

fn subst(store: &mut ExprStore, id: ExprId, replacements: &BTreeMap<SymbolId, ExprId>, memo: &mut BTreeMap<ExprId, ExprId>) -> Result<ExprId, CalculusError> {
    if let Some(out) = memo.get(&id) { return Ok(*out); }
    let node = store.get(id).cloned().ok_or(CalculusError::MissingExpression(id.0))?;
    let out = match node {
        ExprNode::Symbol(symbol) => replacements.get(&symbol).copied().unwrap_or(id),
        ExprNode::Literal(_) => id,
        ExprNode::Neg(x) => { let x = subst(store, x, replacements, memo)?; store.intern(ExprNode::Neg(x)) }
        ExprNode::Add(xs) => { let xs = xs.into_iter().map(|x| subst(store, x, replacements, memo)).collect::<Result<Vec<_>, _>>()?; store.add(xs) }
        ExprNode::Mul(xs) => { let xs = xs.into_iter().map(|x| subst(store, x, replacements, memo)).collect::<Result<Vec<_>, _>>()?; store.mul(xs) }
        ExprNode::Div { numerator, denominator } => { let n = subst(store, numerator, replacements, memo)?; let d = subst(store, denominator, replacements, memo)?; store.intern(ExprNode::Div { numerator: n, denominator: d }) }
        ExprNode::PowI { base, exponent } => { let base = subst(store, base, replacements, memo)?; store.intern(ExprNode::PowI { base, exponent }) }
        ExprNode::Apply { function, args } => { let args = args.into_iter().map(|x| subst(store, x, replacements, memo)).collect::<Result<Vec<_>, _>>()?; store.intern(ExprNode::Apply { function, args }) }
        ExprNode::Derivative { expr, with_respect_to, order } => { let expr = subst(store, expr, replacements, memo)?; store.intern(ExprNode::Derivative { expr, with_respect_to, order }) }
    };
    memo.insert(id, out);
    Ok(out)
}

pub fn evaluate(store: &ExprStore, root: ExprId, values: &BTreeMap<SymbolId, f64>) -> Result<f64, CalculusError> {
    let mut memo = BTreeMap::new();
    eval(store, root, values, &mut memo)
}

fn eval(store: &ExprStore, id: ExprId, values: &BTreeMap<SymbolId, f64>, memo: &mut BTreeMap<ExprId, f64>) -> Result<f64, CalculusError> {
    if let Some(value) = memo.get(&id) { return Ok(*value); }
    let node = store.get(id).ok_or(CalculusError::MissingExpression(id.0))?;
    let value = match node {
        ExprNode::Literal(ScalarLiteral::Integer(v)) => v.parse().map_err(|_| CalculusError::Evaluation(format!("bad integer literal {v}")))?,
        ExprNode::Literal(ScalarLiteral::Rational { numerator, denominator }) => {
            let n: f64 = numerator.parse().map_err(|_| CalculusError::Evaluation("bad rational numerator".into()))?;
            let d: f64 = denominator.parse().map_err(|_| CalculusError::Evaluation("bad rational denominator".into()))?;
            n / d
        }
        ExprNode::Literal(ScalarLiteral::FloatBits(bits)) => f64::from_bits(*bits),
        ExprNode::Literal(ScalarLiteral::NamedConstant(name)) if name == "pi" => std::f64::consts::PI,
        ExprNode::Literal(ScalarLiteral::NamedConstant(name)) if name == "e" => std::f64::consts::E,
        ExprNode::Literal(ScalarLiteral::NamedConstant(name)) => return Err(CalculusError::Evaluation(format!("unknown named constant {name}"))),
        ExprNode::Symbol(symbol) => *values.get(symbol).ok_or_else(|| CalculusError::Evaluation(format!("missing value for symbol {symbol}")))?,
        ExprNode::Neg(x) => -eval(store, *x, values, memo)?,
        ExprNode::Add(xs) => xs.iter().try_fold(0.0, |acc, x| Ok(acc + eval(store, *x, values, memo)?))?,
        ExprNode::Mul(xs) => xs.iter().try_fold(1.0, |acc, x| Ok(acc * eval(store, *x, values, memo)?))?,
        ExprNode::Div { numerator, denominator } => eval(store, *numerator, values, memo)? / eval(store, *denominator, values, memo)?,
        ExprNode::PowI { base, exponent } => eval(store, *base, values, memo)?.powi(*exponent),
        ExprNode::Apply { function, args } if args.len() == 1 => {
            let x = eval(store, args[0], values, memo)?;
            match function.as_str() { "sin" => x.sin(), "cos" => x.cos(), "tan" => x.tan(), "exp" => x.exp(), "log" => x.ln(), "sqrt" => x.sqrt(), _ => return Err(CalculusError::Evaluation(format!("no evaluator for {function}"))) }
        }
        ExprNode::Apply { function, .. } => return Err(CalculusError::Evaluation(format!("no evaluator for {function}"))),
        ExprNode::Derivative { .. } => return Err(CalculusError::Evaluation("semantic derivative must be lowered before numeric evaluation".into())),
    };
    memo.insert(id, value);
    Ok(value)
}
