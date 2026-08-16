//! Independent reference discretizations used to falsify the agent physics authoring stack.
//!
//! These are deliberately small deterministic oracles, not performance backends. They cover
//! mathematical structures that did not exist in Residua: vector H1 elasticity, a mixed
//! saddle block, and globally oriented lowest-order H(curl) elements.

use crate::reference::{ReferenceError, ReferenceMesh2, SparseMatrix};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsotropicElasticity2 {
    pub lambda: f64,
    pub mu: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElasticityOperator2 {
    /// Interleaved vertex DOFs `[u0x,u0y,u1x,u1y,...]`.
    pub stiffness: SparseMatrix,
}

pub fn compile_elasticity_p1_2d(
    mesh: &ReferenceMesh2,
    material: IsotropicElasticity2,
) -> Result<ElasticityOperator2, ReferenceError> {
    validate_regions(mesh)?;
    let n = 2 * mesh.vertices.len();
    let mut coo = vec![];
    let d = [
        [material.lambda + 2.0 * material.mu, material.lambda, 0.0],
        [material.lambda, material.lambda + 2.0 * material.mu, 0.0],
        [0.0, 0.0, material.mu],
    ];
    for (cell, tri) in mesh.triangles.iter().copied().enumerate() {
        let (area, grad) = triangle_geometry(mesh, cell, tri)?;
        let mut b = [[0.0; 6]; 3];
        for a in 0..3 {
            b[0][2 * a] = grad[a][0];
            b[1][2 * a + 1] = grad[a][1];
            b[2][2 * a] = grad[a][1];
            b[2][2 * a + 1] = grad[a][0];
        }
        for a in 0..6 {
            for c in 0..6 {
                let mut value = 0.0;
                for i in 0..3 {
                    for j in 0..3 {
                        value += b[i][a] * d[i][j] * b[j][c];
                    }
                }
                coo.push((2 * tri[a / 2] + a % 2, 2 * tri[c / 2] + c % 2, area * value));
            }
        }
    }
    Ok(ElasticityOperator2 {
        stiffness: SparseMatrix::from_coo(n, n, coo),
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StokesOperatorP1P1 {
    /// Ordering is velocity x/y per vertex followed by one pressure DOF per vertex.
    pub saddle: SparseMatrix,
    pub velocity_dofs: usize,
    pub pressure_dofs: usize,
}

/// Reference equal-order P1/P1 Stokes block. This is a structural/compiler oracle, not a
/// claim that unstabilized P1/P1 is a generally stable production pair. The purpose is to
/// exercise mixed spaces, zero diagonal blocks, gradient/divergence transpose structure and
/// saddle-point solver metadata.
pub fn compile_stokes_p1p1_2d(
    mesh: &ReferenceMesh2,
    viscosity: f64,
) -> Result<StokesOperatorP1P1, ReferenceError> {
    validate_regions(mesh)?;
    let nv = mesh.vertices.len();
    let vel = 2 * nv;
    let total = vel + nv;
    let mut coo = vec![];
    for (cell, tri) in mesh.triangles.iter().copied().enumerate() {
        let (area, grad) = triangle_geometry(mesh, cell, tri)?;
        for a in 0..3 {
            for b in 0..3 {
                let lap = viscosity * area * dot2(grad[a], grad[b]);
                coo.push((2 * tri[a], 2 * tri[b], lap));
                coo.push((2 * tri[a] + 1, 2 * tri[b] + 1, lap));

                // B_ij = -∫ q_i div(phi_j e_component) = -A/3 * d phi_j/dx_k.
                let bx = -(area / 3.0) * grad[b][0];
                let by = -(area / 3.0) * grad[b][1];
                let prow = vel + tri[a];
                let ux = 2 * tri[b];
                let uy = ux + 1;
                coo.push((prow, ux, bx));
                coo.push((ux, prow, bx));
                coo.push((prow, uy, by));
                coo.push((uy, prow, by));
            }
        }
    }
    Ok(StokesOperatorP1P1 {
        saddle: SparseMatrix::from_coo(total, total, coo),
        velocity_dofs: vel,
        pressure_dofs: nv,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NedelecOperator2 {
    /// Canonically oriented global edges `(min_vertex,max_vertex)` in lexicographic order.
    pub edges: Vec<[usize; 2]>,
    pub curl_curl: SparseMatrix,
    pub mass: SparseMatrix,
}

/// Lowest-order first-kind Nedelec/Whitney edge basis on triangles. Global edge orientation
/// is canonical and local basis signs are corrected during assembly, making the operator
/// independent of triangle vertex ordering.
pub fn compile_nedelec0_2d(mesh: &ReferenceMesh2) -> Result<NedelecOperator2, ReferenceError> {
    validate_regions(mesh)?;
    let mut edge_set = BTreeSet::new();
    for tri in &mesh.triangles {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            edge_set.insert(canonical_edge(a, b));
        }
    }
    let edges: Vec<[usize; 2]> = edge_set.into_iter().map(|(a, b)| [a, b]).collect();
    let edge_index: BTreeMap<(usize, usize), usize> = edges
        .iter()
        .enumerate()
        .map(|(i, e)| ((e[0], e[1]), i))
        .collect();
    let mut curl_entries = vec![];
    let mut mass_entries = vec![];
    for (cell, tri) in mesh.triangles.iter().copied().enumerate() {
        let (area, grad) = triangle_geometry(mesh, cell, tri)?;
        let local = [(0usize, 1usize), (1, 2), (2, 0)];
        let mut global = [0usize; 3];
        let mut sign = [1.0f64; 3];
        for (e, (i, j)) in local.into_iter().enumerate() {
            let a = tri[i];
            let b = tri[j];
            let key = canonical_edge(a, b);
            global[e] = edge_index[&key];
            sign[e] = if (a, b) == key { 1.0 } else { -1.0 };
        }
        for (e, (i, j)) in local.into_iter().enumerate() {
            let curl_e = sign[e] * 2.0 * cross2(grad[i], grad[j]);
            for (f, (k, l)) in local.into_iter().enumerate() {
                let curl_f = sign[f] * 2.0 * cross2(grad[k], grad[l]);
                curl_entries.push((global[e], global[f], area * curl_e * curl_f));

                let m = sign[e]
                    * sign[f]
                    * (lambda_integral(area, i, k) * dot2(grad[j], grad[l])
                        - lambda_integral(area, i, l) * dot2(grad[j], grad[k])
                        - lambda_integral(area, j, k) * dot2(grad[i], grad[l])
                        + lambda_integral(area, j, l) * dot2(grad[i], grad[k]));
                mass_entries.push((global[e], global[f], m));
            }
        }
    }
    let n = edges.len();
    Ok(NedelecOperator2 {
        edges,
        curl_curl: SparseMatrix::from_coo(n, n, curl_entries),
        mass: SparseMatrix::from_coo(n, n, mass_entries),
    })
}

fn validate_regions(mesh: &ReferenceMesh2) -> Result<(), ReferenceError> {
    if !mesh.regions.is_empty() && mesh.regions.len() != mesh.triangles.len() {
        return Err(ReferenceError::RegionCount {
            expected: mesh.triangles.len(),
            got: mesh.regions.len(),
        });
    }
    Ok(())
}

fn triangle_geometry(
    mesh: &ReferenceMesh2,
    cell: usize,
    tri: [usize; 3],
) -> Result<(f64, [[f64; 2]; 3]), ReferenceError> {
    for &v in &tri {
        if v >= mesh.vertices.len() {
            return Err(ReferenceError::BadVertex { cell, vertex: v });
        }
    }
    let p = [
        mesh.vertices[tri[0]],
        mesh.vertices[tri[1]],
        mesh.vertices[tri[2]],
    ];
    let signed =
        (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
    if signed.abs() < 1e-300 {
        return Err(ReferenceError::Degenerate {
            cell,
            signed_area: 0.5 * signed,
        });
    }
    let inv = 1.0 / signed;
    Ok((
        0.5 * signed.abs(),
        [
            [(p[1][1] - p[2][1]) * inv, (p[2][0] - p[1][0]) * inv],
            [(p[2][1] - p[0][1]) * inv, (p[0][0] - p[2][0]) * inv],
            [(p[0][1] - p[1][1]) * inv, (p[1][0] - p[0][0]) * inv],
        ],
    ))
}

fn canonical_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}
fn dot2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn lambda_integral(area: f64, i: usize, j: usize) -> f64 {
    if i == j { area / 6.0 } else { area / 12.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_triangle(order: [usize; 3]) -> ReferenceMesh2 {
        ReferenceMesh2 {
            vertices: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            triangles: vec![order],
            regions: vec!["bulk".into()],
            boundary_edges: vec![],
        }
    }

    fn max_abs(values: &[f64]) -> f64 {
        values.iter().copied().map(f64::abs).fold(0.0, f64::max)
    }

    #[test]
    fn elasticity_has_rigid_translation_null_modes() {
        let op = compile_elasticity_p1_2d(
            &one_triangle([0, 1, 2]),
            IsotropicElasticity2 {
                lambda: 2.0,
                mu: 3.0,
            },
        )
        .unwrap();
        let tx = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let ty = [0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        assert!(max_abs(&op.stiffness.apply(&tx).unwrap()) < 1e-12);
        assert!(max_abs(&op.stiffness.apply(&ty).unwrap()) < 1e-12);
    }

    #[test]
    fn stokes_has_an_exact_zero_pressure_block_and_symmetric_coupling() {
        let op = compile_stokes_p1p1_2d(&one_triangle([0, 1, 2]), 1.0).unwrap();
        for p in 0..op.pressure_dofs {
            for q in 0..op.pressure_dofs {
                assert_eq!(
                    op.saddle.get(op.velocity_dofs + p, op.velocity_dofs + q),
                    0.0
                );
            }
        }
        for r in 0..op.saddle.rows {
            for c in 0..op.saddle.cols {
                assert_eq!(op.saddle.get(r, c), op.saddle.get(c, r));
            }
        }
    }

    #[test]
    fn nedelec_orientation_is_invariant_to_triangle_vertex_order() {
        let a = compile_nedelec0_2d(&one_triangle([0, 1, 2])).unwrap();
        let b = compile_nedelec0_2d(&one_triangle([1, 0, 2])).unwrap();
        assert_eq!(a.edges, b.edges);
        for r in 0..a.edges.len() {
            for c in 0..a.edges.len() {
                assert!((a.curl_curl.get(r, c) - b.curl_curl.get(r, c)).abs() < 1e-12);
                assert!((a.mass.get(r, c) - b.mass.get(r, c)).abs() < 1e-12);
            }
        }
    }
}
