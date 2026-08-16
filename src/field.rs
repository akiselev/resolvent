use crate::id::FieldId;
use crate::units::Dimension;
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
#[serde(rename_all = "snake_case")]
pub enum ElementFamily {
    Lagrange,
    Nedelec,
    RaviartThomas,
    BrezziDouglasMarini,
    DiscontinuousGalerkin,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSpace {
    pub family: ElementFamily,
    pub order: u8,
    pub continuity: Continuity,
    pub value_shape: ValueShape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl FunctionSpace {
    pub fn h1_lagrange(order: u8, domain: impl Into<String>) -> Self {
        Self::h1_lagrange_shaped(order, ValueShape::Scalar, domain)
    }

    pub fn h1_lagrange_vector(order: u8, dim: u8, domain: impl Into<String>) -> Self {
        Self::h1_lagrange_shaped(order, ValueShape::Vector { dim }, domain)
    }

    pub fn h1_lagrange_shaped(
        order: u8,
        value_shape: ValueShape,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            family: ElementFamily::Lagrange,
            order,
            continuity: Continuity::H1,
            value_shape,
            domain: Some(domain.into()),
        }
    }

    pub fn hcurl_nedelec(order: u8, geometric_dim: u8, domain: impl Into<String>) -> Self {
        Self {
            family: ElementFamily::Nedelec,
            order,
            continuity: Continuity::HCurl,
            value_shape: ValueShape::Vector { dim: geometric_dim },
            domain: Some(domain.into()),
        }
    }

    pub fn hdiv_raviart_thomas(order: u8, geometric_dim: u8, domain: impl Into<String>) -> Self {
        Self {
            family: ElementFamily::RaviartThomas,
            order,
            continuity: Continuity::HDiv,
            value_shape: ValueShape::Vector { dim: geometric_dim },
            domain: Some(domain.into()),
        }
    }

    pub fn l2_discontinuous(order: u8, value_shape: ValueShape, domain: impl Into<String>) -> Self {
        Self {
            family: ElementFamily::DiscontinuousGalerkin,
            order,
            continuity: Continuity::L2,
            value_shape,
            domain: Some(domain.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldRole {
    Unknown,
    State,
    Coefficient,
    Parameter,
    Trial,
    Test,
    Derived,
    Observable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub id: FieldId,
    pub name: String,
    pub role: FieldRole,
    pub space: FunctionSpace,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<Dimension>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRef {
    pub name: String,
    pub topological_dimension: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometric_dimension: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryRef {
    pub name: String,
    pub domain: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceRef {
    pub name: String,
    pub left_domain: String,
    pub right_domain: String,
}
