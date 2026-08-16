use super::P1Error;
use super::matrix::CsrMatrix;
use super::mesh::{
    BoundaryFlux, DirichletBoundary, DofMap, P1Mesh, PiecewiseConstant, PiecewiseSource,
    p1_shape_gradients,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MassLumping {
    #[default]
    Consistent,
    Lumped,
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
    /// The mass matrix is proven nonsingular under the reference backend's current assumptions.
    Ode,
    /// A downstream structural compiler has established a DAE index.
    Dae { index: u8 },
    /// Resolvent cannot currently prove the mass block nonsingular and does not guess a DAE
    /// index. Structural classification belongs to the model/operator compiler, not heuristics.
    Unclassified,
}

/// Semi-discrete linear evolution `M*u_dot + K*u = f`. Time-step selection,
/// globalization and linear-solve policy belong downstream.
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

        let stiffness = self.stiffness.apply(state)?;
        let mass = self.mass.apply(rate)?;
        Ok(mass
            .into_iter()
            .zip(stiffness)
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
        // The first vertical is symmetric diffusion. A nonsymmetric form must provide a
        // distinct transpose implementation rather than inheriting this shortcut.
        self.stiffness.apply(direction)
    }

    pub fn mass_jvp(&self, direction: &[f64]) -> Result<Vec<f64>, P1Error> {
        self.mass.apply(direction)
    }
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

        for (&vertex_i, gradient_i) in triangle.vertices.iter().zip(gradients.iter()) {
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
