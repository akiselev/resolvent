use crate::Context;
use crate::diagnostic::{Diagnostic, SourceSpan};
use crate::expr::{ExprNode, ScalarLiteral, Symbol, SymbolRole};
use crate::id::{ExprId, ObservableId, SymbolId};
use crate::latex::{self, MathEquation, MathExpr};
use crate::model::{
    Assumption, Equation, Observable, PropertyContract, PropertyKind, ScientificSpec, Scope, System,
};
use crate::units::{Dimension, UnitExpr, parse_unit};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclKind {
    Field,
    Parameter,
    Coefficient,
    Source,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableDecl {
    pub kind: DeclKind,
    pub name: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<UnitExpr>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EquationDecl {
    pub latex: String,
    pub parsed: MathEquation,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundaryDecl {
    pub name: String,
    pub equations: Vec<EquationDecl>,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservableDecl {
    pub name: String,
    pub latex: String,
    pub parsed: MathExpr,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyDecl {
    pub name: String,
    pub statement: Option<MathExpr>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParsedModel {
    pub name: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub declarations: Vec<VariableDecl>,
    #[serde(default)]
    pub equations: Vec<EquationDecl>,
    #[serde(default)]
    pub boundaries: Vec<BoundaryDecl>,
    #[serde(default)]
    pub observables: Vec<ObservableDecl>,
    #[serde(default)]
    pub properties: Vec<PropertyDecl>,
    #[serde(default)]
    pub assumptions: Vec<(String, String)>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub source_digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElaboratedModel {
    pub context: Context,
    pub spec: ScientificSpec,
    pub source_digest: String,
    #[serde(default)]
    pub symbol_spans: BTreeMap<String, SourceSpan>,
}

#[derive(Debug, Error)]
pub enum AuthorError {
    #[error("authoring failed with {0} diagnostic(s)")]
    Diagnostics(usize, Vec<Diagnostic>),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl AuthorError {
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::Diagnostics(_, d) => d,
            Self::Serialization(_) => &[],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Ident(String),
    Text(String),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Colon,
    Semi,
    Comma,
    Equal,
    Slash,
    Star,
    Caret,
    Minus,
    Plus,
}
#[derive(Clone, Debug, PartialEq)]
struct Token {
    kind: Kind,
    span: SourceSpan,
}

pub fn parse_model(source: &str) -> Result<ParsedModel, AuthorError> {
    let tokens = match lex(source) {
        Ok(t) => t,
        Err(d) => return Err(AuthorError::Diagnostics(1, vec![*d])),
    };
    let mut parser = Parser {
        source,
        tokens: &tokens,
        pos: 0,
        diagnostics: Vec::new(),
    };
    let model = parser.model();
    match model {
        Some(mut model) if parser.diagnostics.is_empty() => {
            model.source_digest = blake3::hash(source.as_bytes()).to_hex().to_string();
            Ok(model)
        }
        _ => Err(AuthorError::Diagnostics(
            parser.diagnostics.len(),
            parser.diagnostics,
        )),
    }
}

pub fn elaborate(source: &str) -> Result<ElaboratedModel, AuthorError> {
    let parsed = parse_model(source)?;
    elaborate_parsed(&parsed)
        .map_err(|diagnostics| AuthorError::Diagnostics(diagnostics.len(), diagnostics))
}

pub fn elaborate_parsed(parsed: &ParsedModel) -> Result<ElaboratedModel, Vec<Diagnostic>> {
    let mut context = Context::new();
    let mut symbols = BTreeMap::<String, SymbolId>::new();
    let mut symbol_spans = BTreeMap::new();
    let mut unknowns = Vec::new();
    let mut parameters = Vec::new();
    for decl in &parsed.declarations {
        if matches!(decl.kind, DeclKind::Coefficient | DeclKind::Source) {
            continue;
        }
        let role = match decl.kind {
            DeclKind::Field => SymbolRole::State,
            DeclKind::Parameter => SymbolRole::Parameter,
            _ => SymbolRole::Auxiliary,
        };
        let id = context.declare_symbol(Symbol {
            name: decl.name.clone(),
            role,
            dimension: decl.unit.as_ref().map(|u| u.dimension.to_string()),
        });
        symbols.insert(decl.name.clone(), id);
        symbol_spans.insert(decl.name.clone(), decl.span);
        if decl.kind == DeclKind::Field {
            unknowns.push(id);
        } else {
            parameters.push(id);
        }
    }
    for coordinate in ["t", "x", "y", "z"] {
        if !symbols.contains_key(coordinate) {
            let dimension = if coordinate == "t" {
                Dimension::TIME
            } else {
                Dimension::LENGTH
            };
            let id = context.declare_symbol(Symbol {
                name: coordinate.into(),
                role: SymbolRole::Independent,
                dimension: Some(dimension.to_string()),
            });
            symbols.insert(coordinate.into(), id);
        }
    }
    let mut diagnostics = Vec::new();
    let mut equations = Vec::new();
    for equation in &parsed.equations {
        match (
            lower_math(&mut context, &symbols, &equation.parsed.lhs),
            lower_math(&mut context, &symbols, &equation.parsed.rhs),
        ) {
            (Ok(lhs), Ok(rhs)) => equations.push(Equation {
                lhs,
                rhs,
                label: None,
            }),
            (Err(message), _) | (_, Err(message)) => diagnostics.push(
                Diagnostic::error("RSL-E201", "elaborate", message)
                    .at(equation.span, "while elaborating this equation"),
            ),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let system_id = context.insert_system(System {
        name: parsed.name.clone(),
        unknowns,
        parameters,
        equations,
        events: Vec::new(),
        children: Vec::new(),
        metadata: BTreeMap::new(),
    });
    let mut observables = Vec::new();
    for (index, observable) in parsed.observables.iter().enumerate() {
        let expression =
            lower_math(&mut context, &symbols, &observable.parsed).map_err(|message| {
                vec![
                    Diagnostic::error("RSL-O201", "elaborate", message)
                        .at(observable.span, "while elaborating this observable"),
                ]
            })?;
        observables.push(Observable {
            id: ObservableId(index as u32),
            name: observable.name.clone(),
            expression,
            dimension: None,
            measurement_model: None,
        });
    }
    let mut properties = Vec::new();
    for property in &parsed.properties {
        let statement = if let Some(expr) = &property.statement {
            lower_math(&mut context, &symbols, expr).map_err(|message| {
                vec![
                    Diagnostic::error("RSL-P201", "elaborate", message)
                        .at(property.span, "while elaborating this property"),
                ]
            })?
        } else {
            context.exprs.intern(ExprNode::Apply {
                function: format!("property::{}", property.name),
                args: Vec::new(),
            })
        };
        properties.push(PropertyContract {
            name: property.name.clone(),
            kind: PropertyKind::Custom,
            statement,
            formal_declaration: None,
            notes: Vec::new(),
        });
    }
    let assumptions = parsed
        .assumptions
        .iter()
        .map(|(name, statement)| Assumption {
            name: name.clone(),
            statement: statement.clone(),
            formal_declaration: None,
        })
        .collect();
    let spec = ScientificSpec {
        name: parsed.name.clone(),
        model: system_id,
        assumptions,
        scope: Scope::default(),
        observables,
        properties,
        sources: vec![format!("rsl:{}", parsed.source_digest)],
        metadata: BTreeMap::new(),
    };
    Ok(ElaboratedModel {
        context,
        spec,
        source_digest: parsed.source_digest.clone(),
        symbol_spans,
    })
}

fn lower_math(
    context: &mut Context,
    symbols: &BTreeMap<String, SymbolId>,
    expr: &MathExpr,
) -> Result<ExprId, String> {
    Ok(match expr {
        MathExpr::Number(text) => context.exprs.literal(decimal_literal(text)?),
        MathExpr::Name(name) => {
            if let Some(symbol) = symbols.get(name) {
                context.exprs.symbol(*symbol)
            } else {
                context.exprs.intern(ExprNode::Apply {
                    function: name.clone(),
                    args: Vec::new(),
                })
            }
        }
        MathExpr::Neg(x) => {
            let x = lower_math(context, symbols, x)?;
            context.exprs.intern(ExprNode::Neg(x))
        }
        MathExpr::Add(xs) => {
            let xs = xs
                .iter()
                .map(|x| lower_math(context, symbols, x))
                .collect::<Result<Vec<_>, _>>()?;
            context.exprs.add(xs)
        }
        MathExpr::Mul(xs) => {
            let xs = xs
                .iter()
                .map(|x| lower_math(context, symbols, x))
                .collect::<Result<Vec<_>, _>>()?;
            context.exprs.mul(xs)
        }
        MathExpr::Div {
            numerator,
            denominator,
        } => {
            let n = lower_math(context, symbols, numerator)?;
            let d = lower_math(context, symbols, denominator)?;
            context.exprs.intern(ExprNode::Div {
                numerator: n,
                denominator: d,
            })
        }
        MathExpr::Pow { base, exponent } => {
            let base = lower_math(context, symbols, base)?;
            context.exprs.intern(ExprNode::PowI {
                base,
                exponent: *exponent,
            })
        }
        MathExpr::Call { function, args } => {
            let args = args
                .iter()
                .map(|x| lower_math(context, symbols, x))
                .collect::<Result<Vec<_>, _>>()?;
            context.exprs.intern(ExprNode::Apply {
                function: function.clone(),
                args,
            })
        }
        MathExpr::Derivative {
            expr,
            with_respect_to,
            order,
        } => {
            let expr = lower_math(context, symbols, expr)?;
            let wrt = symbols
                .get(with_respect_to)
                .copied()
                .ok_or_else(|| format!("unknown derivative coordinate `{with_respect_to}`"))?;
            context.exprs.intern(ExprNode::Derivative {
                expr,
                with_respect_to: wrt,
                order: *order,
            })
        }
        MathExpr::Gradient(x) => {
            let x = lower_math(context, symbols, x)?;
            context.exprs.intern(ExprNode::Apply {
                function: "grad".into(),
                args: vec![x],
            })
        }
        MathExpr::Divergence(x) => {
            let x = lower_math(context, symbols, x)?;
            context.exprs.intern(ExprNode::Apply {
                function: "div".into(),
                args: vec![x],
            })
        }
        MathExpr::Curl(x) => {
            let x = lower_math(context, symbols, x)?;
            context.exprs.intern(ExprNode::Apply {
                function: "curl".into(),
                args: vec![x],
            })
        }
        MathExpr::Inner { left, right } => {
            let left = lower_math(context, symbols, left)?;
            let right = lower_math(context, symbols, right)?;
            context.exprs.intern(ExprNode::Apply {
                function: "inner".into(),
                args: vec![left, right],
            })
        }
    })
}

fn decimal_literal(text: &str) -> Result<ScalarLiteral, String> {
    if !text.contains(['.', 'e', 'E']) {
        return text
            .parse::<i64>()
            .map(ScalarLiteral::integer)
            .map_err(|_| {
                format!("integer literal `{text}` is outside the initial i64 authoring range")
            });
    }
    if text.contains(['e', 'E']) {
        return text
            .parse::<f64>()
            .ok()
            .and_then(ScalarLiteral::f64_exact)
            .ok_or_else(|| format!("invalid finite numeric literal `{text}`"));
    }
    let negative = text.starts_with('-');
    let unsigned = text.trim_start_matches(['-', '+']);
    let Some((whole, fractional)) = unsigned.split_once('.') else {
        return Err(format!("invalid decimal `{text}`"));
    };
    let digits = format!(
        "{}{}",
        if whole.is_empty() { "0" } else { whole },
        fractional
    );
    let mut numerator = digits
        .parse::<i128>()
        .map_err(|_| format!("decimal literal `{text}` is too large for the bootstrap parser"))?;
    if negative {
        numerator = -numerator;
    }
    let denominator = 10_i128.pow(fractional.len() as u32);
    Ok(ScalarLiteral::Rational {
        numerator: numerator.to_string(),
        denominator: denominator.to_string(),
    })
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}
impl Parser<'_> {
    fn model(&mut self) -> Option<ParsedModel> {
        if !self.keyword("model") {
            self.error_here("RSL-P001", "expected `model`");
            return None;
        }
        let name = self.ident()?;
        if !self.eat(|k| matches!(k, Kind::LBrace)) {
            self.error_here("RSL-P002", "expected `{` after model name");
            return None;
        }
        let mut model = ParsedModel {
            name,
            domains: Vec::new(),
            declarations: Vec::new(),
            equations: Vec::new(),
            boundaries: Vec::new(),
            observables: Vec::new(),
            properties: Vec::new(),
            assumptions: Vec::new(),
            diagnostics: Vec::new(),
            source_digest: String::new(),
        };
        while self.pos < self.tokens.len() && !matches!(self.tokens[self.pos].kind, Kind::RBrace) {
            if self.keyword("domain") {
                if let Some(name) = self.ident() {
                    model.domains.push(name);
                }
                self.expect_semi();
            } else if self.keyword("field") {
                if let Some(decl) = self.field_decl() {
                    model.declarations.push(decl);
                }
            } else if self.keyword("parameter") {
                if let Some(decl) = self.simple_decl(DeclKind::Parameter) {
                    model.declarations.push(decl);
                }
            } else if self.keyword("coefficient") {
                if let Some(decl) = self.callable_decl(DeclKind::Coefficient) {
                    model.declarations.push(decl);
                }
            } else if self.keyword("source") {
                if let Some(decl) = self.callable_decl(DeclKind::Source) {
                    model.declarations.push(decl);
                }
            } else if self.keyword("equation") {
                if let Some(eq) = self.equation_decl() {
                    model.equations.push(eq);
                }
            } else if self.keyword("boundary") {
                if let Some(boundary) = self.boundary_decl() {
                    model.boundaries.push(boundary);
                }
            } else if self.keyword("observable") {
                if let Some(obs) = self.observable_decl() {
                    model.observables.push(obs);
                }
            } else if self.keyword("property") {
                if let Some(prop) = self.property_decl() {
                    model.properties.push(prop);
                }
            } else if self.keyword("assumption") {
                if let Some(a) = self.assumption_decl() {
                    model.assumptions.push(a);
                }
            } else {
                self.error_here("RSL-P003", "unknown model statement");
                self.pos += 1;
            }
        }
        self.eat(|k| matches!(k, Kind::RBrace));
        Some(model)
    }

    fn field_decl(&mut self) -> Option<VariableDecl> {
        let start = self.current_span();
        let name = self.ident()?;
        if self.eat(|k| matches!(k, Kind::Colon)) {
            let role = self.ident();
            let shape = self.ident();
            let space = self.ident();
            let unit = self.unit_opt();
            self.expect_semi();
            return Some(VariableDecl {
                kind: DeclKind::Field,
                name,
                arguments: Vec::new(),
                role,
                shape,
                space,
                unit,
                span: start,
            });
        }
        if self.eat(|k| matches!(k, Kind::LBrace)) {
            let mut role = None;
            let mut shape = None;
            let mut space = None;
            let mut unit = None;
            while self.pos < self.tokens.len()
                && !matches!(self.tokens[self.pos].kind, Kind::RBrace)
            {
                let key = self.ident()?;
                self.eat(|k| matches!(k, Kind::Colon));
                if key == "units" {
                    unit = self.unit_until_semi();
                } else {
                    let value = self.ident();
                    match key.as_str() {
                        "role" => role = value,
                        "shape" => shape = value,
                        "space" => space = value,
                        _ => self.error_here("RSL-F004", "unknown field attribute"),
                    }
                }
                self.expect_semi();
            }
            self.eat(|k| matches!(k, Kind::RBrace));
            return Some(VariableDecl {
                kind: DeclKind::Field,
                name,
                arguments: Vec::new(),
                role,
                shape,
                space,
                unit,
                span: start,
            });
        }
        self.error_here("RSL-F001", "expected `:` or `{` after field name");
        None
    }

    fn simple_decl(&mut self, kind: DeclKind) -> Option<VariableDecl> {
        let span = self.current_span();
        let name = self.ident()?;
        let unit = self.unit_opt();
        self.expect_semi();
        Some(VariableDecl {
            kind,
            name,
            arguments: Vec::new(),
            role: None,
            shape: None,
            space: None,
            unit,
            span,
        })
    }

    fn callable_decl(&mut self, kind: DeclKind) -> Option<VariableDecl> {
        let span = self.current_span();
        let name = self.ident()?;
        let mut arguments = Vec::new();
        if self.eat(|k| matches!(k, Kind::LParen)) {
            while self.pos < self.tokens.len()
                && !matches!(self.tokens[self.pos].kind, Kind::RParen)
            {
                if let Some(arg) = self.ident() {
                    arguments.push(arg);
                }
                if !self.eat(|k| matches!(k, Kind::Comma)) {
                    break;
                }
            }
            self.eat(|k| matches!(k, Kind::RParen));
        }
        let unit = self.unit_opt();
        self.expect_semi();
        Some(VariableDecl {
            kind,
            name,
            arguments,
            role: None,
            shape: None,
            space: None,
            unit,
            span,
        })
    }

    fn equation_decl(&mut self) -> Option<EquationDecl> {
        let span = self.current_span();
        let latex = self.latex_value()?;
        self.expect_semi();
        match latex::parse_equation(&latex) {
            Ok(parsed) => Some(EquationDecl {
                latex,
                parsed,
                span,
            }),
            Err(error) => {
                self.diagnostics.push(
                    Diagnostic::error("RSL-L101", "latex", error.to_string())
                        .at(span, "invalid equation"),
                );
                None
            }
        }
    }

    fn boundary_decl(&mut self) -> Option<BoundaryDecl> {
        let span = self.current_span();
        let name = self.ident()?;
        if !self.eat(|k| matches!(k, Kind::LBrace)) {
            self.error_here("RSL-B001", "expected boundary body");
            return None;
        }
        let mut equations = Vec::new();
        while self.pos < self.tokens.len() && !matches!(self.tokens[self.pos].kind, Kind::RBrace) {
            if self.keyword("latex") {
                if let Some(text) = self.text_value() {
                    match latex::parse_equation(&text) {
                        Ok(parsed) => equations.push(EquationDecl {
                            latex: text,
                            parsed,
                            span,
                        }),
                        Err(error) => self.diagnostics.push(
                            Diagnostic::error("RSL-L102", "latex", error.to_string())
                                .at(span, "invalid boundary equation"),
                        ),
                    }
                }
                self.expect_semi();
            } else {
                self.error_here(
                    "RSL-B002",
                    "only `latex` is currently legal inside a boundary",
                );
                self.pos += 1;
            }
        }
        self.eat(|k| matches!(k, Kind::RBrace));
        Some(BoundaryDecl {
            name,
            equations,
            span,
        })
    }

    fn observable_decl(&mut self) -> Option<ObservableDecl> {
        let span = self.current_span();
        let name = self.ident()?;
        if self.eat(|k| matches!(k, Kind::Equal)) {
            let latex = self.latex_value()?;
            self.expect_semi();
            return self.observable_from_latex(name, latex, span);
        }
        if self.eat(|k| matches!(k, Kind::LBrace)) {
            if !self.keyword("latex") {
                self.error_here("RSL-O001", "expected latex observable body");
                return None;
            }
            let latex = self.text_value()?;
            self.expect_semi();
            self.eat(|k| matches!(k, Kind::RBrace));
            return self.observable_from_latex(name, latex, span);
        }
        self.error_here("RSL-O002", "expected `=` or observable body");
        None
    }

    fn observable_from_latex(
        &mut self,
        name: String,
        latex: String,
        span: SourceSpan,
    ) -> Option<ObservableDecl> {
        match latex::parse_expr(&latex) {
            Ok(parsed) => Some(ObservableDecl {
                name,
                latex,
                parsed,
                span,
            }),
            Err(error) => {
                self.diagnostics.push(
                    Diagnostic::error("RSL-L103", "latex", error.to_string())
                        .at(span, "invalid observable"),
                );
                None
            }
        }
    }

    fn property_decl(&mut self) -> Option<PropertyDecl> {
        let span = self.current_span();
        let name = self.ident()?;
        self.expect_semi();
        Some(PropertyDecl {
            name,
            statement: None,
            span,
        })
    }
    fn assumption_decl(&mut self) -> Option<(String, String)> {
        let name = self.ident()?;
        self.eat(|k| matches!(k, Kind::Equal));
        let text = self.text_value()?;
        self.expect_semi();
        Some((name, text))
    }

    fn latex_value(&mut self) -> Option<String> {
        if !self.keyword("latex") {
            self.error_here("RSL-L001", "expected `latex`");
            return None;
        }
        if self.eat(|k| matches!(k, Kind::LParen)) {
            let text = self.text_value();
            self.eat(|k| matches!(k, Kind::RParen));
            text
        } else {
            self.text_value()
        }
    }
    fn text_value(&mut self) -> Option<String> {
        match self.tokens.get(self.pos) {
            Some(Token {
                kind: Kind::Text(value),
                ..
            }) => {
                self.pos += 1;
                Some(value.clone())
            }
            _ => {
                self.error_here("RSL-P010", "expected string or triple-quoted text");
                None
            }
        }
    }
    fn unit_opt(&mut self) -> Option<UnitExpr> {
        if !self.eat(|k| matches!(k, Kind::LBracket)) {
            return None;
        }
        let start = self.pos;
        while self.pos < self.tokens.len() && !matches!(self.tokens[self.pos].kind, Kind::RBracket)
        {
            self.pos += 1;
        }
        let text = self.tokens[start..self.pos]
            .iter()
            .map(render_token)
            .collect::<String>();
        self.eat(|k| matches!(k, Kind::RBracket));
        match parse_unit(&text) {
            Ok(unit) => Some(unit),
            Err(error) => {
                self.error_here("RSL-U001", &error.to_string());
                None
            }
        }
    }
    fn unit_until_semi(&mut self) -> Option<UnitExpr> {
        let start = self.pos;
        while self.pos < self.tokens.len() && !matches!(self.tokens[self.pos].kind, Kind::Semi) {
            self.pos += 1;
        }
        let text = self.tokens[start..self.pos]
            .iter()
            .map(render_token)
            .collect::<String>();
        match parse_unit(&text) {
            Ok(unit) => Some(unit),
            Err(error) => {
                self.error_here("RSL-U002", &error.to_string());
                None
            }
        }
    }
    fn keyword(&mut self, expected: &str) -> bool {
        if matches!(self.tokens.get(self.pos), Some(Token { kind: Kind::Ident(s), .. }) if s == expected)
        {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn ident(&mut self) -> Option<String> {
        match self.tokens.get(self.pos) {
            Some(Token {
                kind: Kind::Ident(value),
                ..
            }) => {
                self.pos += 1;
                Some(value.clone())
            }
            _ => {
                self.error_here("RSL-P011", "expected identifier");
                None
            }
        }
    }
    fn eat(&mut self, pred: impl FnOnce(&Kind) -> bool) -> bool {
        if self.tokens.get(self.pos).is_some_and(|t| pred(&t.kind)) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect_semi(&mut self) {
        if !self.eat(|k| matches!(k, Kind::Semi)) {
            self.error_here("RSL-P012", "expected `;`");
        }
    }
    fn current_span(&self) -> SourceSpan {
        self.tokens.get(self.pos).map_or_else(
            || SourceSpan::new(self.source, self.source.len(), self.source.len()),
            |t| t.span,
        )
    }
    fn error_here(&mut self, code: &str, message: &str) {
        let span = self.current_span();
        self.diagnostics
            .push(Diagnostic::error(code, "parse", message).at(span, "here"));
    }
}

fn render_token(token: &Token) -> String {
    match &token.kind {
        Kind::Ident(s) | Kind::Text(s) => s.clone(),
        Kind::LBrace => "{".into(),
        Kind::RBrace => "}".into(),
        Kind::LBracket => "[".into(),
        Kind::RBracket => "]".into(),
        Kind::LParen => "(".into(),
        Kind::RParen => ")".into(),
        Kind::Colon => ":".into(),
        Kind::Semi => ";".into(),
        Kind::Comma => ",".into(),
        Kind::Equal => "=".into(),
        Kind::Slash => "/".into(),
        Kind::Star => "*".into(),
        Kind::Caret => "^".into(),
        Kind::Minus => "-".into(),
        Kind::Plus => "+".into(),
    }
}

fn lex(source: &str) -> Result<Vec<Token>, Box<Diagnostic>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < source.len() {
        let ch = source[i..].chars().next().unwrap();
        let width = ch.len_utf8();
        if ch.is_whitespace() {
            i += width;
            continue;
        }
        if source[i..].starts_with("//") {
            i = source[i..].find('\n').map_or(source.len(), |n| i + n + 1);
            continue;
        }
        let start = i;
        if source[i..].starts_with("\"\"\"") {
            i += 3;
            let body = i;
            let Some(end_rel) = source[i..].find("\"\"\"") else {
                return Err(Box::new(
                    Diagnostic::error("RSL-L000", "lex", "unclosed triple-quoted string")
                        .at(SourceSpan::new(source, start, source.len()), "opened here"),
                ));
            };
            let end = i + end_rel;
            let value = source[body..end].to_string();
            i = end + 3;
            out.push(Token {
                kind: Kind::Text(value),
                span: SourceSpan::new(source, start, i),
            });
            continue;
        }
        if ch == '"' {
            i += 1;
            let body = i;
            while i < source.len() && !source[i..].starts_with('"') {
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            if i >= source.len() {
                return Err(Box::new(
                    Diagnostic::error("RSL-S000", "lex", "unclosed string")
                        .at(SourceSpan::new(source, start, source.len()), "opened here"),
                ));
            }
            let value = source[body..i].to_string();
            i += 1;
            out.push(Token {
                kind: Kind::Text(value),
                span: SourceSpan::new(source, start, i),
            });
            continue;
        }
        let kind = match ch {
            '{' => {
                i += 1;
                Kind::LBrace
            }
            '}' => {
                i += 1;
                Kind::RBrace
            }
            '[' => {
                i += 1;
                Kind::LBracket
            }
            ']' => {
                i += 1;
                Kind::RBracket
            }
            '(' => {
                i += 1;
                Kind::LParen
            }
            ')' => {
                i += 1;
                Kind::RParen
            }
            ':' => {
                i += 1;
                Kind::Colon
            }
            ';' => {
                i += 1;
                Kind::Semi
            }
            ',' => {
                i += 1;
                Kind::Comma
            }
            '=' => {
                i += 1;
                Kind::Equal
            }
            '/' => {
                i += 1;
                Kind::Slash
            }
            '*' => {
                i += 1;
                Kind::Star
            }
            '^' => {
                i += 1;
                Kind::Caret
            }
            '-' => {
                i += 1;
                Kind::Minus
            }
            '+' => {
                i += 1;
                Kind::Plus
            }
            c if c.is_alphanumeric() || c == '_' || !c.is_ascii() => {
                i += width;
                while i < source.len() {
                    let c = source[i..].chars().next().unwrap();
                    if !(c.is_alphanumeric() || c == '_' || !c.is_ascii()) {
                        break;
                    }
                    i += c.len_utf8();
                }
                Kind::Ident(source[start..i].to_string())
            }
            _ => {
                return Err(Box::new(
                    Diagnostic::error("RSL-X001", "lex", format!("unexpected character `{ch}`"))
                        .at(
                            SourceSpan::new(source, start, start + width),
                            "not valid RSL syntax",
                        ),
                ));
            }
        };
        out.push(Token {
            kind,
            span: SourceSpan::new(source, start, i),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAT: &str = r#"
model nonlinear_heat {
    domain Omega;
    field T: state scalar H1 [K];
    parameter rho [kg/m^3];
    coefficient cp(T) [J/(kg K)];
    coefficient k(T) [W/(m K)];
    source Q(T) [W/m^3];
    equation latex """ rho cp \frac{\partial T}{\partial t} = Q """;
    observable temperature = latex("T");
    property energy_balance;
}
"#;

    #[test]
    fn parses_and_elaborates_agent_model() {
        let parsed = parse_model(HEAT).unwrap();
        assert_eq!(parsed.name, "nonlinear_heat");
        assert_eq!(parsed.declarations.len(), 5);
        let elaborated = elaborate(HEAT).unwrap();
        assert_eq!(elaborated.spec.name, "nonlinear_heat");
        assert_eq!(
            elaborated
                .context
                .system(elaborated.spec.model)
                .unwrap()
                .equations
                .len(),
            1
        );
    }
}
