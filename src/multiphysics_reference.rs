//! Portable reference finite-element kernels used to close the R12 physics-space gaps.
//!
//! These routines are deliberately small and deterministic. They are not intended to compete
//! with an optimized Sinbad/Anvil backend; they are executable semantic witnesses for vector
//! H1 elasticity, mixed velocity/pressure systems, and orientation-sensitive H(curl) spaces.

use crate::reference::{PiecewiseConstant, SparseEntry, SparseMatrix, TriangleMesh};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaneKinematics {
    PlaneStress,
    PlaneStrain,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearElasticity2d {
    pub mesh: TriangleMesh,
    pub young_modulus: PiecewiseConstant,
    pub poisson_ratio: PiecewiseConstant,
    pub kinematics: PlaneKinematics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssembledElasticity2d {
    /// Two displacement DOFs per vertex, ordered `[u_x, u_y]`.
    pub stiffness: SparseMatrix,
}

/// Assemble first-order vector-H1 isotropic linear elasticity on a triangle mesh.
pub fn assemble_linear_elasticity_p1(
    problem: &LinearElasticity2d,
) -> Result<AssembledElasticity2d, MultiphysicsReferenceError> {
    let mut entries = Vec::new();
    for cell_index in 0..problem.mesh.cells.len() {
        let (vertices, area, gradients) = triangle_geometry(&problem.mesh, cell_index)?;
        let young = problem
            .young_modulus
            .at(problem.mesh.cells[cell_index].region);
        let poisson = problem
            .poisson_ratio
            .at(problem.mesh.cells[cell_index].region);
        let constitutive = elasticity_matrix(cell_index, young, poisson, problem.kinematics)?;

        let mut b = [[0.0; 6]; 3];
        for a in 0..3 {
            let gx = gradients[a][0];
            let gy = gradients[a][1];
            b[0][2 * a] = gx;
            b[1][2 * a + 1] = gy;
            b[2][2 * a] = gy;
            b[2][2 * a + 1] = gx;
        }

        for i in 0..6 {
            for j in 0..6 {
                let mut value = 0.0;
                for a in 0..3 {
                    for c in 0..3 {
                        value += b[a][i] * constitutive[a][c] * b[c][j];
                    }
                }
                entries.push(SparseEntry {
                    row: 2 * vertices[i / 2] + i % 2,
                    col: 2 * vertices[j / 2] + j % 2,
                    value: area * value,
                });
            }
        }
    }
    Ok(AssembledElasticity2d {
        stiffness: SparseMatrix::from_triplets(
            2 * problem.mesh.vertices.len(),
            2 * problem.mesh.vertices.len(),
            entries,
        ),
    })
}

fn elasticity_matrix(
    cell: usize,
    young: f64,
    poisson: f64,
    kinematics: PlaneKinematics,
) -> Result<[[f64; 3]; 3], MultiphysicsReferenceError> {
    if !young.is_finite() || young <= 0.0 {
        return Err(MultiphysicsReferenceError::InvalidMaterial {
            cell,
            name: "young_modulus".into(),
            value: young,
        });
    }
    if !poisson.is_finite() || poisson <= -1.0 || poisson >= 0.5 {
        return Err(MultiphysicsReferenceError::InvalidMaterial {
            cell,
            name: "poisson_ratio".into(),
            value: poisson,
        });
    }
    Ok(match kinematics {
        PlaneKinematics::PlaneStress => {
            let scale = young / (1.0 - poisson * poisson);
            [
                [scale, scale * poisson, 0.0],
                [scale * poisson, scale, 0.0],
                [0.0, 0.0, scale * (1.0 - poisson) / 2.0],
            ]
        }
        PlaneKinematics::PlaneStrain => {
            let mu = young / (2.0 * (1.0 + poisson));
            let lambda = young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson));
            [
                [lambda + 2.0 * mu, lambda, 0.0],
                [lambda, lambda + 2.0 * mu, 0.0],
                [0.0, 0.0, mu],
            ]
        }
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StokesP1P0Problem2d {
    pub mesh: TriangleMesh,
    pub viscosity: PiecewiseConstant,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssembledStokesP1P0 {
    /// Vector-H1 viscous block. Two velocity DOFs per mesh vertex.
    pub velocity_stiffness: SparseMatrix,
    /// Cellwise P0 pressure test functions against `div(u)`.
    pub divergence: SparseMatrix,
    /// Symmetric saddle matrix `[A B^T; B 0]`.
    pub saddle: SparseMatrix,
    pub pressure_offset: usize,
}

/// Assemble a minimal mixed Stokes witness using vector P1 velocity and cellwise P0 pressure.
///
/// P1/P0 is useful here as a compact mixed-space semantic witness. This function makes no
/// claim that the pair satisfies an inf-sup condition on arbitrary production meshes.
pub fn assemble_stokes_p1_p0(
    problem: &StokesP1P0Problem2d,
) -> Result<AssembledStokesP1P0, MultiphysicsReferenceError> {
    let velocity_dofs = 2 * problem.mesh.vertices.len();
    let pressure_dofs = problem.mesh.cells.len();
    let mut a_entries = Vec::new();
    let mut b_entries = Vec::new();

    for cell_index in 0..problem.mesh.cells.len() {
        let (vertices, area, gradients) = triangle_geometry(&problem.mesh, cell_index)?;
        let viscosity = problem.viscosity.at(problem.mesh.cells[cell_index].region);
        require_positive(cell_index, "viscosity", viscosity)?;
        for i in 0..3 {
            for j in 0..3 {
                let scalar = viscosity
                    * area
                    * (gradients[i][0] * gradients[j][0]
                        + gradients[i][1] * gradients[j][1]);
                for component in 0..2 {
                    a_entries.push(SparseEntry {
                        row: 2 * vertices[i] + component,
                        col: 2 * vertices[j] + component,
                        value: scalar,
                    });
                }
            }
            for component in 0..2 {
                b_entries.push(SparseEntry {
                    row: cell_index,
                    col: 2 * vertices[i] + component,
                    value: area * gradients[i][component],
                });
            }
        }
    }

    let velocity_stiffness =
        SparseMatrix::from_triplets(velocity_dofs, velocity_dofs, a_entries.clone());
    let divergence =
        SparseMatrix::from_triplets(pressure_dofs, velocity_dofs, b_entries.clone());
    let pressure_offset = velocity_dofs;
    let saddle = SparseMatrix::from_triplets(
        velocity_dofs + pressure_dofs,
        velocity_dofs + pressure_dofs,
        a_entries.into_iter().chain(b_entries.into_iter().flat_map(|entry| {
            [
                SparseEntry {
                    row: pressure_offset + entry.row,
                    col: entry.col,
                    value: entry.value,
                },
                SparseEntry {
                    row: entry.col,
                    col: pressure_offset + entry.row,
                    value: entry.value,
                },
            ]
        })),
    );

    Ok(AssembledStokesP1P0 {
        velocity_stiffness,
        divergence,
        saddle,
        pressure_offset,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HcurlMaxwellProblem2d {
    pub mesh: TriangleMesh,
    /// Coefficient multiplying `curl(E) curl(v)`, normally `1 / mu`.
    pub inverse_permeability: PiecewiseConstant,
    /// Coefficient multiplying `E . v`, normally permittivity.
    pub permittivity: PiecewiseConstant,
    /// Angular frequency. The assembled operator is `C - omega^2 M`.
    pub omega: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssembledHcurlMaxwell2d {
    /// Canonically oriented global edges `(min_vertex, max_vertex)`.
    pub edges: Vec<[usize; 2]>,
    pub curl_curl: SparseMatrix,
    pub mass: SparseMatrix,
    /// Time-harmonic Maxwell/waveguide witness `curl(mu^-1 curl E) - omega^2 eps E`.
    pub operator: SparseMatrix,
}

/// Assemble lowest-order first-family Nedelec/Whitney edge elements on triangles.
///
/// Local edge orientations are mapped to a canonical global orientation. Reversing a cell's
/// vertex ordering therefore does not change the represented global operator.
pub fn assemble_hcurl_maxwell_nedelec0(
    problem: &HcurlMaxwellProblem2d,
) -> Result<AssembledHcurlMaxwell2d, MultiphysicsReferenceError> {
    if !problem.omega.is_finite() {
        return Err(MultiphysicsReferenceError::InvalidFrequency(problem.omega));
    }

    let mut edge_set = BTreeSet::new();
    for cell in &problem.mesh.cells {
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let va = cell.vertices[a];
            let vb = cell.vertices[b];
            if va >= problem.mesh.vertices.len() {
                return Err(MultiphysicsReferenceError::BadVertex(va));
            }
            if vb >= problem.mesh.vertices.len() {
                return Err(MultiphysicsReferenceError::BadVertex(vb));
            }
            edge_set.insert(canonical_edge(va, vb));
        }
    }
    let edges: Vec<[usize; 2]> = edge_set.into_iter().collect();
    let edge_index: BTreeMap<[usize; 2], usize> = edges
        .iter()
        .copied()
        .enumerate()
        .map(|(index, edge)| (edge, index))
        .collect();

    let mut curl_entries = Vec::new();
    let mut mass_entries = Vec::new();
    let local_edges = [(0usize, 1usize), (1, 2), (2, 0)];

    for cell_index in 0..problem.mesh.cells.len() {
        let (vertices, area, gradients) = triangle_geometry(&problem.mesh, cell_index)?;
        let region = problem.mesh.cells[cell_index].region;
        let inv_mu = problem.inverse_permeability.at(region);
        let epsilon = problem.permittivity.at(region);
        require_positive(cell_index, "inverse_permeability", inv_mu)?;
        require_positive(cell_index, "permittivity", epsilon)?;

        let mut global = [0usize; 3];
        let mut signs = [1.0f64; 3];
        for (local, &(a, b)) in local_edges.iter().enumerate() {
            let va = vertices[a];
            let vb = vertices[b];
            let canonical = canonical_edge(va, vb);
            global[local] = edge_index[&canonical];
            signs[local] = if [va, vb] == canonical { 1.0 } else { -1.0 };
        }

        for i in 0..3 {
            let (a, b) = local_edges[i];
            let curl_i = 2.0 * cross(gradients[a], gradients[b]);
            for j in 0..3 {
                let (c, d) = local_edges[j];
                let curl_j = 2.0 * cross(gradients[c], gradients[d]);
                let orientation = signs[i] * signs[j];
                curl_entries.push(SparseEntry {
                    row: global[i],
                    col: global[j],
                    value: orientation * inv_mu * area * curl_i * curl_j,
                });
                mass_entries.push(SparseEntry {
                    row: global[i],
                    col: global[j],
                    value: orientation
                        * epsilon
                        * nedelec_mass_entry(area, gradients, a, b, c, d),
                });
            }
        }
    }

    let curl_curl = SparseMatrix::from_triplets(edges.len(), edges.len(), curl_entries);
    let mass = SparseMatrix::from_triplets(edges.len(), edges.len(), mass_entries);
    let operator = curl_curl
        .scaled_add(1.0, &mass, -(problem.omega * problem.omega))
        .map_err(|_| MultiphysicsReferenceError::InternalMatrixShape)?;
    Ok(AssembledHcurlMaxwell2d {
        edges,
        curl_curl,
        mass,
        operator,
    })
}

fn nedelec_mass_entry(
    area: f64,
    gradients: [[f64; 2]; 3],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
) -> f64 {
    lambda_moment(area, a, c) * dot(gradients[b], gradients[d])
        - lambda_moment(area, a, d) * dot(gradients[b], gradients[c])
        - lambda_moment(area, b, c) * dot(gradients[a], gradients[d])
        + lambda_moment(area, b, d) * dot(gradients[a], gradients[c])
}

fn lambda_moment(area: f64, i: usize, j: usize) -> f64 {
    area * if i == j { 1.0 / 6.0 } else { 1.0 / 12.0 }
}

fn canonical_edge(a: usize, b: usize) -> [usize; 2] {
    if a < b { [a, b] } else { [b, a] }
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

fn require_positive(
    cell: usize,
    name: &str,
    value: f64,
) -> Result<(), MultiphysicsReferenceError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(MultiphysicsReferenceError::InvalidMaterial {
            cell,
            name: name.into(),
            value,
        })
    }
}

fn triangle_geometry(
    mesh: &TriangleMesh,
    cell_index: usize,
) -> Result<([usize; 3], f64, [[f64; 2]; 3]), MultiphysicsReferenceError> {
    let cell = mesh
        .cells
        .get(cell_index)
        .ok_or(MultiphysicsReferenceError::BadCell(cell_index))?;
    let mut points = [[0.0; 2]; 3];
    for i in 0..3 {
        points[i] = *mesh
            .vertices
            .get(cell.vertices[i])
            .ok_or(MultiphysicsReferenceError::BadVertex(cell.vertices[i]))?;
    }
    let det = (points[1][0] - points[0][0]) * (points[2][1] - points[0][1])
        - (points[2][0] - points[0][0]) * (points[1][1] - points[0][1]);
    let scale = points
        .iter()
        .flat_map(|p| p.iter())
        .map(|v| v.abs())
        .fold(1.0f64, f64::max);
    if !det.is_finite() || det.abs() <= 64.0 * f64::EPSILON * scale * scale {
        return Err(MultiphysicsReferenceError::DegenerateCell(cell_index));
    }
    let gradients = [
        [
            (points[1][1] - points[2][1]) / det,
            (points[2][0] - points[1][0]) / det,
        ],
        [
            (points[2][1] - points[0][1]) / det,
            (points[0][0] - points[2][0]) / det,
        ],
        [
            (points[0][1] - points[1][1]) / det,
            (points[1][0] - points[0][0]) / det,
        ],
    ];
    Ok((cell.vertices, det.abs() / 2.0, gradients))
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum MultiphysicsReferenceError {
    #[error("cell index {0} is out of range")]
    BadCell(usize),
    #[error("vertex index {0} is out of range")]
    BadVertex(usize),
    #[error("triangle cell {0} is degenerate or non-finite")]
    DegenerateCell(usize),
    #[error("invalid material value in cell {cell}: {name}={value}")]
    InvalidMaterial {
        cell: usize,
        name: String,
        value: f64,
    },
    #[error("invalid angular frequency {0}")]
    InvalidFrequency(f64),
    #[error("internal matrix shape mismatch")]
    InternalMatrixShape,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::TriangleCell;

    fn mesh(vertices: [usize; 3]) -> TriangleMesh {
        TriangleMesh {
            vertices: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            cells: vec![TriangleCell { vertices, region: 0 }],
            boundaries: vec![],
        }
    }

    fn max_abs(values: &[f64]) -> f64 {
        values.iter().map(|x| x.abs()).fold(0.0, f64::max)
    }

    fn dense(matrix: &SparseMatrix) -> Vec<f64> {
        let mut out = vec![0.0; matrix.rows * matrix.cols];
        for entry in &matrix.entries {
            out[entry.row * matrix.cols + entry.col] += entry.value;
        }
        out
    }

    #[test]
    fn elasticity_has_rigid_translation_null_modes() {
        let assembled = assemble_linear_elasticity_p1(&LinearElasticity2d {
            mesh: mesh([0, 1, 2]),
            young_modulus: PiecewiseConstant::uniform(200.0),
            poisson_ratio: PiecewiseConstant::uniform(0.3),
            kinematics: PlaneKinematics::PlaneStrain,
        })
        .unwrap();
        assert!(assembled.stiffness.is_symmetric(1e-12));
        let tx = assembled
            .stiffness
            .apply(&[1.0, 0.0, 1.0, 0.0, 1.0, 0.0])
            .unwrap();
        let ty = assembled
            .stiffness
            .apply(&[0.0, 1.0, 0.0, 1.0, 0.0, 1.0])
            .unwrap();
        assert!(max_abs(&tx) < 1e-10);
        assert!(max_abs(&ty) < 1e-10);
    }

    #[test]
    fn stokes_mixed_block_annihilates_constant_velocity_divergence() {
        let assembled = assemble_stokes_p1_p0(&StokesP1P0Problem2d {
            mesh: mesh([0, 1, 2]),
            viscosity: PiecewiseConstant::uniform(1.5),
        })
        .unwrap();
        assert!(assembled.saddle.is_symmetric(1e-12));
        let divergence = assembled
            .divergence
            .apply(&[2.0, -3.0, 2.0, -3.0, 2.0, -3.0])
            .unwrap();
        assert!(max_abs(&divergence) < 1e-12);
    }

    #[test]
    fn hcurl_global_operator_is_invariant_to_cell_orientation() {
        let problem = |vertices| HcurlMaxwellProblem2d {
            mesh: mesh(vertices),
            inverse_permeability: PiecewiseConstant::uniform(2.0),
            permittivity: PiecewiseConstant::uniform(3.0),
            omega: 0.7,
        };
        let forward = assemble_hcurl_maxwell_nedelec0(&problem([0, 1, 2])).unwrap();
        let reversed = assemble_hcurl_maxwell_nedelec0(&problem([0, 2, 1])).unwrap();
        assert_eq!(forward.edges, reversed.edges);
        for (left, right) in dense(&forward.operator)
            .into_iter()
            .zip(dense(&reversed.operator))
        {
            assert!((left - right).abs() < 1e-12);
        }
        assert!(forward.curl_curl.is_symmetric(1e-12));
        assert!(forward.mass.is_symmetric(1e-12));
        assert!(forward.mass.diagonal().iter().all(|value| *value > 0.0));
    }
}
