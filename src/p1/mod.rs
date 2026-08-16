//! Deterministic P1 reference discretization for the first continuum vertical.
//!
//! This module is a mathematical reference lowering, not a linear solver and not an
//! execution backend. It gives Resolvent a portable implementation of
//! `FormProgram -> P1 discretization -> assembled operator -> semi-discrete evolution`
//! that downstream implementations can differential-test.

mod assembly;
mod compiler;
mod matrix;
mod mesh;

pub use assembly::{
    EvolutionAssembly, EvolutionClass, MassAssembly, MassInput, MassLumping,
    ScalarEllipticAssembly, ScalarEllipticInput, assemble_mass, assemble_scalar_elliptic,
};
pub use compiler::{P1DiscretizationRequest, P1LoweringResult, lower_p1};
pub use matrix::CsrMatrix;
pub use mesh::{
    BoundaryEdge, BoundaryFlux, DirichletBoundary, DofMap, P1Mesh, PiecewiseConstant,
    PiecewiseSource, Point2, Triangle,
};

use crate::refinement::RefinementError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum P1Error {
    #[error("{owner} references vertex {vertex}, but mesh has {vertex_count} vertices")]
    VertexOutOfRange {
        owner: String,
        vertex: usize,
        vertex_count: usize,
    },
    #[error("degenerate triangle at cell {cell} (signed area {signed_area:e})")]
    DegenerateTriangle { cell: usize, signed_area: f64 },
    #[error("matrix index ({row}, {col}) outside {nrows}x{ncols}")]
    MatrixIndexOutOfRange {
        row: usize,
        col: usize,
        nrows: usize,
        ncols: usize,
    },
    #[error("matrix shapes do not match")]
    MatrixShapeMismatch,
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("form id {0} is not present in the context")]
    MissingForm(u32),
    #[error("field id {0} is not present in the form")]
    MissingField(u32),
    #[error("field `{0}` is not scalar Lagrange P1 H1")]
    UnsupportedFieldSpace(String),
    #[error("field `{field}` has unsupported role; expected {expected}")]
    UnsupportedFieldRole { field: String, expected: String },
    #[error("unsupported P1 form: {0}")]
    UnsupportedForm(String),
    #[error(transparent)]
    Refinement(#[from] RefinementError),
}

#[cfg(test)]
mod definiteness_tests;
#[cfg(test)]
mod evolution_class_tests;
#[cfg(test)]
mod tests;
