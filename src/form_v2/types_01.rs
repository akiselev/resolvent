//! FC0-FC1 variational-form compatibility boundary.
//!
//! V2 is deliberately separate from the legacy `FormProgram` and scalar weak-term
//! executor. It records form arguments independently from scientific fields, makes
//! sides/arity/scalar semantics explicit, carries truthful derivative/operator claims,
//! and retains the V1 scalar program only as a named differential oracle.

use crate::id::Digest;
use crate::scientific::{
    Expr, FieldRoleV1, ScientificModel, SpaceFamily, ValueShapeV1,
};
use crate::scientific_weak::{
    WeakOperatorProgram, WeakTerm, lower_scalar_h1_model,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use thiserror::Error;

pub const VARIATIONAL_FORM_V2_SCHEMA: &str = "resolvent-variational-form/2";
pub const VARIATIONAL_ARTIFACT_V2_SCHEMA: &str = "resolvent-stage-artifact/2";
pub const FORMULATION_RECEIPT_V2_SCHEMA: &str = "resolvent-formulation-receipt/2";

macro_rules! string_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(ArgumentIdV2);
string_id!(ScientificFieldIdV2);
string_id!(SpaceRequirementIdV2);
string_id!(ConstantIdV2);
string_id!(FormulationDerivationIdV2);
string_id!(FrameIdV2);
string_id!(IndexSetIdV2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarKindV2 {
    Real32,
    Real64,
    Complex32,
    Complex64,
}

impl ScalarKindV2 {
    pub const fn is_complex(self) -> bool {
        matches!(self, Self::Complex32 | Self::Complex64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceV2 {
    Contravariant,
    Covariant,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AxisKindV2 {
    Spatial {
        frame: FrameIdV2,
        variance: VarianceV2,
    },
    Species {
        index_set: IndexSetIdV2,
    },
    SlipSystem {
        index_set: IndexSetIdV2,
    },
    NetworkNode {
        index_set: IndexSetIdV2,
    },
    NetworkBranch {
        index_set: IndexSetIdV2,
    },
    MaterialComponent {
        index_set: IndexSetIdV2,
    },
    Algebraic {
        extent: u32,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum QuantityTypeV2 {
    #[default]
    Unspecified,
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TensorTypeV2 {
    pub scalar: ScalarKindV2,
    #[serde(default)]
    pub axes: Vec<AxisKindV2>,
    #[serde(default)]
    pub quantity: QuantityTypeV2,
}

impl TensorTypeV2 {
    pub fn scalar(scalar: ScalarKindV2) -> Self {
        Self {
            scalar,
            axes: Vec::new(),
            quantity: QuantityTypeV2::Unspecified,
        }
    }

    pub fn spatial_vector(
        scalar: ScalarKindV2,
        frame: FrameIdV2,
        variance: VarianceV2,
    ) -> Self {
        Self {
            scalar,
            axes: vec![AxisKindV2::Spatial { frame, variance }],
            quantity: QuantityTypeV2::Unspecified,
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.axes.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameV2 {
    pub id: FrameIdV2,
    pub dimension: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSetV2 {
    pub id: IndexSetIdV2,
    pub extent: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SobolevSpaceV2 {
    H1,
    L2,
    HCurl,
    HDiv,
    Dg,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceRequirementV2 {
    pub id: SpaceRequirementIdV2,
    pub domain: String,
    pub spatial_frame: FrameIdV2,
    pub sobolev: SobolevSpaceV2,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormArgumentV2 {
    pub id: ArgumentIdV2,
    pub name: String,
    pub number: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<u16>,
    pub space: SpaceRequirementIdV2,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeLevelV2 {
    #[default]
    Current,
    Previous(u16),
    Rate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormCoefficientV2 {
    pub field: ScientificFieldIdV2,
    pub name: String,
    pub space: SpaceRequirementIdV2,
    pub time_level: TimeLevelV2,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantRefV2 {
    pub id: ConstantIdV2,
    pub name: String,
    pub value_type: TensorTypeV2,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum RegionSelectorV2 {
    #[default]
    All,
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MeasureV2 {
    Cell {
        domain: String,
        region: RegionSelectorV2,
    },
    ExteriorFacet {
        domain: String,
        boundary: String,
    },
    InteriorFacet {
        domain: String,
        region: RegionSelectorV2,
    },
    Interface {
        interface: String,
        left: String,
        right: String,
    },
    Ridge {
        domain: String,
        region: RegionSelectorV2,
    },
    Vertex {
        domain: String,
        region: RegionSelectorV2,
    },
}

impl MeasureV2 {
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Cell { .. } => "cell",
            Self::ExteriorFacet { .. } => "exterior_facet",
            Self::InteriorFacet { .. } => "interior_facet",
            Self::Interface { .. } => "interface",
            Self::Ridge { .. } => "ridge",
            Self::Vertex { .. } => "vertex",
        }
    }

    const fn requires_explicit_side(&self) -> bool {
        matches!(self, Self::InteriorFacet { .. } | Self::Interface { .. })
    }

    const fn allows_side(&self, side: TraceSideV2) -> bool {
        matches!(
            (self, side),
            (
                Self::InteriorFacet { .. },
                TraceSideV2::Plus | TraceSideV2::Minus
            ) | (
                Self::Interface { .. },
                TraceSideV2::Left | TraceSideV2::Right
            ) | (Self::ExteriorFacet { .. }, TraceSideV2::Exterior)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSideV2 {
    Plus,
    Minus,
    Left,
    Right,
    Exterior,
}
