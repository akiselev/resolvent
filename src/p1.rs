//! Deterministic P1 reference discretization for the first continuum vertical.
//!
//! This module is deliberately a *reference mathematical lowering*, not a linear solver and
//! not an execution backend. It gives Resolvent one concrete, portable implementation of the
//! path
//!
//! `FormProgram -> P1 discretization -> assembled operator -> semi-discrete evolution`
//!
//! so downstream implementations can be differential-tested against a common artifact. The
//! first vertical is scalar H1 diffusion on 2-D triangles with piecewise-constant coefficients,
//! constant Dirichlet data, volumetric source, natural Neumann flux, and optional consistent or
//! lumped capacity mass.

use crate::context::Context;
use crate::discrete::{
    BasisEvaluation, DiscreteOp, DiscreteProgram, RestrictionDirection,
};
use crate::form::{Continuity, Field, FieldRole, FormExpr, FormProgram, ValueShape};
use crate::id::{DiscreteProgramId, FieldId, FormId, OperatorId, RefinementId};
use crate::operator::{
    DerivativeCapability, OperatorBlock, OperatorBlockKind, OperatorProgram, OperatorProperty,
    SparsityContract,
};
use crate::refinement::{
    ArtifactKind, RefinementError, RefinementProvenance, RefinementRecord, RefinementRelation,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use thiserror::Error;

const DEGENERATE_AREA: f64 = 1.0e-300;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub region: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryEdge {
    pub vertices: [usize; 2],
    pub tag: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P1Mesh {
    pub vertices: Vec<Point2>,
    pub cells: Vec<Triangle>,
    #[serde(default)]
    pub boundary_edges: Vec<BoundaryEdge>,
}

impl P1Mesh {
    pub fn validate(&self) -> Result<(), P1Error> {
        let n = self.vertices.len();
        for (cell, triangle) in self.cells.iter().enumerate() {
            for &vertex in &triangle.vertices {
                if vertex >= n {
                    return Err(P1Error::VertexOutOfRange {
                        owner: format!("cell {cell}"),
                        vertex,
                        vertex_count: n,
                    });
                }
            }
        }
        for (edge, boundary) in self.boundary_edges.iter().enumerate() {
            for &vertex in &boundary.vertices {
                if vertex >= n {
                    return Err(P1Error::VertexOutOfRange {
                        owner: format!("boundary edge {edge}"),
                        vertex,
                        vertex_count: n,
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PiecewiseConstant {
    #[serde(default)]
    pub per_region: BTreeMap<u32, f64>,
    pub background: f64,
}

impl PiecewiseConstant {
    #[must_use]
    pub fn uniform(value: f64) -> Self {
        Self {
            per_region: BTreeMap::new(),
            background: value,
        }
    }

    #[must_use]
    pub fn value(&self, region: u32) -> f64 {
        self.per_region
            .get(&region)
            .copied()
            .unwrap_or(self.background)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PiecewiseSource {
    #[serde(default)]
    pub per_region: BTreeMap<u32, f64>,
}

impl PiecewiseSource {
    #[must_use]
    pub fn value(&self, region: u32) -> f64 {
        self.per_region.get(&region).copied().unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BoundaryFlux {
    #[serde(default)]
    pub per_tag: BTreeMap<u32, f64>,
}

impl BoundaryFlux {
    #[must_use]
    pub fn value(&self, tag: u32) -> f64 {
        self.per_tag.get(&tag).copied().unwrap_or(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirichletBoundary {
    pub tag: u32,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MassLumping {
    #[default]
    Consistent,
    Lumped,
}

/// Vertex-to-free-unknown partition. Dirichlet vertices retain their prescribed value while
/// free vertices receive dense deterministic indices in vertex order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofMap {
    free_of_vertex: Vec<Option<u32>>,
    prescribed: Vec<Option<f64>>,
    constrained: Vec<(usize, f64)>,
    n_free: usize,
}

impl DofMap {
    pub fn from_dirichlet(
        mesh: &P1Mesh,
        specs: &[DirichletBoundary],
    ) -> Result<Self, P1Error> {
        mesh.validate()?;

        // Deliberately mirrors the incumbent Residua semantics: later declarations for the
        // same boundary tag replace earlier ones, then boundary facets apply in mesh order.
        let mut by_tag = BTreeMap::new();
        for spec in specs {
            by_tag.insert(spec.tag, spec.value);
        }

        let mut prescribed = vec![None; mesh.vertices.len()];
        for edge in &mesh.boundary_edges {
            if let Some(&value) = by_tag.get(&edge.tag) {
                prescribed[edge.vertices[0]] = Some(value);
                prescribed[edge.vertices[1]] = Some(value);
            }
        }

        let mut free_of_vertex = vec![None; mesh.vertices.len()];
        let mut constrained = Vec::new();
        let mut next = 0_u32;
        for (vertex, value) in prescribed.iter().copied().enumerate() {
            match value {
                Some(value) => constrained.push((vertex, value)),
                None => {
                    free_of_vertex[vertex] = Some(next);
                    next += 1;
                }
            }
        }

        Ok(Self {
            free_of_vertex,
            prescribed,
            constrained,
            n_free: next as usize,
        })
    }

    #[must_use]
    pub const fn n_free(&self) -> usize {
        self.n_free
    }

    #[must_use]
    pub fn dof(&self, vertex: usize) -> Option<u32> {
        self.free_of_vertex.get(vertex).copied().flatten()
    }

    #[must_use]
    pub fn value_of(&self, vertex: usize) -> f64 {
        self.prescribed
            .get(vertex)
            .copied()
            .flatten()
            .unwrap_or(0.0)
    }

    #[must_use]
    pub fn constrained(&self) -> &[(usize, f64)] {
        &self.constrained
    }

    #[must_use]
    pub fn prescribed(&self) -> &[Option<f64>] {
        &self.prescribed
    }
}

/// Portable deterministic CSR matrix used by the reference lowering. Solver policy remains
/// outside Resolvent; this type exists so a discretization has a reproducible numerical
/// meaning that other backends can compare against.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CsrMatrix {
    pub nrows: usize,
    pub ncols: usize,
    pub row_offsets: Vec<usize>,
    pub column_indices: Vec<usize>,
    pub values: Vec<f64>,
}

impl CsrMatrix {
    fn from_triplets(
        nrows: usize,
        ncols: usize,
        mut triplets: Vec<(usize, usize, f64)>,
    ) -> Result<Self, P1Error> {
        for &(row, col, _) in &triplets {
            if row >= nrows || col >= ncols {
                return Err(P1Error::MatrixIndexOutOfRange {
                    row,
                    col,
                    nrows,
                    ncols,
                });
            }
        }

        // `sort_by` is stable. Duplicate contributions to one matrix entry therefore sum
        // in assembly insertion order, making floating-point accumulation deterministic.
        triplets.sort_by(|a, b| match a.0.cmp(&b.0) {
            Ordering::Equal => a.1.cmp(&b.1),
            other => other,
        });

        let mut row_counts = vec![0_usize; nrows];
        let mut column_indices = Vec::new();
        let mut values = Vec::new();
        let mut cursor = 0;
        while cursor < triplets.len() {
            let (row, col, _) = triplets[cursor];
            let mut sum = 0.0;
            while cursor < triplets.len()
                && triplets[cursor].0 == row
                && triplets[cursor].1 == col
            {
                sum += triplets[cursor].2;
                cursor += 1;
            }
            if sum != 0.0 {
                row_counts[row] += 1;
                column_indices.push(col);
                values.push(sum);
            }
        }

        let mut row_offsets = vec![0_usize; nrows + 1];
        for (row, count) in row_counts.into_iter().enumerate() {
            row_offsets[row + 1] = row_offsets[row] + count;
        }

        Ok(Self {
            nrows,
            ncols,
            row_offsets,
            column_indices,
            values,
        })
    }

    #[must_use]
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    pub fn apply(&self, x: &[f64]) -> Result<Vec<f64>, P1Error> {
        if x.len() != self.ncols {
            return Err(P1Error::DimensionMismatch {
                expected: self.ncols,
                got: x.len(),
            });
        }
        let mut out = vec![0.0; self.nrows];
        for (row, out_value) in out.iter_mut().enumerate() {
            let begin = self.row_offsets[row];
            let end = self.row_offsets[row + 1];
            let mut value = 0.0;
            for entry in begin..end {
                value += self.values[entry] * x[self.column_indices[entry]];
            }
            *out_value = value;
        }
        Ok(out)
    }

    #[must_use]
    pub fn to_dense(&self) -> Vec<Vec<f64>> {
        let mut dense = vec![vec![0.0; self.ncols]; self.nrows];
        for (row, dense_row) in dense.iter_mut().enumerate() {
            for entry in self.row_offsets[row]..self.row_offsets[row + 1] {
                dense_row[self.column_indices[entry]] = self.values[entry];
            }
        }
        dense
    }

    pub fn scaled_sum(terms: &[(f64, &Self)]) -> Result<Self, P1Error> {
        let Some((_, first)) = terms.first() else {
            return Ok(Self {
                nrows: 0,
                ncols: 0,
                row_offsets: vec![0],
                column_indices: Vec::new(),
                values: Vec::new(),
            });
        };
        let nrows = first.nrows;
        let ncols = first.ncols;
        if terms
            .iter()
            .any(|(_, matrix)| matrix.nrows != nrows || matrix.ncols != ncols)
        {
            return Err(P1Error::MatrixShapeMismatch);
        }

        let capacity = terms.iter().map(|(_, matrix)| matrix.nnz()).sum();
        let mut triplets = Vec::with_capacity(capacity);
        for &(scale, matrix) in terms {
            for row in 0..matrix.nrows {
                for entry in matrix.row_offsets[row]..matrix.row_offsets[row + 1] {
                    triplets.push((
                        row,
                        matrix.column_indices[entry],
                        scale * matrix.values[entry],
                    ));
                }
            }
        }
        Self::from_triplets(nrows, ncols, triplets)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScalarEllipticInput {
    pub diffusion: PiecewiseConstant,
    #[serde(default)]
    pub source: PiecewiseSource,
    #[serde(default)]
    pub neumann: BoundaryFlux,
    #[serde(default)]
    pub dirichlet: Vec<DirichletBoundary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarEllipticAssembly {
    pub dof_map: DofMap,
    pub stiffness_free: CsrMatrix,
    pub stiffness_full: CsrMatrix,
    /// Complete free-space RHS: Dirichlet lift + volume source + Neumann flux.
    pub rhs: Vec<f64>,
    pub dirichlet_lift: Vec<f64>,
    pub source_full: Vec<f64>,
    pub source_free: Vec<f64>,
    pub neumann_full: Vec<f64>,
    pub neumann_free: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MassInput {
    pub capacity: PiecewiseConstant,
    #[serde(default)]
    pub lumping: MassLumping,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MassAssembly {
    pub mass_free: CsrMatrix,
    pub mass_full: CsrMatrix,
    pub lumping: MassLumping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionClass {
    Ode,
    Dae { index: u8 },
}

/// Semi-discrete linear evolution `M·u_dot + K·u = f`. This is a mathematical operator;
/// time-step selection, nonlinear globalization and linear-solve policy belong downstream.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvolutionAssembly {
    pub stiffness: CsrMatrix,
    pub mass: CsrMatrix,
    pub rhs: Vec<f64>,
    pub class: EvolutionClass,
}

impl EvolutionAssembly {
    pub fn residual(&self, state: &[f64], rate: &[f64]) -> Result<Vec<f64>, P1Error> {
        if state.len() != self.stiffness.ncols {
            return Err(P1Error::DimensionMismatch {
                expected: self.stiffness.ncols,
                got: state.len(),
            });
        }
        if rate.len() != self.mass.ncols {
            return Err(P1Error::DimensionMismatch {
                expected: self.mass.ncols,
                got: rate.len(),
            });
        }
        if self.rhs.len() != self.stiffness.nrows {
            return Err(P1Error::DimensionMismatch {
                expected: self.stiffness.nrows,
                got: self.rhs.len(),
            });
        }

        let kx = self.stiffness.apply(state)?;
        let mx = self.mass.apply(rate)?;
        Ok(mx
            .into_iter()
            .zip(kx)
            .zip(&self.rhs)
            .map(|((mass, stiffness), rhs)| mass + stiffness - rhs)
            .collect())
    }

    pub fn iteration_matrix(
        &self,
        mass_coefficient: f64,
        stiffness_coefficient: f64,
    ) -> Result<CsrMatrix, P1Error> {
        CsrMatrix::scaled_sum(&[
            (mass_coefficient, &self.mass),
            (stiffness_coefficient, &self.stiffness),
        ])
    }

    pub fn static_jvp(&self, direction: &[f64]) -> Result<Vec<f64>, P1Error> {
        self.stiffness.apply(direction)
    }

    pub fn static_vjp(&self, direction: &[f64]) -> Result<Vec<f64>, P1Error> {
        // The first vertical is symmetric diffusion. Keeping a separate method is intentional:
        // later nonsymmetric forms must not silently inherit this implementation.
        self.stiffness.apply(direction)
    }

    pub fn mass_jvp(&self, direction: &[f64]) -> Result<Vec<f64>, P1Error> {
        self.mass.apply(direction)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P1DiscretizationRequest {
    pub form: FormId,
    pub unknown: FieldId,
    pub test: FieldId,
    pub mesh: P1Mesh,
    pub elliptic: ScalarEllipticInput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<MassInput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct P1LoweringResult {
    pub form: FormId,
    pub stiffness_program: DiscreteProgramId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_program: Option<DiscreteProgramId>,
    pub operator: OperatorId,
    pub refinement: RefinementId,
    pub elliptic: ScalarEllipticAssembly,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<MassAssembly>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution: Option<EvolutionAssembly>,
}

/// Compile the first concrete continuum vertical. The source form must contain scalar P1 H1
/// unknown/test fields and at least one volume term coupling gradients of those fields.
/// Numerical coefficient/source/boundary data is the discretization binding for that semantic
/// form; it is carried in the returned artifact and covered by the target refinement digest.
pub fn lower_p1(
    context: &mut Context,
    request: &P1DiscretizationRequest,
) -> Result<P1LoweringResult, P1Error> {
    let form = context
        .form(request.form)
        .cloned()
        .ok_or(P1Error::MissingForm(request.form.0))?;
    validate_form(&form, request.unknown, request.test)?;
    request.mesh.validate()?;

    let source_ref = context.rooted_artifact_ref(ArtifactKind::Form, &form)?;
    let elliptic = assemble_scalar_elliptic(&request.mesh, &request.elliptic)?;

    let stiffness_program = context.insert_discrete(stiffness_program(request));
    let mass = request
        .mass
        .as_ref()
        .map(|mass| assemble_mass(&request.mesh, &elliptic.dof_map, mass))
        .transpose()?;
    let mass_program = if mass.is_some() {
        Some(context.insert_discrete(mass_program(request)))
    } else {
        None
    };

    let evolution = mass.as_ref().map(|mass| EvolutionAssembly {
        stiffness: elliptic.stiffness_free.clone(),
        mass: mass.mass_free.clone(),
        rhs: elliptic.rhs.clone(),
        class: EvolutionClass::Ode,
    });

    let operator_program = operator_program(request, stiffness_program, mass_program, &elliptic);
    let operator = context.insert_operator(operator_program.clone());

    #[derive(Serialize)]
    struct NumericalRoot<'a> {
        operator: &'a OperatorProgram,
        elliptic: &'a ScalarEllipticAssembly,
        mass: &'a Option<MassAssembly>,
        evolution: &'a Option<EvolutionAssembly>,
    }
    let target_ref = context.rooted_artifact_ref(
        ArtifactKind::OperatorProgram,
        &NumericalRoot {
            operator: &operator_program,
            elliptic: &elliptic,
            mass: &mass,
            evolution: &evolution,
        },
    )?;

    let mut refinement = RefinementRecord::new(
        source_ref,
        target_ref,
        RefinementRelation::Discretization {
            scheme: "conforming continuous Galerkin P1 triangle FEM".into(),
            declared_order: Some(1),
        },
    );
    refinement.assumptions = vec![
        "piecewise-affine scalar H1 field on conforming 2-D triangles".into(),
        "piecewise-constant cell coefficient/capacity data".into(),
        "Dirichlet values are time-constant in the transient vertical".into(),
    ];
    refinement.provenance = RefinementProvenance {
        producer: Some("resolvent::p1".into()),
        producer_version: Some(env!("CARGO_PKG_VERSION").into()),
        parameters: BTreeMap::from([
            ("quadrature".into(), "exact P1 analytic element integrals".into()),
            ("assembly".into(), "stable-order deterministic CSR".into()),
        ]),
        ..RefinementProvenance::default()
    };
    let refinement = context.record_refinement(refinement);

    Ok(P1LoweringResult {
        form: request.form,
        stiffness_program,
        mass_program,
        operator,
        refinement,
        elliptic,
        mass,
        evolution,
    })
}

pub fn assemble_scalar_elliptic(
    mesh: &P1Mesh,
    input: &ScalarEllipticInput,
) -> Result<ScalarEllipticAssembly, P1Error> {
    mesh.validate()?;
    let dof_map = DofMap::from_dirichlet(mesh, &input.dirichlet)?;
    let n_free = dof_map.n_free();
    let n_vertices = mesh.vertices.len();

    let mut full_triplets = Vec::with_capacity(mesh.cells.len() * 9);
    let mut free_triplets = Vec::with_capacity(mesh.cells.len() * 9);
    let mut dirichlet_lift = vec![0.0; n_free];

    for (cell_index, triangle) in mesh.cells.iter().enumerate() {
        let points = triangle.vertices.map(|vertex| mesh.vertices[vertex]);
        let (area, gradients) = p1_shape_gradients(cell_index, &points)?;
        let coefficient = input.diffusion.value(triangle.region);

        for (i, (&vertex_i, gradient_i)) in triangle
            .vertices
            .iter()
            .zip(gradients.iter())
            .enumerate()
        {
            for (&vertex_j, gradient_j) in triangle.vertices.iter().zip(gradients.iter()) {
                let value = coefficient
                    * area
                    * (gradient_i[0] * gradient_j[0] + gradient_i[1] * gradient_j[1]);
                full_triplets.push((vertex_i, vertex_j, value));
                match (dof_map.dof(vertex_i), dof_map.dof(vertex_j)) {
                    (Some(row), Some(col)) => {
                        free_triplets.push((row as usize, col as usize, value));
                    }
                    (Some(row), None) => {
                        dirichlet_lift[row as usize] -= value * dof_map.value_of(vertex_j);
                    }
                    _ => {}
                }
            }
            let _ = i;
        }
    }

    let stiffness_full = CsrMatrix::from_triplets(n_vertices, n_vertices, full_triplets)?;
    let stiffness_free = CsrMatrix::from_triplets(n_free, n_free, free_triplets)?;
    let source_full = assemble_source(mesh, &input.source)?;
    let source_free = condense_load(&source_full, &dof_map);
    let neumann_full = assemble_neumann(mesh, &input.neumann);
    let neumann_free = condense_load(&neumann_full, &dof_map);
    let rhs = dirichlet_lift
        .iter()
        .zip(&source_free)
        .zip(&neumann_free)
        .map(|((lift, source), neumann)| lift + source + neumann)
        .collect();

    Ok(ScalarEllipticAssembly {
        dof_map,
        stiffness_free,
        stiffness_full,
        rhs,
        dirichlet_lift,
        source_full,
        source_free,
        neumann_full,
        neumann_free,
    })
}

#[allow(clippy::needless_range_loop)]
pub fn assemble_mass(
    mesh: &P1Mesh,
    dof_map: &DofMap,
    input: &MassInput,
) -> Result<MassAssembly, P1Error> {
    mesh.validate()?;
    let n_free = dof_map.n_free();
    let n_vertices = mesh.vertices.len();
    let mut full_triplets = Vec::with_capacity(mesh.cells.len() * 9);
    let mut free_triplets = Vec::with_capacity(mesh.cells.len() * 9);

    for (cell_index, triangle) in mesh.cells.iter().enumerate() {
        let points = triangle.vertices.map(|vertex| mesh.vertices[vertex]);
        let (area, _) = p1_shape_gradients(cell_index, &points)?;
        let scale = input.capacity.value(triangle.region) * area / 12.0;
        let mut local = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                local[i][j] = scale * if i == j { 2.0 } else { 1.0 };
            }
        }
        if matches!(input.lumping, MassLumping::Lumped) {
            for i in 0..3 {
                let row_sum: f64 = local[i].iter().sum();
                for j in 0..3 {
                    local[i][j] = if i == j { row_sum } else { 0.0 };
                }
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let value = local[i][j];
                let vertex_i = triangle.vertices[i];
                let vertex_j = triangle.vertices[j];
                full_triplets.push((vertex_i, vertex_j, value));
                if let (Some(row), Some(col)) = (dof_map.dof(vertex_i), dof_map.dof(vertex_j)) {
                    free_triplets.push((row as usize, col as usize, value));
                }
            }
        }
    }

    Ok(MassAssembly {
        mass_free: CsrMatrix::from_triplets(n_free, n_free, free_triplets)?,
        mass_full: CsrMatrix::from_triplets(n_vertices, n_vertices, full_triplets)?,
        lumping: input.lumping,
    })
}

fn assemble_source(mesh: &P1Mesh, source: &PiecewiseSource) -> Result<Vec<f64>, P1Error> {
    let mut load = vec![0.0; mesh.vertices.len()];
    for (cell_index, triangle) in mesh.cells.iter().enumerate() {
        let value = source.value(triangle.region);
        if value == 0.0 {
            continue;
        }
        let points = triangle.vertices.map(|vertex| mesh.vertices[vertex]);
        let (area, _) = p1_shape_gradients(cell_index, &points)?;
        let nodal = value * area / 3.0;
        for &vertex in &triangle.vertices {
            load[vertex] += nodal;
        }
    }
    Ok(load)
}

fn assemble_neumann(mesh: &P1Mesh, flux: &BoundaryFlux) -> Vec<f64> {
    let mut load = vec![0.0; mesh.vertices.len()];
    for edge in &mesh.boundary_edges {
        let value = flux.value(edge.tag);
        if value == 0.0 {
            continue;
        }
        let a = mesh.vertices[edge.vertices[0]];
        let b = mesh.vertices[edge.vertices[1]];
        let length = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        let nodal = 0.5 * value * length;
        load[edge.vertices[0]] += nodal;
        load[edge.vertices[1]] += nodal;
    }
    load
}

fn condense_load(full: &[f64], dof_map: &DofMap) -> Vec<f64> {
    let mut free = vec![0.0; dof_map.n_free()];
    for (vertex, &value) in full.iter().enumerate() {
        if value != 0.0
            && let Some(dof) = dof_map.dof(vertex)
        {
            free[dof as usize] += value;
        }
    }
    free
}

fn p1_shape_gradients(
    cell: usize,
    points: &[Point2; 3],
) -> Result<(f64, [[f64; 2]; 3]), P1Error> {
    let (x0, y0) = (points[0].x, points[0].y);
    let (x1, y1) = (points[1].x, points[1].y);
    let (x2, y2) = (points[2].x, points[2].y);
    let signed_twice_area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);
    if signed_twice_area.abs() < DEGENERATE_AREA {
        return Err(P1Error::DegenerateTriangle {
            cell,
            signed_area: 0.5 * signed_twice_area,
        });
    }
    let inverse = 1.0 / signed_twice_area;
    let gradients = [
        [(y1 - y2) * inverse, (x2 - x1) * inverse],
        [(y2 - y0) * inverse, (x0 - x2) * inverse],
        [(y0 - y1) * inverse, (x1 - x0) * inverse],
    ];
    Ok((0.5 * signed_twice_area.abs(), gradients))
}

fn validate_form(form: &FormProgram, unknown: FieldId, test: FieldId) -> Result<(), P1Error> {
    let unknown_field = find_field(form, unknown)?;
    let test_field = find_field(form, test)?;
    validate_p1_scalar_h1(unknown_field, false)?;
    validate_p1_scalar_h1(test_field, true)?;

    let has_diffusion_term = form.residual_terms.iter().any(|integral| {
        contains_gradient_of(&integral.integrand, unknown)
            && contains_gradient_of(&integral.integrand, test)
            && matches!(integral.measure, crate::form::Measure::Volume { .. })
    });
    if !has_diffusion_term {
        return Err(P1Error::UnsupportedForm(
            "expected a volume residual term coupling gradients of the unknown and test fields"
                .into(),
        ));
    }
    Ok(())
}

fn find_field(form: &FormProgram, id: FieldId) -> Result<&Field, P1Error> {
    form.fields
        .iter()
        .find(|field| field.id == id)
        .ok_or(P1Error::MissingField(id.0))
}

fn validate_p1_scalar_h1(field: &Field, require_test: bool) -> Result<(), P1Error> {
    if !matches!(field.space.value_shape, ValueShape::Scalar)
        || field.space.order != 1
        || !matches!(field.space.continuity, Continuity::H1)
        || !field.space.family.eq_ignore_ascii_case("lagrange")
    {
        return Err(P1Error::UnsupportedFieldSpace(field.name.clone()));
    }
    if require_test && !matches!(field.role, FieldRole::Test) {
        return Err(P1Error::UnsupportedFieldRole {
            field: field.name.clone(),
            expected: "test".into(),
        });
    }
    if !require_test && !matches!(field.role, FieldRole::Unknown | FieldRole::Trial) {
        return Err(P1Error::UnsupportedFieldRole {
            field: field.name.clone(),
            expected: "unknown or trial".into(),
        });
    }
    Ok(())
}

fn contains_gradient_of(expression: &FormExpr, field: FieldId) -> bool {
    match expression {
        FormExpr::Gradient(inner) => contains_field(inner, field),
        FormExpr::Neg(inner)
        | FormExpr::Divergence(inner)
        | FormExpr::Curl(inner)
        | FormExpr::TimeDerivative(inner)
        | FormExpr::Trace(inner) => contains_gradient_of(inner, field),
        FormExpr::Add(items) | FormExpr::Product(items) => {
            items.iter().any(|item| contains_gradient_of(item, field))
        }
        FormExpr::Inner { left, right } | FormExpr::Contract { left, right } => {
            contains_gradient_of(left, field) || contains_gradient_of(right, field)
        }
        FormExpr::Custom { args, .. } => args.iter().any(|arg| contains_gradient_of(arg, field)),
        FormExpr::Scalar(_) | FormExpr::Field(_) => false,
    }
}

fn contains_field(expression: &FormExpr, field: FieldId) -> bool {
    match expression {
        FormExpr::Field(candidate) => *candidate == field,
        FormExpr::Neg(inner)
        | FormExpr::Gradient(inner)
        | FormExpr::Divergence(inner)
        | FormExpr::Curl(inner)
        | FormExpr::TimeDerivative(inner)
        | FormExpr::Trace(inner) => contains_field(inner, field),
        FormExpr::Add(items) | FormExpr::Product(items) => {
            items.iter().any(|item| contains_field(item, field))
        }
        FormExpr::Inner { left, right } | FormExpr::Contract { left, right } => {
            contains_field(left, field) || contains_field(right, field)
        }
        FormExpr::Custom { args, .. } => args.iter().any(|arg| contains_field(arg, field)),
        FormExpr::Scalar(_) => false,
    }
}

fn stiffness_program(request: &P1DiscretizationRequest) -> DiscreteProgram {
    let mut program = DiscreteProgram {
        name: "p1_scalar_diffusion".into(),
        instructions: Vec::new(),
        outputs: Vec::new(),
        metadata: BTreeMap::from([
            ("scheme".into(), "continuous_galerkin_p1".into()),
            ("topology".into(), "triangle".into()),
            ("assembly".into(), "deterministic_csr".into()),
        ]),
    };
    let field = program.push(DiscreteOp::FieldInput {
        field: request.unknown,
    });
    let element = program.push(DiscreteOp::Restrict {
        input: field,
        field: request.unknown,
        direction: RestrictionDirection::Gather,
    });
    let gradient = program.push(DiscreteOp::Basis {
        input: element,
        field: request.unknown,
        evaluation: BasisEvaluation::Gradient,
        transpose: false,
    });
    let flux = program.push(DiscreteOp::Custom {
        operator: "piecewise_constant_scalar_diffusion".into(),
        inputs: vec![gradient],
        metadata: BTreeMap::new(),
    });
    let weighted = program.push(DiscreteOp::QuadratureWeight {
        input: flux,
        rule: "analytic_p1_triangle".into(),
    });
    let tested = program.push(DiscreteOp::Basis {
        input: weighted,
        field: request.test,
        evaluation: BasisEvaluation::Gradient,
        transpose: true,
    });
    let assembled = program.push(DiscreteOp::Restrict {
        input: tested,
        field: request.test,
        direction: RestrictionDirection::ScatterAdd,
    });
    program.outputs.push(assembled);
    program
}

fn mass_program(request: &P1DiscretizationRequest) -> DiscreteProgram {
    let mut program = DiscreteProgram {
        name: "p1_scalar_mass".into(),
        instructions: Vec::new(),
        outputs: Vec::new(),
        metadata: BTreeMap::from([
            ("scheme".into(), "continuous_galerkin_p1".into()),
            ("topology".into(), "triangle".into()),
        ]),
    };
    let field = program.push(DiscreteOp::FieldInput {
        field: request.unknown,
    });
    let element = program.push(DiscreteOp::Restrict {
        input: field,
        field: request.unknown,
        direction: RestrictionDirection::Gather,
    });
    let value = program.push(DiscreteOp::Basis {
        input: element,
        field: request.unknown,
        evaluation: BasisEvaluation::Value,
        transpose: false,
    });
    let capacity = program.push(DiscreteOp::Custom {
        operator: "piecewise_constant_capacity".into(),
        inputs: vec![value],
        metadata: BTreeMap::new(),
    });
    let weighted = program.push(DiscreteOp::QuadratureWeight {
        input: capacity,
        rule: "analytic_p1_triangle_mass".into(),
    });
    let tested = program.push(DiscreteOp::Basis {
        input: weighted,
        field: request.test,
        evaluation: BasisEvaluation::Value,
        transpose: true,
    });
    let assembled = program.push(DiscreteOp::Restrict {
        input: tested,
        field: request.test,
        direction: RestrictionDirection::ScatterAdd,
    });
    program.outputs.push(assembled);
    program
}

fn operator_program(
    request: &P1DiscretizationRequest,
    stiffness: DiscreteProgramId,
    mass: Option<DiscreteProgramId>,
    assembly: &ScalarEllipticAssembly,
) -> OperatorProgram {
    let mut blocks = vec![OperatorBlock {
        name: "stiffness".into(),
        kind: OperatorBlockKind::Stiffness,
        program: stiffness,
        row_variables: vec![format!("field:{}", request.test.0)],
        column_variables: vec![format!("field:{}", request.unknown.0)],
    }];
    if let Some(mass) = mass {
        blocks.push(OperatorBlock {
            name: "mass".into(),
            kind: OperatorBlockKind::Mass,
            program: mass,
            row_variables: vec![format!("field:{}", request.test.0)],
            column_variables: vec![format!("field:{}", request.unknown.0)],
        });
    }

    OperatorProgram {
        name: "p1_scalar_elliptic".into(),
        blocks,
        derivatives: vec![
            DerivativeCapability::AnalyticJacobian,
            DerivativeCapability::Jvp,
            DerivativeCapability::Vjp,
        ],
        properties: vec![
            OperatorProperty::Symmetric,
            OperatorProperty::PositiveDefinite,
            OperatorProperty::UnitsConsistent,
        ],
        sparsity: Some(SparsityContract {
            rows: assembly.dof_map.n_free(),
            cols: assembly.dof_map.n_free(),
            block_pattern: Vec::new(),
            note: Some("mesh-fixed P1 vertex adjacency; concrete CSR is in the numerical artifact".into()),
        }),
        metadata: BTreeMap::from([
            ("reference_backend".into(), "resolvent::p1".into()),
            ("rhs_terms".into(), "dirichlet_lift+volume_source+neumann_flux".into()),
        ]),
    }
}

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
mod tests {
    use super::*;
    use crate::form::{FunctionSpace, Integral, Measure};

    fn close(actual: f64, expected: f64) {
        let scale = 1.0_f64.max(actual.abs()).max(expected.abs());
        assert!(
            (actual - expected).abs() <= 1.0e-12 * scale,
            "actual={actual:?}, expected={expected:?}"
        );
    }

    fn triangle_mesh() -> P1Mesh {
        P1Mesh {
            vertices: vec![
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(0.0, 1.0),
            ],
            cells: vec![Triangle {
                vertices: [0, 1, 2],
                region: 7,
            }],
            boundary_edges: vec![
                BoundaryEdge {
                    vertices: [0, 1],
                    tag: 10,
                },
                BoundaryEdge {
                    vertices: [1, 2],
                    tag: 20,
                },
                BoundaryEdge {
                    vertices: [2, 0],
                    tag: 30,
                },
            ],
        }
    }

    #[test]
    fn stiffness_matches_closed_form_triangle() {
        let assembled = assemble_scalar_elliptic(
            &triangle_mesh(),
            &ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(2.0),
                ..ScalarEllipticInput::default()
            },
        )
        .unwrap();
        let dense = assembled.stiffness_full.to_dense();
        let expected = [[2.0, -1.0, -1.0], [-1.0, 1.0, 0.0], [-1.0, 0.0, 1.0]];
        for (actual_row, expected_row) in dense.iter().zip(expected) {
            for (&actual, expected) in actual_row.iter().zip(expected_row) {
                close(actual, expected);
            }
        }
    }

    #[test]
    fn source_neumann_and_dirichlet_lift_share_one_rhs() {
        let mut source = PiecewiseSource::default();
        source.per_region.insert(7, 6.0);
        let mut neumann = BoundaryFlux::default();
        neumann.per_tag.insert(20, 2.0);
        let assembled = assemble_scalar_elliptic(
            &triangle_mesh(),
            &ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(2.0),
                source,
                neumann,
                dirichlet: vec![DirichletBoundary {
                    tag: 10,
                    value: 4.0,
                }],
            },
        )
        .unwrap();

        assert_eq!(assembled.dof_map.n_free(), 1);
        close(assembled.stiffness_free.to_dense()[0][0], 1.0);
        close(assembled.dirichlet_lift[0], 4.0);
        close(assembled.source_free[0], 1.0);
        close(assembled.neumann_free[0], 2.0_f64.sqrt());
        close(assembled.rhs[0], 5.0 + 2.0_f64.sqrt());
    }

    #[test]
    fn consistent_and_lumped_mass_preserve_total_capacity() {
        let mesh = triangle_mesh();
        let dof = DofMap::from_dirichlet(&mesh, &[]).unwrap();
        let consistent = assemble_mass(
            &mesh,
            &dof,
            &MassInput {
                capacity: PiecewiseConstant::uniform(3.0),
                lumping: MassLumping::Consistent,
            },
        )
        .unwrap();
        let lumped = assemble_mass(
            &mesh,
            &dof,
            &MassInput {
                capacity: PiecewiseConstant::uniform(3.0),
                lumping: MassLumping::Lumped,
            },
        )
        .unwrap();

        let total_consistent: f64 = consistent.mass_full.values.iter().sum();
        let total_lumped: f64 = lumped.mass_full.values.iter().sum();
        close(total_consistent, 1.5);
        close(total_lumped, 1.5);
        let dense = lumped.mass_full.to_dense();
        close(dense[0][0], 0.5);
        close(dense[1][1], 0.5);
        close(dense[2][2], 0.5);
    }

    #[test]
    fn evolution_residual_and_shifted_matrix_are_consistent() {
        let mesh = triangle_mesh();
        let elliptic = assemble_scalar_elliptic(
            &mesh,
            &ScalarEllipticInput {
                diffusion: PiecewiseConstant::uniform(2.0),
                ..ScalarEllipticInput::default()
            },
        )
        .unwrap();
        let mass = assemble_mass(
            &mesh,
            &elliptic.dof_map,
            &MassInput {
                capacity: PiecewiseConstant::uniform(3.0),
                lumping: MassLumping::Consistent,
            },
        )
        .unwrap();
        let evolution = EvolutionAssembly {
            stiffness: elliptic.stiffness_free,
            mass: mass.mass_free,
            rhs: elliptic.rhs,
            class: EvolutionClass::Ode,
        };
        let state = [1.0, 2.0, 3.0];
        let rate = [0.5, -0.5, 1.0];
        let residual = evolution.residual(&state, &rate).unwrap();
        let kx = evolution.static_jvp(&state).unwrap();
        let mx = evolution.mass_jvp(&rate).unwrap();
        for ((&actual, stiffness), mass) in residual.iter().zip(kx).zip(mx) {
            close(actual, stiffness + mass);
        }

        let shifted = evolution.iteration_matrix(10.0, 1.0).unwrap();
        let direction = [0.25, 0.5, -0.25];
        let applied = shifted.apply(&direction).unwrap();
        let expected_mass = evolution.mass_jvp(&direction).unwrap();
        let expected_stiffness = evolution.static_jvp(&direction).unwrap();
        for ((&actual, mass), stiffness) in applied
            .iter()
            .zip(expected_mass)
            .zip(expected_stiffness)
        {
            close(actual, 10.0 * mass + stiffness);
        }
    }

    #[test]
    fn compiler_emits_discrete_operator_and_refinement_artifacts() {
        let mut context = Context::new();
        let unknown = context.allocate_field_id();
        let test = context.allocate_field_id();
        let space = FunctionSpace {
            family: "Lagrange".into(),
            order: 1,
            continuity: Continuity::H1,
            value_shape: ValueShape::Scalar,
            domain: Some("omega".into()),
        };
        let form = context.insert_form(FormProgram {
            name: "heat".into(),
            fields: vec![
                Field {
                    id: unknown,
                    name: "temperature".into(),
                    role: FieldRole::Unknown,
                    space: space.clone(),
                    dimension: Some("K".into()),
                    metadata: BTreeMap::new(),
                },
                Field {
                    id: test,
                    name: "test_temperature".into(),
                    role: FieldRole::Test,
                    space,
                    dimension: None,
                    metadata: BTreeMap::new(),
                },
            ],
            residual_terms: vec![Integral {
                integrand: FormExpr::Inner {
                    left: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(test)))),
                    right: Box::new(FormExpr::Gradient(Box::new(FormExpr::Field(unknown)))),
                },
                measure: Measure::Volume {
                    domain: "omega".into(),
                },
                label: Some("diffusion".into()),
            }],
            boundary_terms: Vec::new(),
            metadata: BTreeMap::new(),
        });

        let result = lower_p1(
            &mut context,
            &P1DiscretizationRequest {
                form,
                unknown,
                test,
                mesh: triangle_mesh(),
                elliptic: ScalarEllipticInput {
                    diffusion: PiecewiseConstant::uniform(4.0),
                    ..ScalarEllipticInput::default()
                },
                mass: Some(MassInput {
                    capacity: PiecewiseConstant::uniform(2.0),
                    lumping: MassLumping::Consistent,
                }),
            },
        )
        .unwrap();

        assert!(context.discrete(result.stiffness_program).is_some());
        assert!(context.discrete(result.mass_program.unwrap()).is_some());
        assert!(context.operator(result.operator).is_some());
        let refinement = context.refinement(result.refinement).unwrap();
        assert!(matches!(
            refinement.relation,
            RefinementRelation::Discretization {
                declared_order: Some(1),
                ..
            }
        ));
        assert_eq!(result.evolution.unwrap().class, EvolutionClass::Ode);
    }
}
