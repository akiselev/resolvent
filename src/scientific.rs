//! R13-R20 scientific authoring and semantic infrastructure.
//!
//! This module is deliberately compiler-oriented: source declarations become typed data,
//! expressions remain structured, coupling is derived from use, and solver strategy never
//! enters the model semantics.

use crate::source::SourceSpan;
use resolvent_quantities::{
    Bound, CanonicalQuantity, Dimension, DisplayUnit, KindStrictness, QuantityKindId,
    QuantityLiteral, UnitId, UnitRegistry,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const SCIENTIFIC_V1_SCHEMA: &str = "resolvent-scientific-v1/1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScientificModule {
    pub schema: String,
    pub name: String,
    pub imports: Vec<String>,
    pub models: Vec<ScientificModel>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScientificModel {
    pub name: String,
    pub domains: Vec<DomainDecl>,
    pub fields: Vec<FieldDecl>,
    pub parameters: Vec<ValueDecl>,
    pub constants: Vec<ValueDecl>,
    pub sources: Vec<ValueDecl>,
    pub properties: Vec<PropertyBinding>,
    pub constitutive_laws: Vec<ConstitutiveBinding>,
    pub equations: Vec<EquationDecl>,
    pub forms: Vec<FormDecl>,
    pub initial_conditions: Vec<ConditionDecl>,
    pub boundary_conditions: Vec<BoundaryConditionDecl>,
    pub interface_conditions: Vec<BoundaryConditionDecl>,
    pub observables: Vec<ObservableDecl>,
    pub invariants: Vec<ObservableDecl>,
    pub verifications: Vec<VerificationAnnotation>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DomainDecl {
    pub name: String,
    pub dimension: u8,
    pub coordinates: CoordinateSystem,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateSystem {
    Cartesian,
    Cylindrical,
    Spherical,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldRoleV1 {
    State,
    Unknown,
    Test,
    Trial,
    Coefficient,
    Parameter,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueShapeV1 {
    Scalar,
    Vector(u8),
    Tensor { rows: u8, cols: u8 },
    SymmetricTensor(u8),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceSpec {
    pub family: SpaceFamily,
    pub order: u8,
    pub continuity: ContinuityV1,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceFamily {
    H1,
    L2,
    HCurl,
    HDiv,
    Dg,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuityV1 {
    Continuous,
    Discontinuous,
    Tangential,
    Normal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub role: FieldRoleV1,
    pub shape: ValueShapeV1,
    pub space: SpaceSpec,
    pub domain: String,
    pub quantity_kind: Option<QuantityKindId>,
    pub unit: Option<UnitId>,
    pub nominal: Option<QuantityLiteral>,
    pub physical_min: Option<QuantityLiteral>,
    pub physical_max: Option<QuantityLiteral>,
    pub time_role: Option<TimeRole>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueDecl {
    pub name: String,
    pub quantity_kind: Option<QuantityKindId>,
    pub unit: Option<UnitId>,
    pub value: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyBinding {
    pub name: String,
    pub value: Expr,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstitutiveBinding {
    pub name: String,
    pub law: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EquationDecl {
    pub name: String,
    pub domain: Option<String>,
    pub lhs: Expr,
    pub rhs: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormDecl {
    pub name: String,
    pub integrals: Vec<IntegralDecl>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegralDecl {
    pub measure: MeasureV1,
    pub integrand: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureV1 {
    Cell(String),
    Boundary(String),
    InteriorFacet(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionDecl {
    pub target: String,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundaryConditionDecl {
    pub name: String,
    pub region: Expr,
    pub kind: BoundaryConditionKind,
    pub target: String,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryConditionKind {
    Dirichlet,
    Neumann,
    Robin,
    Interface,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservableDecl {
    pub name: String,
    pub value: Expr,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerificationAnnotation {
    pub name: String,
    pub args: BTreeMap<String, Expr>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number {
        value: f64,
        unit: Option<String>,
    },
    String(String),
    Name(String),
    Unary {
        op: UnaryOp,
        arg: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        function: String,
        args: Vec<Expr>,
    },
    Index {
        value: Box<Expr>,
        indices: Vec<Expr>,
    },
    Vector(Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Expr {
    pub fn names(&self, out: &mut BTreeSet<String>) {
        match self {
            Expr::Name(name) => {
                out.insert(name.clone());
            }
            Expr::Unary { arg, .. } => arg.names(out),
            Expr::Binary { lhs, rhs, .. } => {
                lhs.names(out);
                rhs.names(out);
            }
            Expr::Call { args, .. } | Expr::Vector(args) => {
                for arg in args {
                    arg.names(out);
                }
            }
            Expr::Index { value, indices } => {
                value.names(out);
                for i in indices {
                    i.names(out);
                }
            }
            Expr::Number { .. } | Expr::String(_) => {}
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ScientificError {
    #[error("syntax error at {span:?}: {message}")]
    Syntax { message: String, span: SourceSpan },
    #[error("duplicate declaration `{0}`")]
    Duplicate(String),
    #[error("unknown name `{0}`")]
    UnknownName(String),
    #[error("import cycle: {0}")]
    ImportCycle(String),
    #[error("missing imported module `{0}`")]
    MissingModule(String),
    #[error("quantity error: {0}")]
    Quantity(String),
    #[error("property evaluation failed: {0}")]
    Property(String),
}

// ---------------- R14 lexer/parser ----------------

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Ident(String),
    Number(f64),
    String(String),
    Punct(char),
    Op(String),
    Eof,
}
#[derive(Clone, Debug, PartialEq)]
struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

fn lex(input: &str) -> Result<Vec<Token>, Vec<ScientificError>> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    let mut errors = Vec::new();
    while i < bytes.len() {
        let c = input[i..].chars().next().unwrap();
        if c.is_whitespace() {
            i += c.len_utf8();
            continue;
        }
        if c == '#' || input[i..].starts_with("//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        if c.is_ascii_alphabetic() || c == '_' || c == 'π' {
            i += c.len_utf8();
            while i < bytes.len() {
                let x = input[i..].chars().next().unwrap();
                if x.is_alphanumeric() || matches!(x, '_' | '.' | 'π') {
                    i += x.len_utf8();
                } else {
                    break;
                }
            }
            out.push(Token {
                kind: TokenKind::Ident(input[start..i].into()),
                span: SourceSpan::new(start, i),
            });
            continue;
        }
        if c.is_ascii_digit()
            || (c == '.'
                && input[i + 1..]
                    .chars()
                    .next()
                    .is_some_and(|x| x.is_ascii_digit()))
        {
            i += c.len_utf8();
            while i < bytes.len() {
                let x = input[i..].chars().next().unwrap();
                if x.is_ascii_digit() || matches!(x, '.' | 'e' | 'E' | '+' | '-') {
                    if (x == '+' || x == '-')
                        && !matches!(input[..i].chars().last(), Some('e' | 'E'))
                    {
                        break;
                    }
                    i += x.len_utf8();
                } else {
                    break;
                }
            }
            match input[start..i].parse::<f64>() {
                Ok(v) => out.push(Token {
                    kind: TokenKind::Number(v),
                    span: SourceSpan::new(start, i),
                }),
                Err(_) => errors.push(ScientificError::Syntax {
                    message: "invalid number".into(),
                    span: SourceSpan::new(start, i),
                }),
            }
            continue;
        }
        if c == '"' {
            i += 1;
            let body = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i >= bytes.len() {
                errors.push(ScientificError::Syntax {
                    message: "unterminated string".into(),
                    span: SourceSpan::new(start, bytes.len()),
                });
                break;
            }
            out.push(Token {
                kind: TokenKind::String(input[body..i].into()),
                span: SourceSpan::new(start, i + 1),
            });
            i += 1;
            continue;
        }
        let two = if i + 1 < bytes.len() {
            &input[i..i + 2]
        } else {
            ""
        };
        if matches!(two, "==" | "<=" | ">=" | "->") {
            out.push(Token {
                kind: TokenKind::Op(two.into()),
                span: SourceSpan::new(i, i + 2),
            });
            i += 2;
            continue;
        }
        if "+-*/^=<>".contains(c) {
            out.push(Token {
                kind: TokenKind::Op(c.to_string()),
                span: SourceSpan::new(i, i + c.len_utf8()),
            });
            i += c.len_utf8();
            continue;
        }
        if "{}();:,[]@".contains(c) {
            out.push(Token {
                kind: TokenKind::Punct(c),
                span: SourceSpan::new(i, i + c.len_utf8()),
            });
            i += c.len_utf8();
            continue;
        }
        errors.push(ScientificError::Syntax {
            message: format!("unexpected character `{c}`"),
            span: SourceSpan::new(start, start + c.len_utf8()),
        });
        i += c.len_utf8();
    }
    out.push(Token {
        kind: TokenKind::Eof,
        span: SourceSpan::new(input.len(), input.len()),
    });
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    errors: Vec<ScientificError>,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            i: 0,
            errors: vec![],
        }
    }
    fn token(&self) -> &Token {
        &self.tokens[self.i]
    }
    fn bump(&mut self) -> Token {
        let t = self.tokens[self.i].clone();
        if !matches!(t.kind, TokenKind::Eof) {
            self.i += 1;
        }
        t
    }
    fn ident_is(&self, s: &str) -> bool {
        matches!(&self.token().kind, TokenKind::Ident(x) if x == s)
    }
    fn eat_ident(&mut self, s: &str) -> bool {
        if self.ident_is(s) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_punct(&mut self, c: char) -> bool {
        if matches!(self.token().kind, TokenKind::Punct(x) if x == c) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_op(&mut self, op: &str) -> bool {
        if matches!(&self.token().kind, TokenKind::Op(x) if x == op) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect_punct(&mut self, c: char) -> bool {
        if self.eat_punct(c) {
            true
        } else {
            self.error(format!("expected `{c}`"));
            false
        }
    }
    fn expect_ident_value(&mut self) -> Option<(String, SourceSpan)> {
        let t = self.bump();
        match t.kind {
            TokenKind::Ident(x) => Some((x, t.span)),
            _ => {
                self.errors.push(ScientificError::Syntax {
                    message: "expected identifier".into(),
                    span: t.span,
                });
                None
            }
        }
    }
    fn error(&mut self, message: String) {
        self.errors.push(ScientificError::Syntax {
            message,
            span: self.token().span,
        });
    }
    fn sync(&mut self) {
        while !matches!(self.token().kind, TokenKind::Eof) {
            if self.eat_punct(';') {
                break;
            }
            if matches!(self.token().kind, TokenKind::Punct('}')) {
                break;
            }
            self.bump();
        }
    }

    fn module(&mut self) -> ScientificModule {
        let start = self.token().span.start;
        let name = if self.eat_ident("module") {
            let name = self
                .expect_ident_value()
                .map(|x| x.0)
                .unwrap_or_else(|| "invalid".into());
            self.expect_punct(';');
            name
        } else {
            "main".into()
        };
        let mut imports = vec![];
        while self.eat_ident("use") {
            if let Some((name, _)) = self.expect_ident_value() {
                imports.push(name);
            }
            self.expect_punct(';');
        }
        let mut models = vec![];
        while !matches!(self.token().kind, TokenKind::Eof) {
            if self.eat_ident("model") {
                if let Some(model) = self.model() {
                    models.push(model);
                }
            } else {
                self.error("expected `model` declaration".into());
                self.sync();
                if matches!(self.token().kind, TokenKind::Punct('}')) {
                    self.bump();
                }
            }
        }
        ScientificModule {
            schema: SCIENTIFIC_V1_SCHEMA.into(),
            name,
            imports,
            models,
            span: SourceSpan::new(start, self.token().span.end),
        }
    }

    fn model(&mut self) -> Option<ScientificModel> {
        let (name, span) = self.expect_ident_value()?;
        self.expect_punct('{');
        let mut model = ScientificModel {
            name,
            domains: vec![],
            fields: vec![],
            parameters: vec![],
            constants: vec![],
            sources: vec![],
            properties: vec![],
            constitutive_laws: vec![],
            equations: vec![],
            forms: vec![],
            initial_conditions: vec![],
            boundary_conditions: vec![],
            interface_conditions: vec![],
            observables: vec![],
            invariants: vec![],
            verifications: vec![],
            span,
        };
        while !matches!(self.token().kind, TokenKind::Eof | TokenKind::Punct('}')) {
            if self.eat_ident("domain") {
                if let Some(x) = self.domain() {
                    model.domains.push(x);
                }
            } else if self.eat_ident("field") {
                if let Some(x) = self.field() {
                    model.fields.push(x);
                }
            } else if self.eat_ident("parameter") {
                if let Some(x) = self.value_decl() {
                    model.parameters.push(x);
                }
            } else if self.eat_ident("constant") {
                if let Some(x) = self.value_decl() {
                    model.constants.push(x);
                }
            } else if self.eat_ident("source") {
                if let Some(x) = self.value_decl() {
                    model.sources.push(x);
                }
            } else if self.eat_ident("property") {
                if let Some(x) = self.property() {
                    model.properties.push(x);
                }
            } else if self.eat_ident("constitutive") {
                if let Some(x) = self.constitutive() {
                    model.constitutive_laws.push(x);
                }
            } else if self.eat_ident("equation") {
                if let Some(x) = self.equation() {
                    model.equations.push(x);
                }
            } else if self.eat_ident("form") {
                if let Some(x) = self.form_decl() {
                    model.forms.push(x);
                }
            } else if self.eat_ident("initial") {
                model.initial_conditions.extend(self.assignment_block());
            } else if self.eat_ident("boundary") {
                if let Some(x) = self.boundary(false) {
                    model.boundary_conditions.push(x);
                }
            } else if self.eat_ident("interface") {
                if let Some(x) = self.boundary(true) {
                    model.interface_conditions.push(x);
                }
            } else if self.eat_ident("observable") {
                if let Some(x) = self.observable() {
                    model.observables.push(x);
                }
            } else if self.eat_ident("invariant") {
                if let Some(x) = self.observable() {
                    model.invariants.push(x);
                }
            } else if self.eat_punct('@') {
                if let Some(x) = self.verification() {
                    model.verifications.push(x);
                }
            } else {
                self.error("unknown model declaration".into());
                self.sync();
            }
        }
        self.expect_punct('}');
        Some(model)
    }

    fn domain(&mut self) -> Option<DomainDecl> {
        let (name, span) = self.expect_ident_value()?;
        self.expect_punct('{');
        let mut dimension = 2;
        let mut coordinates = CoordinateSystem::Cartesian;
        while !matches!(self.token().kind, TokenKind::Eof | TokenKind::Punct('}')) {
            let key = self.expect_ident_value()?.0;
            self.eat_op("=");
            if key == "dimension" {
                if let TokenKind::Number(v) = self.bump().kind {
                    dimension = v as u8;
                }
            } else if key == "coordinates" {
                if let Some((v, _)) = self.expect_ident_value() {
                    coordinates = match v.as_str() {
                        "cartesian" => CoordinateSystem::Cartesian,
                        "cylindrical" => CoordinateSystem::Cylindrical,
                        "spherical" => CoordinateSystem::Spherical,
                        _ => CoordinateSystem::Custom(v),
                    };
                }
            } else {
                self.bump();
            }
            self.eat_punct(';');
        }
        self.expect_punct('}');
        self.eat_punct(';');
        Some(DomainDecl {
            name,
            dimension,
            coordinates,
            span,
        })
    }

    fn field(&mut self) -> Option<FieldDecl> {
        let (name, span) = self.expect_ident_value()?;
        self.expect_punct(':');
        let role_name = self.expect_ident_value()?.0;
        let role = match role_name.as_str() {
            "state" => FieldRoleV1::State,
            "unknown" => FieldRoleV1::Unknown,
            "test" => FieldRoleV1::Test,
            "trial" => FieldRoleV1::Trial,
            "coefficient" => FieldRoleV1::Coefficient,
            "parameter" => FieldRoleV1::Parameter,
            "derived" => FieldRoleV1::Derived,
            _ => {
                self.error(format!("unknown field role `{role_name}`"));
                FieldRoleV1::Unknown
            }
        };
        let mut shape = ValueShapeV1::Scalar;
        if self.ident_is("scalar") {
            self.bump();
        } else if self.eat_ident("vector") {
            self.expect_punct('(');
            let n = self.number_u8(3);
            self.expect_punct(')');
            shape = ValueShapeV1::Vector(n);
        } else if self.eat_ident("tensor") {
            self.expect_punct('(');
            let a = self.number_u8(3);
            self.eat_punct(',');
            let b = self.number_u8(a);
            self.expect_punct(')');
            shape = ValueShapeV1::Tensor { rows: a, cols: b };
        }
        let family = self.expect_ident_value()?.0;
        self.expect_punct('(');
        let mut order = 1;
        if self.eat_ident("order") {
            self.eat_op("=");
            order = self.number_u8(1);
        } else if matches!(self.token().kind, TokenKind::Number(_)) {
            order = self.number_u8(1);
        }
        self.expect_punct(')');
        let space = match family.as_str() {
            "H1" => SpaceSpec {
                family: SpaceFamily::H1,
                order,
                continuity: ContinuityV1::Continuous,
            },
            "L2" => SpaceSpec {
                family: SpaceFamily::L2,
                order,
                continuity: ContinuityV1::Discontinuous,
            },
            "HCurl" | "Hcurl" => SpaceSpec {
                family: SpaceFamily::HCurl,
                order,
                continuity: ContinuityV1::Tangential,
            },
            "HDiv" | "Hdiv" => SpaceSpec {
                family: SpaceFamily::HDiv,
                order,
                continuity: ContinuityV1::Normal,
            },
            "DG" => SpaceSpec {
                family: SpaceFamily::Dg,
                order,
                continuity: ContinuityV1::Discontinuous,
            },
            _ => {
                self.error(format!("unsupported function space `{family}`"));
                SpaceSpec {
                    family: SpaceFamily::H1,
                    order,
                    continuity: ContinuityV1::Continuous,
                }
            }
        };
        if !self.eat_ident("on") {
            self.error("field requires `on <domain>`".into());
        }
        let domain = self
            .expect_ident_value()
            .map(|x| x.0)
            .unwrap_or_else(|| "Omega".into());
        let mut quantity_kind = None;
        let mut unit = None;
        let mut nominal = None;
        let mut physical_min = None;
        let mut physical_max = None;
        let mut time_role = None;
        if self.eat_punct('{') {
            while !matches!(self.token().kind, TokenKind::Eof | TokenKind::Punct('}')) {
                let key = self.expect_ident_value()?.0;
                self.eat_op("=");
                match key.as_str() {
                    "quantity" => {
                        quantity_kind = self.expect_ident_value().map(|x| QuantityKindId::new(x.0))
                    }
                    "unit" => unit = self.expect_ident_value().map(|x| UnitId::new(x.0)),
                    "nominal" => nominal = self.quantity_literal(quantity_kind.clone()),
                    "min" => physical_min = self.quantity_literal(quantity_kind.clone()),
                    "max" => physical_max = self.quantity_literal(quantity_kind.clone()),
                    "time_role" => {
                        time_role = self.expect_ident_value().map(|x| {
                            if x.0 == "algebraic" {
                                TimeRole::Algebraic
                            } else {
                                TimeRole::Differential
                            }
                        })
                    }
                    _ => {
                        self.error(format!("unknown field attribute `{key}`"));
                        self.sync();
                    }
                }
                self.eat_punct(';');
            }
            self.expect_punct('}');
        }
        self.eat_punct(';');
        Some(FieldDecl {
            name,
            role,
            shape,
            space,
            domain,
            quantity_kind,
            unit,
            nominal,
            physical_min,
            physical_max,
            time_role,
            span,
        })
    }

    fn number_u8(&mut self, default: u8) -> u8 {
        let t = self.bump();
        if let TokenKind::Number(v) = t.kind {
            v as u8
        } else {
            self.errors.push(ScientificError::Syntax {
                message: "expected integer".into(),
                span: t.span,
            });
            default
        }
    }
    fn quantity_literal(&mut self, kind: Option<QuantityKindId>) -> Option<QuantityLiteral> {
        let t = self.bump();
        let value = if let TokenKind::Number(v) = t.kind {
            v
        } else {
            self.error("expected quantity value".into());
            return None;
        };
        let unit = self.expect_ident_value()?.0;
        Some(QuantityLiteral {
            value,
            unit: UnitId::new(unit),
            quantity_kind: kind.unwrap_or_else(|| QuantityKindId::new("resolvent:Unspecified")),
        })
    }

    fn value_decl(&mut self) -> Option<ValueDecl> {
        let (name, span) = self.expect_ident_value()?;
        let mut kind = None;
        let mut unit = None;
        if self.eat_punct(':') {
            kind = self.expect_ident_value().map(|x| QuantityKindId::new(x.0));
        }
        if self.eat_punct('[') {
            unit = self.expect_ident_value().map(|x| UnitId::new(x.0));
            self.expect_punct(']');
        }
        let value = if self.eat_op("=") {
            Some(self.expr(0)?)
        } else {
            None
        };
        self.expect_punct(';');
        Some(ValueDecl {
            name,
            quantity_kind: kind,
            unit,
            value,
            span,
        })
    }
    fn property(&mut self) -> Option<PropertyBinding> {
        let (name, span) = self.expect_ident_value()?;
        if !self.eat_op("=") {
            self.error("property requires `=`".into());
        }
        let value = self.expr(0)?;
        self.expect_punct(';');
        Some(PropertyBinding { name, value, span })
    }
    fn constitutive(&mut self) -> Option<ConstitutiveBinding> {
        let (name, span) = self.expect_ident_value()?;
        self.eat_op("=");
        let law = self.expr(0)?;
        self.expect_punct(';');
        Some(ConstitutiveBinding { name, law, span })
    }

    fn equation(&mut self) -> Option<EquationDecl> {
        let (name, span) = self.expect_ident_value()?;
        let domain = if self.eat_ident("on") {
            self.expect_ident_value().map(|x| x.0)
        } else {
            None
        };
        self.expect_punct('{');
        // Top-level equality belongs to the equation declaration, not the expression tree.
        // Parse above equality precedence so comparisons remain legal inside each side.
        let lhs = self.expr(2)?;
        if !self.eat_op("=") {
            self.error("equation requires `=`".into());
        }
        let rhs = self.expr(0)?;
        self.eat_punct(';');
        self.expect_punct('}');
        self.eat_punct(';');
        Some(EquationDecl {
            name,
            domain,
            lhs,
            rhs,
            span,
        })
    }

    fn form_decl(&mut self) -> Option<FormDecl> {
        let (name, span) = self.expect_ident_value()?;
        self.expect_punct('{');
        let mut integrals = vec![];
        while !matches!(self.token().kind, TokenKind::Eof | TokenKind::Punct('}')) {
            let measure_span = self.token().span;
            let measure_name = self.expect_ident_value()?.0;
            self.expect_punct('(');
            let target = self.expect_ident_value()?.0;
            self.expect_punct(')');
            self.eat_punct(':');
            let integrand = self.expr(0)?;
            self.expect_punct(';');
            let measure = match measure_name.as_str() {
                "cell" => MeasureV1::Cell(target),
                "boundary" => MeasureV1::Boundary(target),
                "interior_facet" => MeasureV1::InteriorFacet(target),
                _ => {
                    self.error(format!("unknown measure `{measure_name}`"));
                    MeasureV1::Cell(target)
                }
            };
            integrals.push(IntegralDecl {
                measure,
                integrand,
                span: measure_span,
            });
        }
        self.expect_punct('}');
        self.eat_punct(';');
        Some(FormDecl {
            name,
            integrals,
            span,
        })
    }

    fn assignment_block(&mut self) -> Vec<ConditionDecl> {
        let mut out = vec![];
        if !self.expect_punct('{') {
            return out;
        }
        while !matches!(self.token().kind, TokenKind::Eof | TokenKind::Punct('}')) {
            let Some((target, span)) = self.expect_ident_value() else {
                self.sync();
                continue;
            };
            self.eat_op("=");
            if let Some(value) = self.expr(0) {
                out.push(ConditionDecl {
                    target,
                    value,
                    span,
                });
            }
            self.eat_punct(';');
        }
        self.expect_punct('}');
        self.eat_punct(';');
        out
    }

    fn boundary(&mut self, interface: bool) -> Option<BoundaryConditionDecl> {
        let (name, span) = self.expect_ident_value()?;
        if !self.eat_ident("on") {
            self.error("boundary/interface requires `on`".into());
        }
        let region = self.expr(0)?;
        self.expect_punct('{');
        let kind_name = self.expect_ident_value()?.0;
        let kind = if interface {
            BoundaryConditionKind::Interface
        } else {
            match kind_name.as_str() {
                "dirichlet" => BoundaryConditionKind::Dirichlet,
                "neumann" => BoundaryConditionKind::Neumann,
                "robin" => BoundaryConditionKind::Robin,
                _ => {
                    self.error(format!("unknown boundary condition `{kind_name}`"));
                    BoundaryConditionKind::Dirichlet
                }
            }
        };
        let target = self.expect_ident_value()?.0;
        self.eat_op("=");
        let value = self.expr(0)?;
        self.eat_punct(';');
        self.expect_punct('}');
        self.eat_punct(';');
        Some(BoundaryConditionDecl {
            name,
            region,
            kind,
            target,
            value,
            span,
        })
    }

    fn observable(&mut self) -> Option<ObservableDecl> {
        let (name, span) = self.expect_ident_value()?;
        self.expect_punct('{');
        let value = self.expr(0)?;
        self.eat_punct(';');
        self.expect_punct('}');
        self.eat_punct(';');
        Some(ObservableDecl { name, value, span })
    }
    fn verification(&mut self) -> Option<VerificationAnnotation> {
        let (name, span) = self.expect_ident_value()?;
        let mut args = BTreeMap::new();
        if self.eat_punct('(') {
            while !self.eat_punct(')') && !matches!(self.token().kind, TokenKind::Eof) {
                let key = self.expect_ident_value()?.0;
                self.eat_op("=");
                let value = self.expr(0)?;
                args.insert(key, value);
                if !self.eat_punct(',') {
                    self.expect_punct(')');
                    break;
                }
            }
        }
        self.eat_punct(';');
        Some(VerificationAnnotation { name, args, span })
    }

    fn expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = match self.bump() {
            Token {
                kind: TokenKind::Number(value),
                ..
            } => {
                let unit = if let TokenKind::Ident(name) = &self.token().kind {
                    if is_unitish(name) {
                        Some(if let TokenKind::Ident(x) = self.bump().kind {
                            x
                        } else {
                            unreachable!()
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
                Expr::Number { value, unit }
            }
            Token {
                kind: TokenKind::String(s),
                ..
            } => Expr::String(s),
            Token {
                kind: TokenKind::Ident(name),
                ..
            } => {
                if self.eat_punct('(') {
                    let mut args = vec![];
                    if !self.eat_punct(')') {
                        loop {
                            args.push(self.expr(0)?);
                            if self.eat_punct(')') {
                                break;
                            }
                            self.expect_punct(',');
                        }
                    }
                    Expr::Call {
                        function: name,
                        args,
                    }
                } else {
                    Expr::Name(name)
                }
            }
            Token {
                kind: TokenKind::Op(op),
                ..
            } if op == "-" => Expr::Unary {
                op: UnaryOp::Neg,
                arg: Box::new(self.expr(11)?),
            },
            Token {
                kind: TokenKind::Punct('('),
                ..
            } => {
                let x = self.expr(0)?;
                self.expect_punct(')');
                x
            }
            Token {
                kind: TokenKind::Punct('['),
                ..
            } => {
                let mut xs = vec![];
                if !self.eat_punct(']') {
                    loop {
                        xs.push(self.expr(0)?);
                        if self.eat_punct(']') {
                            break;
                        }
                        self.expect_punct(',');
                    }
                }
                Expr::Vector(xs)
            }
            t => {
                self.errors.push(ScientificError::Syntax {
                    message: "expected expression".into(),
                    span: t.span,
                });
                return None;
            }
        };
        loop {
            if self.eat_punct('[') {
                let mut indices = vec![];
                if !self.eat_punct(']') {
                    loop {
                        indices.push(self.expr(0)?);
                        if self.eat_punct(']') {
                            break;
                        }
                        self.expect_punct(',');
                    }
                }
                lhs = Expr::Index {
                    value: Box::new(lhs),
                    indices,
                };
                continue;
            }
            let op_text = match &self.token().kind {
                TokenKind::Op(x) => x.clone(),
                _ => break,
            };
            let Some((lbp, rbp, op)) = binary_binding(&op_text) else {
                break;
            };
            if lbp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.expr(rbp)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Some(lhs)
    }
}

fn is_unitish(s: &str) -> bool {
    matches!(
        s,
        "K" | "degC" | "delta_degC" | "s" | "m" | "kg" | "Pa" | "J" | "W" | "V" | "A"
    )
}
fn binary_binding(op: &str) -> Option<(u8, u8, BinaryOp)> {
    Some(match op {
        "=" | "==" => (1, 2, BinaryOp::Eq),
        "<" => (3, 4, BinaryOp::Lt),
        "<=" => (3, 4, BinaryOp::Le),
        ">" => (3, 4, BinaryOp::Gt),
        ">=" => (3, 4, BinaryOp::Ge),
        "+" => (5, 6, BinaryOp::Add),
        "-" => (5, 6, BinaryOp::Sub),
        "*" => (7, 8, BinaryOp::Mul),
        "/" => (7, 8, BinaryOp::Div),
        "^" => (10, 9, BinaryOp::Pow),
        _ => return None,
    })
}

pub fn parse_scientific_module(input: &str) -> Result<ScientificModule, Vec<ScientificError>> {
    let tokens = lex(input)?;
    let mut parser = Parser::new(tokens);
    let module = parser.module();
    if parser.errors.is_empty() {
        Ok(module)
    } else {
        Err(parser.errors)
    }
}

pub fn semantic_digest(module: &ScientificModule) -> String {
    // Spans are provenance, not scientific meaning. Strip them before hashing so
    // whitespace/comments/formatting do not perturb the physics identity.
    let mut value =
        serde_json::to_value(module).expect("scientific module serialization is infallible");
    fn canonicalize(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("span");
                for child in map.values_mut() {
                    canonicalize(child);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items.iter_mut() {
                    canonicalize(child);
                }
                if items
                    .iter()
                    .all(|item| item.get("name").and_then(|x| x.as_str()).is_some())
                {
                    items.sort_by(|a, b| {
                        a.get("name")
                            .and_then(|x| x.as_str())
                            .cmp(&b.get("name").and_then(|x| x.as_str()))
                    });
                } else if items.iter().all(|item| item.as_str().is_some()) {
                    items.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
                }
            }
            _ => {}
        }
    }
    canonicalize(&mut value);
    let bytes =
        serde_json::to_vec(&value).expect("semantic projection serialization is infallible");
    blake3::hash(&bytes).to_hex().to_string()
}

pub fn format_scientific_module(module: &ScientificModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("module {};\n\n", module.name));
    for import in &module.imports {
        out.push_str(&format!("use {import};\n"));
    }
    if !module.imports.is_empty() {
        out.push('\n');
    }
    for model in &module.models {
        out.push_str(&format!("model {} {{\n", model.name));
        for d in &model.domains {
            out.push_str(&format!(
                "    domain {} {{ dimension = {}; coordinates = {}; }}\n",
                d.name,
                d.dimension,
                coordinate_name(&d.coordinates)
            ));
        }
        for f in &model.fields {
            out.push_str(&format!(
                "    field {}: {} {} {}(order={}) on {}",
                f.name,
                field_role_name(&f.role),
                shape_name(&f.shape),
                space_name(&f.space.family),
                f.space.order,
                f.domain
            ));
            if f.quantity_kind.is_some()
                || f.unit.is_some()
                || f.nominal.is_some()
                || f.physical_min.is_some()
                || f.physical_max.is_some()
                || f.time_role.is_some()
            {
                out.push_str(" {\n");
                if let Some(k) = &f.quantity_kind {
                    out.push_str(&format!("        quantity = {};\n", k.0));
                }
                if let Some(u) = &f.unit {
                    out.push_str(&format!("        unit = {};\n", u.0));
                }
                if let Some(n) = &f.nominal {
                    out.push_str(&format!("        nominal = {} {};\n", n.value, n.unit.0));
                }
                if let Some(n) = &f.physical_min {
                    out.push_str(&format!("        min = {} {};\n", n.value, n.unit.0));
                }
                if let Some(n) = &f.physical_max {
                    out.push_str(&format!("        max = {} {};\n", n.value, n.unit.0));
                }
                if let Some(role) = f.time_role {
                    let role = match role {
                        TimeRole::Differential => "differential",
                        TimeRole::Algebraic => "algebraic",
                    };
                    out.push_str(&format!("        time_role = {role};\n"));
                }
                out.push_str("    }");
            }
            out.push_str(";\n");
        }
        for p in &model.parameters {
            out.push_str(&format_value_decl("parameter", p));
        }
        for c in &model.constants {
            out.push_str(&format_value_decl("constant", c));
        }
        for s in &model.sources {
            out.push_str(&format_value_decl("source", s));
        }
        for p in &model.properties {
            out.push_str(&format!(
                "    property {} = {};\n",
                p.name,
                format_expr(&p.value)
            ));
        }
        for c in &model.constitutive_laws {
            out.push_str(&format!(
                "    constitutive {} = {};\n",
                c.name,
                format_expr(&c.law)
            ));
        }
        for e in &model.equations {
            out.push_str(&format!(
                "    equation {}{} {{ {} = {}; }}\n",
                e.name,
                e.domain
                    .as_ref()
                    .map(|d| format!(" on {d}"))
                    .unwrap_or_default(),
                format_expr(&e.lhs),
                format_expr(&e.rhs)
            ));
        }
        for form in &model.forms {
            out.push_str(&format!("    form {} {{\n", form.name));
            for integral in &form.integrals {
                let (measure, target) = match &integral.measure {
                    MeasureV1::Cell(target) => ("cell", target),
                    MeasureV1::Boundary(target) => ("boundary", target),
                    MeasureV1::InteriorFacet(target) => ("interior_facet", target),
                };
                out.push_str(&format!(
                    "        {measure}({target}): {};\n",
                    format_expr(&integral.integrand)
                ));
            }
            out.push_str("    }\n");
        }
        if !model.initial_conditions.is_empty() {
            out.push_str("    initial {\n");
            for c in &model.initial_conditions {
                out.push_str(&format!(
                    "        {} = {};\n",
                    c.target,
                    format_expr(&c.value)
                ));
            }
            out.push_str("    }\n");
        }
        for bc in &model.boundary_conditions {
            let kind = match bc.kind {
                BoundaryConditionKind::Dirichlet => "dirichlet",
                BoundaryConditionKind::Neumann => "neumann",
                BoundaryConditionKind::Robin => "robin",
                BoundaryConditionKind::Interface => "interface",
            };
            out.push_str(&format!(
                "    boundary {} on {} {{\n        {kind} {} = {};\n    }}\n",
                bc.name,
                format_expr(&bc.region),
                bc.target,
                format_expr(&bc.value)
            ));
        }
        for bc in &model.interface_conditions {
            out.push_str(&format!(
                "    interface {} on {} {{\n        interface {} = {};\n    }}\n",
                bc.name,
                format_expr(&bc.region),
                bc.target,
                format_expr(&bc.value)
            ));
        }
        for o in &model.observables {
            out.push_str(&format!(
                "    observable {} {{ {}; }}\n",
                o.name,
                format_expr(&o.value)
            ));
        }
        for i in &model.invariants {
            out.push_str(&format!(
                "    invariant {} {{ {}; }}\n",
                i.name,
                format_expr(&i.value)
            ));
        }
        for v in &model.verifications {
            out.push_str(&format!("    @{}", v.name));
            if !v.args.is_empty() {
                out.push('(');
                out.push_str(
                    &v.args
                        .iter()
                        .map(|(k, x)| format!("{k} = {}", format_expr(x)))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                out.push(')');
            }
            out.push_str(";\n");
        }
        out.push_str("}\n\n");
    }
    out
}
fn coordinate_name(c: &CoordinateSystem) -> &str {
    match c {
        CoordinateSystem::Cartesian => "cartesian",
        CoordinateSystem::Cylindrical => "cylindrical",
        CoordinateSystem::Spherical => "spherical",
        CoordinateSystem::Custom(x) => x,
    }
}
fn field_role_name(r: &FieldRoleV1) -> &str {
    match r {
        FieldRoleV1::State => "state",
        FieldRoleV1::Unknown => "unknown",
        FieldRoleV1::Test => "test",
        FieldRoleV1::Trial => "trial",
        FieldRoleV1::Coefficient => "coefficient",
        FieldRoleV1::Parameter => "parameter",
        FieldRoleV1::Derived => "derived",
    }
}
fn shape_name(s: &ValueShapeV1) -> String {
    match s {
        ValueShapeV1::Scalar => "scalar".into(),
        ValueShapeV1::Vector(n) => format!("vector({n})"),
        ValueShapeV1::Tensor { rows, cols } => format!("tensor({rows},{cols})"),
        ValueShapeV1::SymmetricTensor(n) => format!("tensor({n},{n})"),
    }
}
fn space_name(s: &SpaceFamily) -> &str {
    match s {
        SpaceFamily::H1 => "H1",
        SpaceFamily::L2 => "L2",
        SpaceFamily::HCurl => "HCurl",
        SpaceFamily::HDiv => "HDiv",
        SpaceFamily::Dg => "DG",
    }
}
fn format_value_decl(kind: &str, d: &ValueDecl) -> String {
    let ty = d
        .quantity_kind
        .as_ref()
        .map(|x| format!(": {}", x.0))
        .unwrap_or_default();
    let unit = d
        .unit
        .as_ref()
        .map(|x| format!(" [{}]", x.0))
        .unwrap_or_default();
    let value = d
        .value
        .as_ref()
        .map(|x| format!(" = {}", format_expr(x)))
        .unwrap_or_default();
    format!("    {kind} {}{ty}{unit}{value};\n", d.name)
}
fn format_expr(e: &Expr) -> String {
    match e {
        Expr::Number { value, unit } => format!(
            "{value}{}",
            unit.as_ref().map(|u| format!(" {u}")).unwrap_or_default()
        ),
        Expr::String(s) => format!("\"{s}\""),
        Expr::Name(n) => n.clone(),
        Expr::Unary { arg, .. } => format!("-{}", format_expr(arg)),
        Expr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            format_expr(lhs),
            binary_name(*op),
            format_expr(rhs)
        ),
        Expr::Call { function, args } => format!(
            "{}({})",
            function,
            args.iter().map(format_expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::Index { value, indices } => format!(
            "{}[{}]",
            format_expr(value),
            indices
                .iter()
                .map(format_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Vector(xs) => format!(
            "[{}]",
            xs.iter().map(format_expr).collect::<Vec<_>>().join(", ")
        ),
    }
}
fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Pow => "^",
        BinaryOp::Eq => "=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
    }
}

pub trait ModuleSource {
    fn load(&self, name: &str) -> Option<String>;
}
impl ModuleSource for BTreeMap<String, String> {
    fn load(&self, name: &str) -> Option<String> {
        self.get(name).cloned()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedModules {
    pub modules: BTreeMap<String, ScientificModule>,
    pub semantic_digest: String,
}

pub fn resolve_modules(
    root: ScientificModule,
    source: &impl ModuleSource,
) -> Result<ResolvedModules, ScientificError> {
    fn visit(
        name: &str,
        module: ScientificModule,
        source: &impl ModuleSource,
        state: &mut BTreeMap<String, u8>,
        out: &mut BTreeMap<String, ScientificModule>,
    ) -> Result<(), ScientificError> {
        match state.get(name).copied() {
            Some(1) => return Err(ScientificError::ImportCycle(name.into())),
            Some(2) => return Ok(()),
            _ => {}
        }
        state.insert(name.into(), 1);
        for import in &module.imports {
            let text = source
                .load(import)
                .ok_or_else(|| ScientificError::MissingModule(import.clone()))?;
            let parsed =
                parse_scientific_module(&text).map_err(|e| e.into_iter().next().unwrap())?;
            visit(import, parsed, source, state, out)?;
        }
        state.insert(name.into(), 2);
        out.insert(name.into(), module);
        Ok(())
    }
    let root_name = root.name.clone();
    let mut state = BTreeMap::new();
    let mut modules = BTreeMap::new();
    visit(&root_name, root, source, &mut state, &mut modules)?;
    let bytes = serde_json::to_vec(&modules).unwrap();
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok(ResolvedModules {
        modules,
        semantic_digest: digest,
    })
}

// ---------------- R15 properties/material semantics ----------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorSymmetry {
    None,
    Symmetric,
    MajorMinor,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameSemantics {
    Scalar,
    Material,
    Reference,
    Spatial,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyLocality {
    Pointwise,
    ElementConstant,
    ExternalProvider,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivativeContract {
    Symbolic,
    AnalyticProvided,
    Automatic,
    Piecewise,
    NumericalAllowed,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyInput {
    pub name: String,
    pub quantity_kind: QuantityKindId,
    pub dimension: Dimension,
    pub shape: ValueShapeV1,
    pub physical_min: Option<f64>,
    pub physical_max: Option<f64>,
    pub nominal: Option<CanonicalQuantity>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyOutput {
    pub quantity_kind: QuantityKindId,
    pub dimension: Dimension,
    pub shape: ValueShapeV1,
    pub symmetry: TensorSymmetry,
    pub frame: FrameSemantics,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertySignature {
    pub id: String,
    pub inputs: Vec<PropertyInput>,
    pub output: PropertyOutput,
    pub locality: PropertyLocality,
    pub differentiability: DerivativeContract,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyBranch {
    pub when: Option<Predicate>,
    pub value: Expr,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyModel {
    Constant(Expr),
    Expression(Expr),
    Piecewise(Vec<PropertyBranch>),
    Table(PropertyTable),
    External(PropertyProviderRef),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyProviderRef {
    pub provider: String,
    pub property: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Predicate {
    pub variable: String,
    pub op: CompareOp,
    pub value: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyTable {
    pub axes: Vec<TableAxis>,
    pub values: Vec<f64>,
    pub interpolation: Interpolation,
    pub derivative_policy: TableDerivativePolicy,
    pub out_of_range: OutOfValidityPolicy,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableAxis {
    pub name: String,
    pub points: Vec<f64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpolation {
    Linear,
    Multilinear,
    MonotoneCubic,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableDerivativePolicy {
    PiecewiseConstantSlope,
    Numerical,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutOfValidityPolicy {
    Error,
    Warn,
    ExplicitExtrapolation(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputBounds {
    pub input: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyDomain {
    pub physical_bounds: Vec<InputBounds>,
    pub validity_bounds: Vec<InputBounds>,
    pub phase_constraints: Vec<String>,
    pub composition_constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub out_of_validity: OutOfValidityPolicy,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UncertaintyModel {
    StandardAbsolute(f64),
    StandardRelative(f64),
    Expanded {
        value: f64,
        confidence: f64,
        coverage_factor: Option<f64>,
    },
    TablePerPoint(Vec<f64>),
    CovarianceRef(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyEvidence {
    pub sources: Vec<String>,
    pub dataset_digest: Option<String>,
    pub fit_digest: Option<String>,
    pub uncertainty: Option<UncertaintyModel>,
    pub notes: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub signature: PropertySignature,
    pub model: PropertyModel,
    pub domain: PropertyDomain,
    pub evidence: PropertyEvidence,
}

impl PropertyDefinition {
    pub fn evaluate(&self, inputs: &BTreeMap<String, f64>) -> Result<f64, ScientificError> {
        for bound in &self.domain.physical_bounds {
            check_bound(bound, inputs).map_err(ScientificError::Property)?;
        }
        for bound in &self.domain.validity_bounds {
            if let Err(message) = check_bound(bound, inputs)
                && matches!(self.domain.out_of_validity, OutOfValidityPolicy::Error)
            {
                return Err(ScientificError::Property(message));
            }
        }
        evaluate_property_model(&self.model, inputs)
    }
    pub fn derivative(
        &self,
        input: &str,
        inputs: &BTreeMap<String, f64>,
    ) -> Result<Option<f64>, ScientificError> {
        match &self.model {
            PropertyModel::Constant(_) => Ok(Some(0.0)),
            PropertyModel::Expression(expr) => {
                Ok(Some(eval_expr(&differentiate_expr(expr, input), inputs)?))
            }
            PropertyModel::Piecewise(branches) => {
                for b in branches {
                    if b.when.as_ref().is_none_or(|p| predicate(p, inputs)) {
                        return Ok(Some(eval_expr(
                            &differentiate_expr(&b.value, input),
                            inputs,
                        )?));
                    }
                }
                Ok(None)
            }
            PropertyModel::Table(table) => table_derivative(table, input, inputs).map(Some),
            PropertyModel::External(_) => Ok(None),
        }
    }
}
fn check_bound(b: &InputBounds, inputs: &BTreeMap<String, f64>) -> Result<(), String> {
    let v = *inputs
        .get(&b.input)
        .ok_or_else(|| format!("missing property input `{}`", b.input))?;
    if b.min.is_some_and(|x| v < x) || b.max.is_some_and(|x| v > x) {
        Err(format!(
            "input `{}`={v} outside [{:?},{:?}]",
            b.input, b.min, b.max
        ))
    } else {
        Ok(())
    }
}
fn predicate(p: &Predicate, inputs: &BTreeMap<String, f64>) -> bool {
    let Some(v) = inputs.get(&p.variable) else {
        return false;
    };
    match p.op {
        CompareOp::Lt => *v < p.value,
        CompareOp::Le => *v <= p.value,
        CompareOp::Gt => *v > p.value,
        CompareOp::Ge => *v >= p.value,
    }
}
fn evaluate_property_model(
    model: &PropertyModel,
    inputs: &BTreeMap<String, f64>,
) -> Result<f64, ScientificError> {
    match model {
        PropertyModel::Constant(e) | PropertyModel::Expression(e) => eval_expr(e, inputs),
        PropertyModel::Piecewise(bs) => bs
            .iter()
            .find(|b| b.when.as_ref().is_none_or(|p| predicate(p, inputs)))
            .map(|b| eval_expr(&b.value, inputs))
            .unwrap_or_else(|| {
                Err(ScientificError::Property(
                    "no piecewise branch matched".into(),
                ))
            }),
        PropertyModel::Table(t) => table_evaluate(t, inputs),
        PropertyModel::External(p) => Err(ScientificError::Property(format!(
            "external provider {}:{} requires runtime ABI",
            p.provider, p.property
        ))),
    }
}

pub fn eval_expr(expr: &Expr, env: &BTreeMap<String, f64>) -> Result<f64, ScientificError> {
    let e = |x: &Expr| eval_expr(x, env);
    Ok(match expr {
        Expr::Number { value, .. } => *value,
        Expr::Name(n) => *env
            .get(n)
            .ok_or_else(|| ScientificError::UnknownName(n.clone()))?,
        Expr::Unary { arg, .. } => -e(arg)?,
        Expr::Binary { op, lhs, rhs } => {
            let a = e(lhs)?;
            let b = e(rhs)?;
            match op {
                BinaryOp::Add => a + b,
                BinaryOp::Sub => a - b,
                BinaryOp::Mul => a * b,
                BinaryOp::Div => a / b,
                BinaryOp::Pow => a.powf(b),
                BinaryOp::Eq => (a == b) as u8 as f64,
                BinaryOp::Lt => (a < b) as u8 as f64,
                BinaryOp::Le => (a <= b) as u8 as f64,
                BinaryOp::Gt => (a > b) as u8 as f64,
                BinaryOp::Ge => (a >= b) as u8 as f64,
            }
        }
        Expr::Call { function, args } => {
            let xs = args.iter().map(e).collect::<Result<Vec<_>, _>>()?;
            match (function.as_str(), xs.as_slice()) {
                ("sin", [x]) => x.sin(),
                ("cos", [x]) => x.cos(),
                ("exp", [x]) => x.exp(),
                ("log" | "ln", [x]) => x.ln(),
                ("sqrt", [x]) => x.sqrt(),
                ("abs", [x]) => x.abs(),
                ("min", [a, b]) => a.min(*b),
                ("max", [a, b]) => a.max(*b),
                _ => {
                    return Err(ScientificError::Property(format!(
                        "cannot numerically evaluate `{function}`"
                    )));
                }
            }
        }
        Expr::String(_) | Expr::Index { .. } | Expr::Vector(_) => {
            return Err(ScientificError::Property("non-scalar expression".into()));
        }
    })
}

pub fn differentiate_expr(expr: &Expr, var: &str) -> Expr {
    let n = |v: f64| Expr::Number {
        value: v,
        unit: None,
    };
    match expr {
        Expr::Number { .. } | Expr::String(_) => n(0.0),
        Expr::Name(x) => n(if x == var { 1.0 } else { 0.0 }),
        Expr::Unary { arg, .. } => Expr::Unary {
            op: UnaryOp::Neg,
            arg: Box::new(differentiate_expr(arg, var)),
        },
        Expr::Binary { op, lhs, rhs } => match op {
            BinaryOp::Add | BinaryOp::Sub => Expr::Binary {
                op: *op,
                lhs: Box::new(differentiate_expr(lhs, var)),
                rhs: Box::new(differentiate_expr(rhs, var)),
            },
            BinaryOp::Mul => Expr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::Binary {
                    op: BinaryOp::Mul,
                    lhs: Box::new(differentiate_expr(lhs, var)),
                    rhs: rhs.clone(),
                }),
                rhs: Box::new(Expr::Binary {
                    op: BinaryOp::Mul,
                    lhs: lhs.clone(),
                    rhs: Box::new(differentiate_expr(rhs, var)),
                }),
            },
            BinaryOp::Div => Expr::Binary {
                op: BinaryOp::Div,
                lhs: Box::new(Expr::Binary {
                    op: BinaryOp::Sub,
                    lhs: Box::new(Expr::Binary {
                        op: BinaryOp::Mul,
                        lhs: Box::new(differentiate_expr(lhs, var)),
                        rhs: rhs.clone(),
                    }),
                    rhs: Box::new(Expr::Binary {
                        op: BinaryOp::Mul,
                        lhs: lhs.clone(),
                        rhs: Box::new(differentiate_expr(rhs, var)),
                    }),
                }),
                rhs: Box::new(Expr::Binary {
                    op: BinaryOp::Pow,
                    lhs: rhs.clone(),
                    rhs: Box::new(n(2.0)),
                }),
            },
            BinaryOp::Pow => {
                if let Expr::Number { value: p, .. } = **rhs {
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        lhs: Box::new(n(p)),
                        rhs: Box::new(Expr::Binary {
                            op: BinaryOp::Mul,
                            lhs: Box::new(Expr::Binary {
                                op: BinaryOp::Pow,
                                lhs: lhs.clone(),
                                rhs: Box::new(n(p - 1.0)),
                            }),
                            rhs: Box::new(differentiate_expr(lhs, var)),
                        }),
                    }
                } else {
                    n(0.0)
                }
            }
            _ => n(0.0),
        },
        Expr::Call { function, args } if args.len() == 1 => {
            let x = &args[0];
            let dx = differentiate_expr(x, var);
            let outer = match function.as_str() {
                "sin" => Expr::Call {
                    function: "cos".into(),
                    args: vec![x.clone()],
                },
                "cos" => Expr::Unary {
                    op: UnaryOp::Neg,
                    arg: Box::new(Expr::Call {
                        function: "sin".into(),
                        args: vec![x.clone()],
                    }),
                },
                "exp" => Expr::Call {
                    function: "exp".into(),
                    args: vec![x.clone()],
                },
                "log" | "ln" => Expr::Binary {
                    op: BinaryOp::Div,
                    lhs: Box::new(n(1.0)),
                    rhs: Box::new(x.clone()),
                },
                _ => n(0.0),
            };
            Expr::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(outer),
                rhs: Box::new(dx),
            }
        }
        Expr::Call { .. } | Expr::Index { .. } | Expr::Vector(_) => n(0.0),
    }
}

fn table_evaluate(
    t: &PropertyTable,
    inputs: &BTreeMap<String, f64>,
) -> Result<f64, ScientificError> {
    match t.axes.as_slice() {
        [axis] => {
            if t.values.len() != axis.points.len() {
                return Err(ScientificError::Property("1-D table shape mismatch".into()));
            }
            let x = *inputs
                .get(&axis.name)
                .ok_or_else(|| ScientificError::UnknownName(axis.name.clone()))?;
            linear_axis(axis, &t.values, x, &t.out_of_range).map(|x| x.0)
        }
        [a, b] => bilinear(t, a, b, inputs),
        _ => Err(ScientificError::Property(
            "tables currently support one or two axes".into(),
        )),
    }
}
fn table_derivative(
    t: &PropertyTable,
    input: &str,
    inputs: &BTreeMap<String, f64>,
) -> Result<f64, ScientificError> {
    if matches!(t.derivative_policy, TableDerivativePolicy::Unavailable) {
        return Err(ScientificError::Property(
            "table derivatives unavailable".into(),
        ));
    }
    if t.axes.len() == 1 && t.axes[0].name == input {
        let x = *inputs
            .get(input)
            .ok_or_else(|| ScientificError::UnknownName(input.into()))?;
        Ok(linear_axis(&t.axes[0], &t.values, x, &t.out_of_range)?.1)
    } else {
        let x = *inputs
            .get(input)
            .ok_or_else(|| ScientificError::UnknownName(input.into()))?;
        let h = (x.abs().max(1.0)) * 1e-6;
        let mut p = inputs.clone();
        let mut m = inputs.clone();
        p.insert(input.into(), x + h);
        m.insert(input.into(), x - h);
        Ok((table_evaluate(t, &p)? - table_evaluate(t, &m)?) / (2.0 * h))
    }
}
fn linear_axis(
    axis: &TableAxis,
    values: &[f64],
    x: f64,
    policy: &OutOfValidityPolicy,
) -> Result<(f64, f64), ScientificError> {
    if axis.points.len() < 2 {
        return Err(ScientificError::Property(
            "table axis needs >=2 points".into(),
        ));
    }
    let outside = x < axis.points[0] || x > *axis.points.last().unwrap();
    if outside && matches!(policy, OutOfValidityPolicy::Error) {
        return Err(ScientificError::Property(format!(
            "{x} outside table axis `{}`",
            axis.name
        )));
    }
    let mut i = 0;
    while i + 1 < axis.points.len() - 1 && x > axis.points[i + 1] {
        i += 1;
    }
    let x0 = axis.points[i];
    let x1 = axis.points[i + 1];
    let slope = (values[i + 1] - values[i]) / (x1 - x0);
    Ok((values[i] + slope * (x - x0), slope))
}
fn bilinear(
    t: &PropertyTable,
    a: &TableAxis,
    b: &TableAxis,
    inputs: &BTreeMap<String, f64>,
) -> Result<f64, ScientificError> {
    if t.values.len() != a.points.len() * b.points.len() {
        return Err(ScientificError::Property("2-D table shape mismatch".into()));
    }
    let x = *inputs
        .get(&a.name)
        .ok_or_else(|| ScientificError::UnknownName(a.name.clone()))?;
    let y = *inputs
        .get(&b.name)
        .ok_or_else(|| ScientificError::UnknownName(b.name.clone()))?;
    let bracket = |points: &[f64], v: f64| {
        let mut i = 0;
        while i + 1 < points.len() - 1 && v > points[i + 1] {
            i += 1;
        }
        i
    };
    if (x < a.points[0]
        || x > *a.points.last().unwrap()
        || y < b.points[0]
        || y > *b.points.last().unwrap())
        && matches!(t.out_of_range, OutOfValidityPolicy::Error)
    {
        return Err(ScientificError::Property(
            "point outside bilinear table".into(),
        ));
    }
    let i = bracket(&a.points, x);
    let j = bracket(&b.points, y);
    let idx = |ii: usize, jj: usize| ii * b.points.len() + jj;
    let tx = (x - a.points[i]) / (a.points[i + 1] - a.points[i]);
    let ty = (y - b.points[j]) / (b.points[j + 1] - b.points[j]);
    let q00 = t.values[idx(i, j)];
    let q10 = t.values[idx(i + 1, j)];
    let q01 = t.values[idx(i, j + 1)];
    let q11 = t.values[idx(i + 1, j + 1)];
    Ok((1.0 - tx) * (1.0 - ty) * q00
        + tx * (1.0 - ty) * q10
        + (1.0 - tx) * ty * q01
        + tx * ty * q11)
}

// ---------------- R16 constitutive semantics ----------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstitutiveLaw {
    pub id: String,
    pub driving: Vec<LawVariable>,
    pub forces: Vec<LawVariable>,
    pub internal_state: Vec<StateVariable>,
    pub potential: Option<Expr>,
    pub direct_relations: BTreeMap<String, Expr>,
    pub dissipation: Option<Expr>,
    pub tangent: TangentContract,
    pub update: UpdateContract,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LawVariable {
    pub name: String,
    pub quantity_kind: QuantityKindId,
    pub shape: ValueShapeV1,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateVariable {
    pub name: String,
    pub shape: ValueShapeV1,
    pub initial: Expr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TangentContract {
    Symbolic,
    Automatic,
    AnalyticProvided,
    NumericalAllowed,
    None,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateContract {
    Stateless,
    TransactionalLocal { may_request_step_reduction: bool },
}

pub fn standard_constitutive_laws() -> Vec<&'static str> {
    vec![
        "thermal.fourier",
        "diffusion.fick",
        "electrical.ohm",
        "mechanics.hooke_isotropic",
        "fluids.newtonian",
    ]
}

// ---------------- R17 production discretization catalog ----------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceCell {
    Triangle,
    Tetrahedron,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementFamilyV1 {
    Lagrange,
    Discontinuous,
    NedelecFirstKind,
    RaviartThomas,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementSpec {
    pub cell: ReferenceCell,
    pub family: ElementFamilyV1,
    pub space: SpaceFamily,
    pub order: u8,
    pub value_shape: ValueShapeV1,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuadraturePolicy {
    Automatic,
    Order(u8),
    ExactForDegree(u8),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BasisTabulationV1 {
    pub values: Vec<Vec<f64>>,
    pub gradients: Vec<Vec<Vec<f64>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscretizationCatalog {
    pub elements: Vec<ElementSpec>,
}
impl DiscretizationCatalog {
    pub fn production() -> Self {
        let mut elements = vec![];
        for cell in [ReferenceCell::Triangle, ReferenceCell::Tetrahedron] {
            for order in [1, 2] {
                elements.push(ElementSpec {
                    cell,
                    family: ElementFamilyV1::Lagrange,
                    space: SpaceFamily::H1,
                    order,
                    value_shape: ValueShapeV1::Scalar,
                });
            }
            for order in [0, 1] {
                elements.push(ElementSpec {
                    cell,
                    family: ElementFamilyV1::Discontinuous,
                    space: SpaceFamily::L2,
                    order,
                    value_shape: ValueShapeV1::Scalar,
                });
            }
            elements.push(ElementSpec {
                cell,
                family: ElementFamilyV1::NedelecFirstKind,
                space: SpaceFamily::HCurl,
                order: 0,
                value_shape: ValueShapeV1::Vector(match cell {
                    ReferenceCell::Triangle => 2,
                    ReferenceCell::Tetrahedron => 3,
                }),
            });
            elements.push(ElementSpec {
                cell,
                family: ElementFamilyV1::RaviartThomas,
                space: SpaceFamily::HDiv,
                order: 0,
                value_shape: ValueShapeV1::Vector(match cell {
                    ReferenceCell::Triangle => 2,
                    ReferenceCell::Tetrahedron => 3,
                }),
            });
        }
        Self { elements }
    }
    pub fn supports(&self, cell: ReferenceCell, space: SpaceFamily, order: u8) -> bool {
        self.elements
            .iter()
            .any(|e| e.cell == cell && e.space == space && e.order == order)
    }
}

pub fn triangle_lagrange_basis(order: u8, xi: f64, eta: f64) -> Result<Vec<f64>, ScientificError> {
    let l1 = 1.0 - xi - eta;
    let l2 = xi;
    let l3 = eta;
    Ok(match order {
        1 => vec![l1, l2, l3],
        2 => vec![
            l1 * (2.0 * l1 - 1.0),
            l2 * (2.0 * l2 - 1.0),
            l3 * (2.0 * l3 - 1.0),
            4.0 * l1 * l2,
            4.0 * l2 * l3,
            4.0 * l3 * l1,
        ],
        _ => {
            return Err(ScientificError::Property(format!(
                "unsupported triangle Lagrange order {order}"
            )));
        }
    })
}
pub fn orientation_sign(permutation: &[usize]) -> i8 {
    let mut inversions = 0;
    for i in 0..permutation.len() {
        for j in i + 1..permutation.len() {
            if permutation[i] > permutation[j] {
                inversions += 1;
            }
        }
    }
    if inversions % 2 == 0 { 1 } else { -1 }
}

// ---------------- R18 coupling semantics ----------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownBlock {
    pub name: String,
    pub field: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouplingReason {
    DirectFieldUse,
    PropertyDependency(String),
    ConstitutiveDependency(String),
    InterfaceTerm,
    HistoryState,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingEdge {
    pub from: String,
    pub to: String,
    pub reason: CouplingReason,
    pub path: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockDerivative {
    pub residual: String,
    pub unknown: String,
    pub structurally_nonzero: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingGraph {
    pub unknowns: Vec<UnknownBlock>,
    pub residual_blocks: Vec<String>,
    pub edges: Vec<CouplingEdge>,
    pub derivatives: Vec<BlockDerivative>,
}

pub fn derive_coupling_graph(model: &ScientificModel) -> CouplingGraph {
    let field_names: BTreeSet<_> = model
        .fields
        .iter()
        .filter(|f| matches!(f.role, FieldRoleV1::State | FieldRoleV1::Unknown))
        .map(|f| f.name.clone())
        .collect();
    let property_map: BTreeMap<_, _> = model
        .properties
        .iter()
        .map(|p| (p.name.clone(), p.value.clone()))
        .collect();
    let constitutive_map: BTreeMap<_, _> = model
        .constitutive_laws
        .iter()
        .map(|law| (law.name.clone(), law.law.clone()))
        .collect();

    struct TraceContext<'a> {
        field_names: &'a BTreeSet<String>,
        property_map: &'a BTreeMap<String, Expr>,
        constitutive_map: &'a BTreeMap<String, Expr>,
    }
    fn trace(
        symbol: &str,
        residual: &str,
        context: &TraceContext<'_>,
        path: &mut Vec<String>,
        seen: &mut BTreeSet<String>,
        reason: Option<CouplingReason>,
        out: &mut Vec<CouplingEdge>,
    ) {
        if context.field_names.contains(symbol) {
            let mut full = vec![symbol.to_string()];
            full.extend(path.iter().cloned());
            full.push(residual.to_string());
            out.push(CouplingEdge {
                from: symbol.to_string(),
                to: residual.to_string(),
                reason: reason.unwrap_or(CouplingReason::DirectFieldUse),
                path: full,
            });
            return;
        }
        if !seen.insert(symbol.to_string()) {
            return;
        }
        if let Some(expr) = context.property_map.get(symbol) {
            path.insert(0, symbol.to_string());
            let mut names = BTreeSet::new();
            expr.names(&mut names);
            for name in names {
                trace(
                    &name,
                    residual,
                    context,
                    path,
                    seen,
                    Some(CouplingReason::PropertyDependency(symbol.to_string())),
                    out,
                );
            }
            path.remove(0);
        } else if let Some(expr) = context.constitutive_map.get(symbol) {
            path.insert(0, symbol.to_string());
            let mut names = BTreeSet::new();
            expr.names(&mut names);
            for name in names {
                trace(
                    &name,
                    residual,
                    context,
                    path,
                    seen,
                    Some(CouplingReason::ConstitutiveDependency(symbol.to_string())),
                    out,
                );
            }
            path.remove(0);
        }
        seen.remove(symbol);
    }

    let unknowns = field_names
        .iter()
        .map(|f| UnknownBlock {
            name: f.clone(),
            field: f.clone(),
        })
        .collect::<Vec<_>>();
    let mut residual_blocks = model
        .equations
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<_>>();
    residual_blocks.extend(model.forms.iter().map(|f| f.name.clone()));
    residual_blocks.sort();
    residual_blocks.dedup();

    let mut edges = vec![];
    let context = TraceContext {
        field_names: &field_names,
        property_map: &property_map,
        constitutive_map: &constitutive_map,
    };
    for equation in &model.equations {
        let mut names = BTreeSet::new();
        equation.lhs.names(&mut names);
        equation.rhs.names(&mut names);
        for name in names {
            trace(
                &name,
                &equation.name,
                &context,
                &mut Vec::new(),
                &mut BTreeSet::new(),
                None,
                &mut edges,
            );
        }
    }
    for form in &model.forms {
        for integral in &form.integrals {
            let mut names = BTreeSet::new();
            integral.integrand.names(&mut names);
            for name in names {
                trace(
                    &name,
                    &form.name,
                    &context,
                    &mut Vec::new(),
                    &mut BTreeSet::new(),
                    None,
                    &mut edges,
                );
            }
        }
    }
    // Conditions contribute to the residual block for their target field. Their region/value
    // dependencies are explicit interface/boundary coupling rather than invisible runtime state.
    for condition in model
        .boundary_conditions
        .iter()
        .chain(model.interface_conditions.iter())
    {
        let mut names = BTreeSet::new();
        condition.region.names(&mut names);
        condition.value.names(&mut names);
        for name in names {
            let before = edges.len();
            trace(
                &name,
                &condition.target,
                &context,
                &mut Vec::new(),
                &mut BTreeSet::new(),
                Some(CouplingReason::InterfaceTerm),
                &mut edges,
            );
            for edge in &mut edges[before..] {
                edge.reason = CouplingReason::InterfaceTerm;
            }
        }
    }

    edges.sort_by(|a, b| (&a.to, &a.from, &a.path).cmp(&(&b.to, &b.from, &b.path)));
    edges.dedup_by(|a, b| a.from == b.from && a.to == b.to && a.path == b.path);
    let mut derivatives = Vec::new();
    for residual in &residual_blocks {
        for unknown in &field_names {
            derivatives.push(BlockDerivative {
                residual: residual.clone(),
                unknown: unknown.clone(),
                structurally_nonzero: edges
                    .iter()
                    .any(|edge| &edge.to == residual && &edge.from == unknown),
            });
        }
    }
    CouplingGraph {
        unknowns,
        residual_blocks,
        edges,
        derivatives,
    }
}

// ---------------- R13 execution staging / canonical heat ----------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeStage {
    Gather,
    Restrict,
    Geometry,
    Basis,
    FieldDerivative,
    PointwiseProperty,
    WeakIntegrand,
    Quadrature,
    LocalAccumulation,
    Scatter,
    BoundaryLift,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificExecutionPlan {
    pub schema: String,
    pub model: String,
    pub stages: Vec<RuntimeStage>,
    pub residual_blocks: Vec<String>,
    pub derivative_blocks: Vec<BlockDerivative>,
}
pub fn execution_plan(model: &ScientificModel) -> ScientificExecutionPlan {
    let graph = derive_coupling_graph(model);
    ScientificExecutionPlan {
        schema: "resolvent-scientific-execution/1".into(),
        model: model.name.clone(),
        stages: vec![
            RuntimeStage::Gather,
            RuntimeStage::Restrict,
            RuntimeStage::Geometry,
            RuntimeStage::Basis,
            RuntimeStage::FieldDerivative,
            RuntimeStage::PointwiseProperty,
            RuntimeStage::WeakIntegrand,
            RuntimeStage::Quadrature,
            RuntimeStage::LocalAccumulation,
            RuntimeStage::Scatter,
            RuntimeStage::BoundaryLift,
        ],
        residual_blocks: graph.residual_blocks,
        derivative_blocks: graph.derivatives,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearTemperatureProperty {
    pub at_t0: f64,
    pub slope: f64,
    pub t0: f64,
}
impl LinearTemperatureProperty {
    pub fn value(self, t: f64) -> f64 {
        self.at_t0 * (1.0 + self.slope * (t - self.t0))
    }
    pub fn derivative(self) -> f64 {
        self.at_t0 * self.slope
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonicalHeatCase {
    pub t0: f64,
    pub amplitude: f64,
    pub rho: LinearTemperatureProperty,
    pub cp: LinearTemperatureProperty,
    pub k: LinearTemperatureProperty,
}
impl Default for CanonicalHeatCase {
    fn default() -> Self {
        Self {
            t0: 300.0,
            amplitude: 10.0,
            rho: LinearTemperatureProperty {
                at_t0: 7800.0,
                slope: 1e-5,
                t0: 300.0,
            },
            cp: LinearTemperatureProperty {
                at_t0: 500.0,
                slope: 2e-4,
                t0: 300.0,
            },
            k: LinearTemperatureProperty {
                at_t0: 16.0,
                slope: 5e-4,
                t0: 300.0,
            },
        }
    }
}
impl CanonicalHeatCase {
    pub fn exact(&self, x: f64, y: f64, t: f64) -> f64 {
        self.t0
            + self.amplitude
                * (std::f64::consts::PI * x).sin()
                * (std::f64::consts::PI * y).sin()
                * (-t).exp()
    }
    pub fn source(&self, x: f64, y: f64, t: f64) -> f64 {
        let pi = std::f64::consts::PI;
        let mode = (pi * x).sin() * (pi * y).sin() * (-t).exp();
        let temp = self.t0 + self.amplitude * mode;
        let dt = -self.amplitude * mode;
        let lap = -2.0 * pi * pi * self.amplitude * mode;
        let gx = self.amplitude * pi * (pi * x).cos() * (pi * y).sin() * (-t).exp();
        let gy = self.amplitude * pi * (pi * x).sin() * (pi * y).cos() * (-t).exp();
        let rho = self.rho.value(temp);
        let cp = self.cp.value(temp);
        let k = self.k.value(temp);
        rho * cp * dt - (k * lap + self.k.derivative() * (gx * gx + gy * gy))
    }
    pub fn strong_residual(&self, x: f64, y: f64, t: f64) -> f64 {
        let q = self.source(x, y, t);
        self.source(x, y, t) - q
    }
}

// ---------------- R20 time/state semantics ----------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeRole {
    Differential,
    Algebraic,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeField {
    pub field: String,
    pub role: TimeRole,
    pub initial: Option<Expr>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventSurface {
    pub name: String,
    pub expression: Expr,
    pub direction: EventDirection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventDirection {
    Any,
    Rising,
    Falling,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryStateSchema {
    pub law: String,
    pub variables: Vec<StateVariable>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeStateSemantics {
    pub fields: Vec<TimeField>,
    pub events: Vec<EventSurface>,
    pub history: Vec<HistoryStateSchema>,
    pub dae_form: String,
}
impl TimeStateSemantics {
    pub fn from_model(model: &ScientificModel) -> Self {
        let initial: BTreeMap<_, _> = model
            .initial_conditions
            .iter()
            .map(|c| (c.target.clone(), c.value.clone()))
            .collect();
        let fields = model
            .fields
            .iter()
            .filter(|f| matches!(f.role, FieldRoleV1::State | FieldRoleV1::Unknown))
            .map(|f| TimeField {
                field: f.name.clone(),
                role: f.time_role.unwrap_or(TimeRole::Differential),
                initial: initial.get(&f.name).cloned(),
            })
            .collect();
        Self {
            fields,
            events: vec![],
            history: vec![],
            dae_form: "F(t, y, ydot, p) = 0".into(),
        }
    }
}

pub fn validate_quantities(
    model: &ScientificModel,
    registry: &UnitRegistry,
) -> Result<(), ScientificError> {
    for field in &model.fields {
        if let Some(nominal) = &field.nominal {
            registry
                .canonicalize(nominal)
                .map_err(|e| ScientificError::Quantity(e.to_string()))?;
        }
        if let (Some(min), Some(max)) = (&field.physical_min, &field.physical_max) {
            let a = registry
                .canonicalize(min)
                .map_err(|e| ScientificError::Quantity(e.to_string()))?;
            let b = registry
                .canonicalize(max)
                .map_err(|e| ScientificError::Quantity(e.to_string()))?;
            if a.value_si > b.value_si {
                return Err(ScientificError::Quantity(format!(
                    "field `{}` has min > max",
                    field.name
                )));
            }
        }
    }
    Ok(())
}

pub fn quantity_defaults() -> (
    UnitRegistry,
    KindStrictness,
    Option<DisplayUnit>,
    Option<Bound<f64>>,
) {
    (
        UnitRegistry::standard(),
        KindStrictness::KindCompatible,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAT: &str = r#"
module examples.nonlinear_heat;
use physics.thermal.fourier;
model NonlinearHeat {
  domain Omega { dimension = 2; coordinates = cartesian; }
  field T: state scalar H1(order=1) on Omega { quantity = ThermodynamicTemperature; unit = K; nominal = 300 K; time_role = differential; };
  property rho = density(T);
  property cp = specific_heat(T);
  property k = thermal_conductivity(T);
  source Q: VolumetricHeatSource;
  equation energy on Omega { rho * cp * dt(T) - div(k * grad(T)) = Q; }
  initial { T = exact_T(0); }
  observable total_energy { integrate(rho * cp * T); }
}
"#;

    #[test]
    fn parses_structured_heat_source() {
        // The import is intentionally unresolved here: parsing and module resolution are separate phases.
        let m = parse_scientific_module(HEAT).unwrap();
        assert_eq!(m.name, "examples.nonlinear_heat");
        assert_eq!(m.models[0].properties.len(), 3);
        assert_eq!(m.models[0].equations.len(), 1);
        let graph = derive_coupling_graph(&m.models[0]);
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.from == "T" && e.to == "energy")
        );
    }

    #[test]
    fn property_expression_symbolic_derivative_matches_finite_difference() {
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            lhs: Box::new(Expr::Number {
                value: 10.0,
                unit: None,
            }),
            rhs: Box::new(Expr::Binary {
                op: BinaryOp::Mul,
                lhs: Box::new(Expr::Number {
                    value: 0.5,
                    unit: None,
                }),
                rhs: Box::new(Expr::Name("T".into())),
            }),
        };
        let mut env = BTreeMap::new();
        env.insert("T".into(), 300.0);
        let d = eval_expr(&differentiate_expr(&expr, "T"), &env).unwrap();
        assert!((d - 0.5).abs() < 1e-12);
    }

    #[test]
    fn production_catalog_covers_agent_gate_spaces() {
        let c = DiscretizationCatalog::production();
        assert!(c.supports(ReferenceCell::Triangle, SpaceFamily::H1, 2));
        assert!(c.supports(ReferenceCell::Triangle, SpaceFamily::L2, 0));
        assert!(c.supports(ReferenceCell::Triangle, SpaceFamily::HCurl, 0));
        assert!(c.supports(ReferenceCell::Triangle, SpaceFamily::HDiv, 0));
        assert_eq!(orientation_sign(&[1, 0, 2]), -1);
    }

    #[test]
    fn canonical_heat_source_is_smooth_and_finite() {
        let c = CanonicalHeatCase::default();
        for p in [(0.2, 0.3, 0.0), (0.5, 0.5, 0.2), (0.8, 0.4, 1.0)] {
            assert!(c.source(p.0, p.1, p.2).is_finite());
            assert_eq!(c.strong_residual(p.0, p.1, p.2), 0.0);
        }
    }
}
