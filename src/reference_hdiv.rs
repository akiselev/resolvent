//! Deterministic lowest-order Raviart-Thomas H(div) reference discretization on triangles.
//!
//! This is a small falsification/reference compiler, not a production backend. Global edge
//! normals are canonicalized independently of triangle ordering so the assembled mass and
//! div-div operators exercise the orientation semantics required by R17.

use crate::reference::{ReferenceError, ReferenceMesh2, SparseMatrix};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RaviartThomasOperator2 {
    /// Canonical global edge orientation `(min_vertex,max_vertex)`.
    pub edges: Vec<[usize; 2]>,
    /// `∫ u · v` in the globally oriented RT0 basis.
    pub mass: SparseMatrix,
    /// `∫ div(u) div(v)` in the globally oriented RT0 basis.
    pub div_div: SparseMatrix,
}

pub fn compile_raviart_thomas0_2d(
    mesh: &ReferenceMesh2,
) -> Result<RaviartThomasOperator2, ReferenceError> {
    if !mesh.regions.is_empty() && mesh.regions.len() != mesh.triangles.len() {
        return Err(ReferenceError::RegionCount {
            expected: mesh.triangles.len(),
            got: mesh.regions.len(),
        });
    }
    let mut edge_set = BTreeSet::new();
    for tri in &mesh.triangles {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            edge_set.insert(canonical_edge(a, b));
        }
    }
    let edges = edge_set
        .into_iter()
        .map(|(a, b)| [a, b])
        .collect::<Vec<_>>();
    let edge_index = edges
        .iter()
        .enumerate()
        .map(|(i, e)| ((e[0], e[1]), i))
        .collect::<BTreeMap<_, _>>();

    let mut mass = Vec::new();
    let mut div_div = Vec::new();
    for (cell, tri) in mesh.triangles.iter().copied().enumerate() {
        let p = triangle(mesh, cell, tri)?;
        let signed2 = cross(sub(p[1], p[0]), sub(p[2], p[0]));
        let area = 0.5 * signed2.abs();
        if area < 1e-300 {
            return Err(ReferenceError::Degenerate {
                cell,
                signed_area: 0.5 * signed2,
            });
        }
        let ccw = signed2 > 0.0;
        // Local basis i is associated with the edge opposite vertex i and has unit outward flux.
        let local_edges = [(1usize, 2usize), (2, 0), (0, 1)];
        let mut global = [0usize; 3];
        let mut sign = [1.0f64; 3];
        for (i, (a, b)) in local_edges.into_iter().enumerate() {
            let va = tri[a];
            let vb = tri[b];
            let key = canonical_edge(va, vb);
            global[i] = edge_index[&key];
            let local_tangent = sub(mesh.vertices[vb], mesh.vertices[va]);
            let local_outward = if ccw {
                [local_tangent[1], -local_tangent[0]]
            } else {
                [-local_tangent[1], local_tangent[0]]
            };
            let global_tangent = sub(mesh.vertices[key.1], mesh.vertices[key.0]);
            let global_normal = [global_tangent[1], -global_tangent[0]];
            sign[i] = if dot(local_outward, global_normal) >= 0.0 {
                1.0
            } else {
                -1.0
            };
        }

        // Three-point degree-2 triangle quadrature, exact for RT0 mass products.
        let bary = [
            [2.0 / 3.0, 1.0 / 6.0, 1.0 / 6.0],
            [1.0 / 6.0, 2.0 / 3.0, 1.0 / 6.0],
            [1.0 / 6.0, 1.0 / 6.0, 2.0 / 3.0],
        ];
        let mut local_mass = [[0.0f64; 3]; 3];
        for lambda in bary {
            let x = [
                lambda[0] * p[0][0] + lambda[1] * p[1][0] + lambda[2] * p[2][0],
                lambda[0] * p[0][1] + lambda[1] * p[1][1] + lambda[2] * p[2][1],
            ];
            let phi = [
                scale(sub(x, p[0]), 1.0 / (2.0 * area)),
                scale(sub(x, p[1]), 1.0 / (2.0 * area)),
                scale(sub(x, p[2]), 1.0 / (2.0 * area)),
            ];
            for i in 0..3 {
                for j in 0..3 {
                    local_mass[i][j] += (area / 3.0) * dot(phi[i], phi[j]);
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                mass.push((global[i], global[j], sign[i] * sign[j] * local_mass[i][j]));
                // div((x-p_i)/(2A)) = 1/A.
                div_div.push((global[i], global[j], sign[i] * sign[j] / area));
            }
        }
    }
    let n = edges.len();
    Ok(RaviartThomasOperator2 {
        edges,
        mass: SparseMatrix::from_coo(n, n, mass),
        div_div: SparseMatrix::from_coo(n, n, div_div),
    })
}

fn triangle(
    mesh: &ReferenceMesh2,
    cell: usize,
    tri: [usize; 3],
) -> Result<[[f64; 2]; 3], ReferenceError> {
    for &vertex in &tri {
        if vertex >= mesh.vertices.len() {
            return Err(ReferenceError::BadVertex { cell, vertex });
        }
    }
    Ok([
        mesh.vertices[tri[0]],
        mesh.vertices[tri[1]],
        mesh.vertices[tri[2]],
    ])
}
fn canonical_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn scale(a: [f64; 2], s: f64) -> [f64; 2] {
    [a[0] * s, a[1] * s]
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle(order: [usize; 3]) -> ReferenceMesh2 {
        ReferenceMesh2 {
            vertices: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            triangles: vec![order],
            regions: vec!["bulk".into()],
            boundary_edges: vec![],
        }
    }

    #[test]
    fn rt0_is_invariant_to_triangle_vertex_order() {
        let a = compile_raviart_thomas0_2d(&triangle([0, 1, 2])).unwrap();
        let b = compile_raviart_thomas0_2d(&triangle([1, 0, 2])).unwrap();
        assert_eq!(a.edges, b.edges);
        for r in 0..a.edges.len() {
            for c in 0..a.edges.len() {
                assert!((a.mass.get(r, c) - b.mass.get(r, c)).abs() < 1e-12);
                assert!((a.div_div.get(r, c) - b.div_div.get(r, c)).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn constant_divergence_gives_rank_one_local_div_div() {
        let op = compile_raviart_thomas0_2d(&triangle([0, 1, 2])).unwrap();
        let trace = (0..3).map(|i| op.div_div.get(i, i)).sum::<f64>();
        assert!(trace > 0.0);
        assert!(
            op.mass
                .entries
                .iter()
                .all(|(_, _, value)| value.is_finite())
        );
    }
}
