use crate::id::{ExprId, FieldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueShape {
    Scalar,
    Vector { dim: u8 },
    Tensor { rows: u8, cols: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Continuity {
    H1,
    HCurl,
    HDiv,
    L2,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSpace {
    pub family: String,
    pub order: u8,
    pub continuity: Continuity,
    pub value_shape: ValueShape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldRole {
    Unknown,
    Coefficient,
    Trial,
    Test,
    Derived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub id: FieldId,
    pub name: String,
    pub role: FieldRole,
    pub space: FunctionSpace,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

/// Continuum/variational expression dialect. Scalar coefficients reference the generic
/// expression store by `ExprId`; they are not copied into a second CAS AST.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormExpr {
    Scalar(ExprId),
    Field(FieldId),
    Neg(Box<FormExpr>),
    Add(Vec<FormExpr>),
    Product(Vec<FormExpr>),
    Gradient(Box<FormExpr>),
    Divergence(Box<FormExpr>),
    Curl(Box<FormExpr>),
    TimeDerivative(Box<FormExpr>),
    Trace(Box<FormExpr>),
    Inner {
        left: Box<FormExpr>,
        right: Box<FormExpr>,
    },
    Contract {
        left: Box<FormExpr>,
        right: Box<FormExpr>,
    },
    Custom {
        operator: String,
        args: Vec<FormExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    Volume { domain: String },
    Boundary { boundary: String },
    Interface { interface: String },
    Point { set: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integral {
    pub integrand: FormExpr,
    pub measure: Measure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormProgram {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    #[serde(default)]
    pub residual_terms: Vec<Integral>,
    #[serde(default)]
    pub boundary_terms: Vec<Integral>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}
