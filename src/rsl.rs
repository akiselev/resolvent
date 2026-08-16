use crate::Context;
use crate::expr::{ExprNode, ScalarLiteral, Symbol, SymbolRole};
use crate::field::{DomainRef, Field, FieldRole, FunctionSpace};
use crate::id::{ExprId, SymbolId};
use crate::latex::{MathExpr, parse_scientific_latex};
use crate::model::{Equation, ScientificSpec, Scope, System};
use crate::source::{SourceDiagnostic, SourceSpan, Spanned};
use crate::units::Dimension;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RslFieldDecl {
    pub name: String,
    pub role: FieldRole,
    pub space: FunctionSpace,
    pub dimension: Option<Dimension>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RslSymbolDecl {
    pub name: String,
    pub role: SymbolRole,
    pub dimension: Option<Dimension>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RslModel {
    pub name: String,
    pub domains: Vec<DomainRef>,
    pub fields: Vec<RslFieldDecl>,
    pub symbols: Vec<RslSymbolDecl>,
    pub equations: Vec<Spanned<String>>,
    #[serde(default)]
    pub observables: Vec<Spanned<String>>,
    #[serde(default)]
    pub properties: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElaboratedRsl {
    pub spec: ScientificSpec,
    pub system: System,
    pub fields: Vec<Field>,
    pub source_map: BTreeMap<String, SourceSpan>,
}

pub fn parse_rsl(input: &str) -> Result<RslModel, Vec<SourceDiagnostic>> {
    let trimmed = input.trim();
    let Some(model_kw) = trimmed.strip_prefix("model") else {
        return Err(vec![
            SourceDiagnostic::error(
                "RSL-P001",
                "file must start with `model <name> { ... }`",
                SourceSpan::new(0, input.len().min(5)),
            )
            .phase("parse"),
        ]);
    };
    let model_kw = model_kw.trim_start();
    let name_end = model_kw
        .find(|c: char| c.is_whitespace() || c == '{')
        .unwrap_or(model_kw.len());
    let name = model_kw[..name_end].trim();
    if name.is_empty() {
        return Err(vec![
            SourceDiagnostic::error(
                "RSL-P002",
                "model name is required",
                SourceSpan::new(0, input.len().min(8)),
            )
            .phase("parse"),
        ]);
    }
    let brace_rel = model_kw[name_end..].find('{').ok_or_else(|| {
        vec![
            SourceDiagnostic::error(
                "RSL-P003",
                "missing `{` after model name",
                SourceSpan::new(0, input.len()),
            )
            .phase("parse"),
        ]
    })?;
    let body_start_in_trim = "model".len()
        + (trimmed["model".len()..].len() - model_kw.len())
        + name_end
        + brace_rel
        + 1;
    let close = trimmed.rfind('}').ok_or_else(|| {
        vec![
            SourceDiagnostic::error(
                "RSL-P004",
                "missing final `}`",
                SourceSpan::new(0, input.len()),
            )
            .phase("parse"),
        ]
    })?;
    if close < body_start_in_trim {
        return Err(vec![
            SourceDiagnostic::error(
                "RSL-P004",
                "malformed model body",
                SourceSpan::new(0, input.len()),
            )
            .phase("parse"),
        ]);
    }
    let body = &trimmed[body_start_in_trim..close];
    let base_offset = input.find(body).unwrap_or(0);
    let statements = split_statements(body, base_offset)?;

    let mut model = RslModel {
        name: name.to_string(),
        domains: vec![],
        fields: vec![],
        symbols: vec![],
        equations: vec![],
        observables: vec![],
        properties: vec![],
    };
    let mut diagnostics = vec![];
    for statement in statements {
        let text = statement.value.trim();
        if text.is_empty() {
            continue;
        }
        if let Some(rest) = text.strip_prefix("domain ") {
            let mut pieces = rest.split_whitespace();
            let domain_name = pieces.next().unwrap_or_default().trim_matches(';');
            let dim = pieces
                .next()
                .and_then(|s| s.strip_prefix("dim="))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);
            model.domains.push(DomainRef {
                name: domain_name.to_string(),
                topological_dimension: dim,
                geometric_dimension: None,
            });
        } else if text.starts_with("field ") {
            match parse_field(text, statement.span) {
                Ok(v) => model.fields.push(v),
                Err(d) => diagnostics.push(d),
            }
        } else if text.starts_with("parameter ")
            || text.starts_with("coefficient ")
            || text.starts_with("source ")
            || text.starts_with("state ")
            || text.starts_with("algebraic ")
        {
            match parse_symbol(text, statement.span) {
                Ok(v) => model.symbols.push(v),
                Err(d) => diagnostics.push(d),
            }
        } else if let Some(rest) = text.strip_prefix("equation ") {
            match extract_latex(rest, statement.span) {
                Ok(v) => model.equations.push(v),
                Err(d) => diagnostics.push(d),
            }
        } else if let Some(rest) = text.strip_prefix("observable ") {
            match extract_latex(rest, statement.span) {
                Ok(v) => model.observables.push(v),
                Err(d) => diagnostics.push(d),
            }
        } else if let Some(rest) = text.strip_prefix("property ") {
            model
                .properties
                .push(rest.trim().trim_end_matches(';').to_string());
        } else {
            diagnostics.push(SourceDiagnostic::error("RSL-P010", format!("unknown declaration `{}`", text.lines().next().unwrap_or(text)), statement.span).hint("expected domain, field, state/parameter/coefficient/source/algebraic, equation, observable or property").phase("parse"));
        }
    }
    if diagnostics.is_empty() {
        Ok(model)
    } else {
        Err(diagnostics)
    }
}

fn split_statements(
    body: &str,
    base: usize,
) -> Result<Vec<Spanned<String>>, Vec<SourceDiagnostic>> {
    let bytes = body.as_bytes();
    let mut out = vec![];
    let mut start = 0usize;
    let mut i = 0usize;
    let mut braces = 0i32;
    let mut triple = false;
    while i < bytes.len() {
        if i + 2 < bytes.len() && &body[i..i + 3] == "\"\"\"" {
            triple = !triple;
            i += 3;
            continue;
        }
        if !triple {
            match bytes[i] as char {
                '{' => braces += 1,
                '}' => braces -= 1,
                ';' if braces == 0 => {
                    out.push(Spanned {
                        value: body[start..i].to_string(),
                        span: SourceSpan::new(base + start, base + i + 1),
                    });
                    start = i + 1;
                }
                _ => {}
            }
        }
        i += 1;
    }
    if triple {
        return Err(vec![
            SourceDiagnostic::error(
                "RSL-P005",
                "unterminated triple-quoted LaTeX block",
                SourceSpan::new(base + start, base + body.len()),
            )
            .phase("parse"),
        ]);
    }
    if !body[start..].trim().is_empty() {
        out.push(Spanned {
            value: body[start..].to_string(),
            span: SourceSpan::new(base + start, base + body.len()),
        });
    }
    Ok(out)
}

#[allow(clippy::result_large_err)]
fn parse_field(text: &str, span: SourceSpan) -> Result<RslFieldDecl, SourceDiagnostic> {
    // Compact forms:
    //   field T: state H1(1) [K] on Omega
    //   field u: state vector(2) H1(1) [m] on Omega
    //   field E: state vector(2) HCurl(0) [V / m] on Omega
    let rest = text.strip_prefix("field ").unwrap().trim();
    let Some((name, attrs)) = rest.split_once(':') else {
        return Err(SourceDiagnostic::error(
            "RSL-F001",
            "field requires `field <name>: <role> [shape] <space> [unit] on <domain>`",
            span,
        )
        .phase("elaborate"));
    };
    let attrs = attrs.trim();
    let role_word = attrs.split_whitespace().next().unwrap_or("unknown");
    let role = match role_word {
        "state" => FieldRole::State,
        "unknown" => FieldRole::Unknown,
        "coefficient" => FieldRole::Coefficient,
        "parameter" => FieldRole::Parameter,
        "test" => FieldRole::Test,
        "trial" => FieldRole::Trial,
        "derived" => FieldRole::Derived,
        _ => {
            return Err(SourceDiagnostic::error(
                "RSL-F002",
                format!("unknown field role `{role_word}`"),
                span,
            )
            .phase("elaborate"));
        }
    };
    let domain = attrs.split(" on ").nth(1).unwrap_or("Omega").trim();
    let vector_dim = attrs
        .find("vector(")
        .and_then(|p| attrs[p + 7..].split(')').next())
        .and_then(|x| x.parse::<u8>().ok());
    let parse_order = |tag: &str, default: u8| {
        attrs
            .find(tag)
            .and_then(|p| attrs[p + tag.len()..].split(')').next())
            .and_then(|x| x.parse::<u8>().ok())
            .unwrap_or(default)
    };
    let space = if attrs.contains("HCurl(") {
        FunctionSpace::hcurl_nedelec(parse_order("HCurl(", 0), vector_dim.unwrap_or(2), domain)
    } else if attrs.contains("HDiv(") {
        FunctionSpace::hdiv_raviart_thomas(parse_order("HDiv(", 0), vector_dim.unwrap_or(2), domain)
    } else if attrs.contains("L2(") {
        FunctionSpace::l2_discontinuous(
            parse_order("L2(", 0),
            vector_dim.map_or(crate::field::ValueShape::Scalar, |dim| {
                crate::field::ValueShape::Vector { dim }
            }),
            domain,
        )
    } else if attrs.contains("H1(") {
        let order = parse_order("H1(", 1);
        match vector_dim {
            Some(dim) => FunctionSpace::h1_lagrange_vector(order, dim, domain),
            None => FunctionSpace::h1_lagrange(order, domain),
        }
    } else {
        return Err(SourceDiagnostic::error(
            "RSL-F003",
            "unsupported function space; expected H1(n), HCurl(n), HDiv(n), or L2(n)",
            span,
        )
        .phase("elaborate"));
    };
    let dimension = parse_bracket_unit(attrs, span)?;
    Ok(RslFieldDecl {
        name: name.trim().to_string(),
        role,
        space,
        dimension,
        span,
    })
}

#[allow(clippy::result_large_err)]
fn parse_symbol(text: &str, span: SourceSpan) -> Result<RslSymbolDecl, SourceDiagnostic> {
    let (kind, rest) = text.split_once(' ').unwrap();
    let name = rest
        .split(|c: char| c.is_whitespace() || c == '[' || c == '(')
        .next()
        .unwrap_or_default();
    if name.is_empty() {
        return Err(
            SourceDiagnostic::error("RSL-S001", "symbol name is required", span).phase("elaborate"),
        );
    }
    let role = match kind {
        "parameter" | "coefficient" | "source" => SymbolRole::Parameter,
        "state" => SymbolRole::State,
        "algebraic" => SymbolRole::Algebraic,
        _ => SymbolRole::Auxiliary,
    };
    let dimension = parse_bracket_unit(rest, span)?;
    Ok(RslSymbolDecl {
        name: name.to_string(),
        role,
        dimension,
        span,
    })
}

#[allow(clippy::result_large_err)]
fn parse_bracket_unit(text: &str, span: SourceSpan) -> Result<Option<Dimension>, SourceDiagnostic> {
    let Some(open) = text.find('[') else {
        return Ok(None);
    };
    let Some(close_rel) = text[open + 1..].find(']') else {
        return Err(
            SourceDiagnostic::error("RSL-U001", "unterminated unit annotation", span)
                .phase("units"),
        );
    };
    let unit = &text[open + 1..open + 1 + close_rel];
    Dimension::parse(unit)
        .map(Some)
        .map_err(|e| SourceDiagnostic::error("RSL-U002", e.to_string(), span).phase("units"))
}

#[allow(clippy::result_large_err)]
fn extract_latex(text: &str, span: SourceSpan) -> Result<Spanned<String>, SourceDiagnostic> {
    let text = text.trim();
    let Some(rest) = text.strip_prefix("latex") else {
        return Err(SourceDiagnostic::error(
            "RSL-L001",
            "equations and observables require `latex ...`",
            span,
        )
        .phase("parse"));
    };
    let rest = rest.trim();
    let value = if rest.starts_with("\"\"\"") && rest.ends_with("\"\"\"") && rest.len() >= 6 {
        &rest[3..rest.len() - 3]
    } else if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
        &rest[1..rest.len() - 1]
    } else {
        return Err(
            SourceDiagnostic::error("RSL-L002", "latex expression must be quoted", span)
                .phase("parse"),
        );
    };
    Ok(Spanned {
        value: value.trim().to_string(),
        span,
    })
}

impl RslModel {
    pub fn elaborate(&self, ctx: &mut Context) -> Result<ElaboratedRsl, Vec<SourceDiagnostic>> {
        let mut ids = BTreeMap::<String, SymbolId>::new();
        let mut source_map = BTreeMap::new();
        let mut diagnostics = vec![];
        let time = ctx.declare_symbol(Symbol {
            name: "t".into(),
            role: SymbolRole::Independent,
            dimension: Some("s".into()),
        });
        ids.insert("t".into(), time);
        let mut fields = vec![];
        for decl in &self.fields {
            let id = ctx.declare_symbol(Symbol {
                name: decl.name.clone(),
                role: field_symbol_role(decl.role),
                dimension: decl.dimension.map(|d| d.to_string()),
            });
            ids.insert(decl.name.clone(), id);
            source_map.insert(decl.name.clone(), decl.span);
            fields.push(Field {
                id: ctx.allocate_field_id(),
                name: decl.name.clone(),
                role: decl.role,
                space: decl.space.clone(),
                dimension: decl.dimension,
                metadata: BTreeMap::new(),
            });
        }
        for decl in &self.symbols {
            let id = ctx.declare_symbol(Symbol {
                name: decl.name.clone(),
                role: decl.role,
                dimension: decl.dimension.map(|d| d.to_string()),
            });
            ids.insert(decl.name.clone(), id);
            source_map.insert(decl.name.clone(), decl.span);
        }
        let mut equations = vec![];
        for (index, equation) in self.equations.iter().enumerate() {
            let Some((lhs_text, rhs_text)) = split_equation(&equation.value) else {
                diagnostics.push(
                    SourceDiagnostic::error(
                        "RSL-E001",
                        "equation must contain exactly one top-level `=`",
                        equation.span,
                    )
                    .phase("elaborate"),
                );
                continue;
            };
            let lhs = match parse_scientific_latex(lhs_text)
                .and_then(|m| lower_math(ctx, &ids, time, &m, equation.span))
            {
                Ok(v) => v,
                Err(mut d) => {
                    diagnostics.append(&mut d);
                    continue;
                }
            };
            let rhs = match parse_scientific_latex(rhs_text)
                .and_then(|m| lower_math(ctx, &ids, time, &m, equation.span))
            {
                Ok(v) => v,
                Err(mut d) => {
                    diagnostics.append(&mut d);
                    continue;
                }
            };
            equations.push(Equation {
                lhs,
                rhs,
                label: Some(format!("rsl_equation_{index}")),
            });
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let unknowns = ids
            .iter()
            .filter_map(|(name, id)| {
                let symbol = ctx.symbols.get(*id)?;
                matches!(symbol.role, SymbolRole::State | SymbolRole::Algebraic)
                    .then_some((name.clone(), *id))
            })
            .map(|(_, id)| id)
            .collect();
        let parameters = ids
            .iter()
            .filter_map(|(_, id)| {
                ctx.symbols
                    .get(*id)
                    .and_then(|s| (s.role == SymbolRole::Parameter).then_some(*id))
            })
            .collect();
        let system = System {
            name: self.name.clone(),
            unknowns,
            parameters,
            equations,
            events: vec![],
            children: vec![],
            metadata: BTreeMap::from([("authoring_language".into(), "rsl/0.1".into())]),
        };
        let system_id = ctx.insert_system(system.clone());
        let spec = ScientificSpec {
            name: self.name.clone(),
            model: system_id,
            assumptions: vec![],
            scope: Scope {
                domain: self.domains.first().map(|d| d.name.clone()),
                ..Default::default()
            },
            observables: vec![],
            properties: vec![],
            sources: vec!["rsl".into()],
            metadata: BTreeMap::new(),
        };
        Ok(ElaboratedRsl {
            spec,
            system,
            fields,
            source_map,
        })
    }
}

fn field_symbol_role(role: FieldRole) -> SymbolRole {
    match role {
        FieldRole::State | FieldRole::Unknown | FieldRole::Trial => SymbolRole::State,
        FieldRole::Parameter | FieldRole::Coefficient => SymbolRole::Parameter,
        _ => SymbolRole::Auxiliary,
    }
}

fn split_equation(text: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let mut found = None;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => depth -= 1,
            '=' if depth == 0 => {
                if found.is_some() {
                    return None;
                }
                found = Some(i);
            }
            _ => {}
        }
    }
    found.map(|i| (&text[..i], &text[i + 1..]))
}

