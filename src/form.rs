use crate::id::{ExprId, FieldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use crate::field::{Continuity, ElementFamily, Field, FieldRole, FunctionSpace, ValueShape};

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
    Normal,
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
pub struct EssentialBoundary {
    pub field: FieldId,
    pub boundary: String,
    pub value: ExprId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaturalBoundary {
    pub field: FieldId,
    pub boundary: String,
    pub flux: ExprId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RobinBoundary {
    pub field: FieldId,
    pub boundary: String,
    pub coefficient: ExprId,
    pub ambient: ExprId,
}

/// A method-neutral continuum form. Boundary conditions are semantic data and are compiled
/// into restriction/elimination or natural integral terms; solver policy is deliberately
/// absent.
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
    pub essential_boundaries: Vec<EssentialBoundary>,
    #[serde(default)]
    pub natural_boundaries: Vec<NaturalBoundary>,
    #[serde(default)]
    pub robin_boundaries: Vec<RobinBoundary>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}
