use crate::latex::{MathExpr, parse_scientific_latex};
use crate::scientific::{
    BinaryOp, Expr, PropertyDefinition, ScientificError, ScientificModule, UnaryOp,
    parse_scientific_module, semantic_digest,
};
use crate::source::SourceDiagnostic;
use serde::{Deserialize, Serialize};

/// Evidence carried into the frozen scientific-v1 physics identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyEvidenceLock {
    pub property_id: String,
    pub sources: Vec<String>,
    pub dataset_digest: Option<String>,
    pub fit_digest: Option<String>,
    pub uncertainty_digest: Option<String>,
}

/// Frozen scientific-v1 identity used by Sinbad execution artifacts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScientificPhysicsLock {
    pub schema: &'static str,
    pub source_digest: String,
    pub semantic_digest: String,
    pub compiler_schema: &'static str,
    pub property_evidence: Vec<PropertyEvidenceLock>,
    pub digest: String,
}

pub fn freeze_scientific(
    source: &str,
    module: &ScientificModule,
    properties: &[PropertyDefinition],
) -> ScientificPhysicsLock {
    let mut property_evidence = properties
        .iter()
        .map(|property| PropertyEvidenceLock {
            property_id: property.signature.id.clone(),
            sources: property.evidence.sources.clone(),
            dataset_digest: property.evidence.dataset_digest.clone(),
            fit_digest: property.evidence.fit_digest.clone(),
            uncertainty_digest: property.evidence.uncertainty.as_ref().map(|uncertainty| {
                let bytes = serde_json::to_vec(uncertainty)
                    .expect("uncertainty serialization is infallible");
                blake3::hash(&bytes).to_hex().to_string()
            }),
        })
        .collect::<Vec<_>>();
    property_evidence.sort_by(|a, b| a.property_id.cmp(&b.property_id));
    let source_digest = blake3::hash(source.as_bytes()).to_hex().to_string();
    let semantic_digest = semantic_digest(module);
    let payload = serde_json::to_vec(&(
        "resolvent-scientific-physics-lock/1",
        &source_digest,
        &semantic_digest,
        crate::SCIENTIFIC_SCHEMA_VERSION,
        &property_evidence,
    ))
    .expect("scientific lock serialization is infallible");
    let digest = blake3::hash(&payload).to_hex().to_string();
    ScientificPhysicsLock {
        schema: "resolvent-scientific-physics-lock/1",
        source_digest,
        semantic_digest,
        compiler_schema: crate::SCIENTIFIC_SCHEMA_VERSION,
        property_evidence,
        digest,
    }
}

pub fn parse_and_freeze_scientific(
    source: &str,
    properties: &[PropertyDefinition],
) -> Result<(ScientificModule, ScientificPhysicsLock), Vec<ScientificError>> {
    let module = parse_scientific_module(source)?;
    let lock = freeze_scientific(source, &module, properties);
    Ok((module, lock))
}

/// Convert the constrained scientific-LaTeX source AST into the same scientific-v1 expression
/// nodes used by textual RSL. Unsupported TeX is rejected by `parse_scientific_latex` before this
/// bridge is entered; this function therefore contains no heuristic TeX interpretation.
pub fn parse_scientific_latex_expr(input: &str) -> Result<Expr, Vec<SourceDiagnostic>> {
    parse_scientific_latex(input).and_then(|expr| {
        lower_math(expr).map_err(|message| {
            vec![
                SourceDiagnostic::error(
                    "RSL-LATEX-005",
                    message,
                    crate::source::SourceSpan::new(0, input.len()),
                )
                .phase("latex"),
            ]
        })
    })
}