fn lower_math(
    ctx: &mut Context,
    ids: &BTreeMap<String, SymbolId>,
    time: SymbolId,
    m: &MathExpr,
    span: SourceSpan,
) -> Result<ExprId, Vec<SourceDiagnostic>> {
    let e = match m {
        MathExpr::Number(n) => {
            if let Ok(i) = n.parse::<i64>() {
                ctx.exprs.literal(ScalarLiteral::integer(i))
            } else if let Ok(f) = n.parse::<f64>() {
                ctx.exprs
                    .literal(ScalarLiteral::f64_exact(f).ok_or_else(|| {
                        vec![
                            SourceDiagnostic::error("RSL-N001", "non-finite literal", span)
                                .phase("elaborate"),
                        ]
                    })?)
            } else {
                return Err(vec![
                    SourceDiagnostic::error(
                        "RSL-N002",
                        format!("invalid numeric literal `{n}`"),
                        span,
                    )
                    .phase("elaborate"),
                ]);
            }
        }
        MathExpr::Name(name) => {
            let Some(id) = ids.get(name) else {
                return Err(vec![
                    SourceDiagnostic::error("RSL-N003", format!("unknown symbol `{name}`"), span)
                        .hint("declare the variable and its units before using it")
                        .phase("elaborate"),
                ]);
            };
            ctx.exprs.symbol(*id)
        }
        MathExpr::Neg(x) => {
            let x = lower_math(ctx, ids, time, x, span)?;
            ctx.exprs.intern(ExprNode::Neg(x))
        }
        MathExpr::Add(xs) => {
            let xs = xs
                .iter()
                .map(|x| lower_math(ctx, ids, time, x, span))
                .collect::<Result<Vec<_>, _>>()?;
            ctx.exprs.add(xs)
        }
        MathExpr::Mul(xs) => {
            let xs = xs
                .iter()
                .map(|x| lower_math(ctx, ids, time, x, span))
                .collect::<Result<Vec<_>, _>>()?;
            ctx.exprs.mul(xs)
        }
        MathExpr::Div(a, b) => {
            let a = lower_math(ctx, ids, time, a, span)?;
            let b = lower_math(ctx, ids, time, b, span)?;
            ctx.exprs.intern(ExprNode::Div {
                numerator: a,
                denominator: b,
            })
        }
        MathExpr::Pow(a, p) => {
            let a = lower_math(ctx, ids, time, a, span)?;
            ctx.exprs.intern(ExprNode::PowI {
                base: a,
                exponent: *p,
            })
        }
        MathExpr::Call { name, args } => {
            let args = args
                .iter()
                .map(|x| lower_math(ctx, ids, time, x, span))
                .collect::<Result<Vec<_>, _>>()?;
            ctx.exprs.intern(ExprNode::Apply {
                function: name.clone(),
                args,
            })
        }
        MathExpr::Grad(x) => unary_apply(ctx, ids, time, "grad", x, span)?,
        MathExpr::DivOp(x) => unary_apply(ctx, ids, time, "div", x, span)?,
        MathExpr::Curl(x) => unary_apply(ctx, ids, time, "curl", x, span)?,
        MathExpr::Dt(x) => {
            let x = lower_math(ctx, ids, time, x, span)?;
            ctx.exprs.intern(ExprNode::Derivative {
                expr: x,
                with_respect_to: time,
                order: 1,
            })
        }
        MathExpr::Dot(a, b) => {
            let a = lower_math(ctx, ids, time, a, span)?;
            let b = lower_math(ctx, ids, time, b, span)?;
            ctx.exprs.intern(ExprNode::Apply {
                function: "dot".into(),
                args: vec![a, b],
            })
        }
    };
    Ok(e)
}

fn unary_apply(
    ctx: &mut Context,
    ids: &BTreeMap<String, SymbolId>,
    time: SymbolId,
    name: &str,
    x: &MathExpr,
    span: SourceSpan,
) -> Result<ExprId, Vec<SourceDiagnostic>> {
    let x = lower_math(ctx, ids, time, x, span)?;
    Ok(ctx.exprs.intern(ExprNode::Apply {
        function: name.into(),
        args: vec![x],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_elaborates_heat_model() {
        let src = r#"model Heat {
            domain Omega dim=2;
            field T: state H1(1) [K] on Omega;
            parameter rho [kg / m^3];
            coefficient cp(T) [J / (kg K)];
            equation latex "rho cp(T) \\frac{\\partial T}{\\partial t} = 0";
        }"#;
        let model = parse_rsl(src).unwrap();
        let mut ctx = Context::new();
        let out = model.elaborate(&mut ctx).unwrap();
        assert_eq!(out.system.equations.len(), 1);
        assert_eq!(out.fields.len(), 1);
    }
}
