use super::P1Error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const DEGENERATE_AREA: f64 = 1.0e-300;

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
        let vertex_count = self.vertices.len();
        for (cell, triangle) in self.cells.iter().enumerate() {
            for &vertex in &triangle.vertices {
                if vertex >= vertex_count {
                    return Err(P1Error::VertexOutOfRange {
                        owner: format!("cell {cell}"),
                        vertex,
                        vertex_count,
                    });
                }
            }
        }
        for (edge, boundary) in self.boundary_edges.iter().enumerate() {
            for &vertex in &boundary.vertices {
                if vertex >= vertex_count {
                    return Err(P1Error::VertexOutOfRange {
                        owner: format!("boundary edge {edge}"),
                        vertex,
                        vertex_count,
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
    pub fn from_dirichlet(mesh: &P1Mesh, specs: &[DirichletBoundary]) -> Result<Self, P1Error> {
        mesh.validate()?;

        // Mirror Residua's incumbent behavior exactly: the last declaration for a tag wins,
        // then boundary facets apply in mesh order. If two differently-valued tagged facets
        // meet at a vertex, the later facet in the mesh snapshot wins there as well.
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

pub(super) fn p1_shape_gradients(
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