fn lower_math(expr: MathExpr) -> Result<Expr, String> {
    fn fold(op: BinaryOp, mut values: Vec<MathExpr>) -> Result<Expr, String> {
        let first = values
            .is_empty()
            .then_some(())
            .map(|_| Err("empty variadic mathematical expression".to_owned()))
            .unwrap_or_else(|| lower_math(values.remove(0)))?;
        values.into_iter().try_fold(first, |lhs, rhs| {
            Ok(Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(lower_math(rhs)?),
            })
        })
    }
    Ok(match expr {
        MathExpr::Number(value) => Expr::Number {
            value: value
                .parse::<f64>()
                .map_err(|_| format!("invalid numeric literal `{value}`"))?,
            unit: None,
        },
        MathExpr::Name(name) => Expr::Name(name),
        MathExpr::Neg(arg) => Expr::Unary {
            op: UnaryOp::Neg,
            arg: Box::new(lower_math(*arg)?),
        },
        MathExpr::Add(values) => fold(BinaryOp::Add, values)?,
        MathExpr::Mul(values) => fold(BinaryOp::Mul, values)?,
        MathExpr::Div(lhs, rhs) => Expr::Binary {
            op: BinaryOp::Div,
            lhs: Box::new(lower_math(*lhs)?),
            rhs: Box::new(lower_math(*rhs)?),
        },
        MathExpr::Pow(lhs, exponent) => Expr::Binary {
            op: BinaryOp::Pow,
            lhs: Box::new(lower_math(*lhs)?),
            rhs: Box::new(Expr::Number {
                value: exponent as f64,
                unit: None,
            }),
        },
        MathExpr::Call { name, args } => Expr::Call {
            function: name,
            args: args
                .into_iter()
                .map(lower_math)
                .collect::<Result<Vec<_>, _>>()?,
        },
        MathExpr::Grad(arg) => Expr::Call {
            function: "grad".into(),
            args: vec![lower_math(*arg)?],
        },
        MathExpr::DivOp(arg) => Expr::Call {
            function: "div".into(),
            args: vec![lower_math(*arg)?],
        },
        MathExpr::Curl(arg) => Expr::Call {
            function: "curl".into(),
            args: vec![lower_math(*arg)?],
        },
        MathExpr::Dt(arg) => Expr::Call {
            function: "dt".into(),
            args: vec![lower_math(*arg)?],
        },
        MathExpr::Dot(lhs, rhs) => Expr::Call {
            function: "dot".into(),
            args: vec![lower_math(*lhs)?, lower_math(*rhs)?],
        },
    })
}

/// Embed a scientific-v1 `.res` file while retaining the exact same parser/freeze implementation
/// used by `resolvent-science` and Sinbad Lab. The macro never constructs semantic IR itself.
#[macro_export]
macro_rules! include_scientific {
    ($vis:vis $name:ident = $path:literal) => {
        $vis struct $name;
        impl $name {
            pub const SOURCE: &'static str = include_str!($path);
            pub fn parse() -> Result<$crate::ScientificModule, Vec<$crate::scientific::ScientificError>> {
                $crate::parse_scientific_module(Self::SOURCE)
            }
            pub fn semantic_digest() -> Result<String, Vec<$crate::scientific::ScientificError>> {
                Self::parse().map(|module| $crate::semantic_digest(&module))
            }
            pub fn freeze() -> Result<$crate::ScientificPhysicsLock, Vec<$crate::scientific::ScientificError>> {
                Self::parse().map(|module| $crate::freeze_scientific(Self::SOURCE, &module, &[]))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrained_latex_lowers_to_scientific_expression_nodes() {
        let parsed = parse_scientific_latex_expr(r"\nabla \cdot (k * \nabla T)").unwrap();
        assert_eq!(
            parsed,
            Expr::Call {
                function: "div".into(),
                args: vec![Expr::Binary {
                    op: BinaryOp::Mul,
                    lhs: Box::new(Expr::Name("k".into())),
                    rhs: Box::new(Expr::Call {
                        function: "grad".into(),
                        args: vec![Expr::Name("T".into())],
                    }),
                }],
            }
        );
    }
}
