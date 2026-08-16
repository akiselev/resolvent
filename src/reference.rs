use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Portable mesh snapshot used by the deterministic reference evaluator. Sinbad adapts its
/// mesh contracts into this data without creating a Resolvent -> Sinbad dependency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceMesh2 {
    pub vertices: Vec<[f64; 2]>,
    pub triangles: Vec<[usize; 3]>,
    #[serde(default)]
    pub regions: Vec<String>,
    #[serde(default)]
    pub boundary_edges: Vec<BoundaryEdge>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryEdge {
    pub vertices: [usize; 2],
    pub tag: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PiecewiseConstant {
    pub background: f64,
    #[serde(default)]
    pub per_region: BTreeMap<String, f64>,
}
impl PiecewiseConstant {
    pub fn uniform(value: f64) -> Self {
        Self {
            background: value,
            per_region: BTreeMap::new(),
        }
    }
    pub fn value(&self, region: &str) -> f64 {
        self.per_region
            .get(region)
            .copied()
            .unwrap_or(self.background)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarH1Problem {
    pub diffusion: PiecewiseConstant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<PiecewiseConstant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<PiecewiseConstant>,
    #[serde(default)]
    pub dirichlet: BTreeMap<String, f64>,
    #[serde(default)]
    pub neumann: BTreeMap<String, f64>,
    #[serde(default)]
    pub robin: BTreeMap<String, RobinData>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RobinData {
    pub coefficient: f64,
    pub ambient: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofLayout {
    pub free_of_vertex: Vec<Option<usize>>,
    pub prescribed: Vec<Option<f64>>,
    pub free_vertices: Vec<usize>,
}
impl DofLayout {
    pub fn n_free(&self) -> usize {
        self.free_vertices.len()
    }
    pub fn scatter(&self, free: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if free.len() != self.n_free() {
            return Err(ReferenceError::Dimension {
                expected: self.n_free(),
                got: free.len(),
            });
        }
        let mut full = vec![0.0; self.free_of_vertex.len()];
        for (i, p) in self.prescribed.iter().enumerate() {
            if let Some(v) = p {
                full[i] = *v
            } else if let Some(g) = self.free_of_vertex[i] {
                full[i] = free[g]
            }
        }
        Ok(full)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SparseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<(usize, usize, f64)>,
}
impl SparseMatrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            entries: vec![],
        }
    }
    pub fn from_coo(rows: usize, cols: usize, mut entries: Vec<(usize, usize, f64)>) -> Self {
        entries.sort_by_key(|(r, c, _)| (*r, *c));
        let mut merged: Vec<(usize, usize, f64)> = vec![];
        for (r, c, v) in entries {
            if let Some(last) = merged.last_mut()
                && last.0 == r
                && last.1 == c
            {
                last.2 += v;
                continue;
            }
            merged.push((r, c, v))
        }
        merged.retain(|e| e.2 != 0.0);
        Self {
            rows,
            cols,
            entries: merged,
        }
    }
    pub fn apply(&self, x: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if x.len() != self.cols {
            return Err(ReferenceError::Dimension {
                expected: self.cols,
                got: x.len(),
            });
        }
        let mut y = vec![0.0; self.rows];
        for &(r, c, v) in &self.entries {
            y[r] += v * x[c]
        }
        Ok(y)
    }
    pub fn transpose_apply(&self, x: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if x.len() != self.rows {
            return Err(ReferenceError::Dimension {
                expected: self.rows,
                got: x.len(),
            });
        }
        let mut y = vec![0.0; self.cols];
        for &(r, c, v) in &self.entries {
            y[c] += v * x[r]
        }
        Ok(y)
    }
    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.entries
            .iter()
            .find_map(|&(rr, cc, v)| (rr == r && cc == c).then_some(v))
            .unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceOperator {
    pub dofs: DofLayout,
    pub stiffness: SparseMatrix,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<SparseMatrix>,
    pub rhs: Vec<f64>,
    pub full_stiffness: SparseMatrix,
}
impl ReferenceOperator {
    pub fn residual(&self, u: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        let mut r = self.stiffness.apply(u)?;
        for (v, f) in r.iter_mut().zip(&self.rhs) {
            *v -= *f
        }
        Ok(r)
    }
    pub fn jvp(&self, v: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        self.stiffness.apply(v)
    }
    pub fn vjp(&self, v: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        self.stiffness.transpose_apply(v)
    }
    pub fn shifted(&self, mass_scale: f64, stiffness_scale: f64) -> SparseMatrix {
        let mut e = self
            .stiffness
            .entries
            .iter()
            .map(|&(r, c, v)| (r, c, stiffness_scale * v))
            .collect::<Vec<_>>();
        if let Some(m) = &self.mass {
            e.extend(m.entries.iter().map(|&(r, c, v)| (r, c, mass_scale * v)))
        }
        SparseMatrix::from_coo(self.stiffness.rows, self.stiffness.cols, e)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ReferenceError {
    #[error("triangle {cell} has invalid vertex {vertex}")]
    BadVertex { cell: usize, vertex: usize },
    #[error("triangle {cell} is degenerate (signed area {signed_area:e})")]
    Degenerate { cell: usize, signed_area: f64 },
    #[error("region list has {got} entries for {expected} triangles")]
    RegionCount { expected: usize, got: usize },
    #[error("boundary edge references invalid vertex {0}")]
    BadBoundaryVertex(usize),
    #[error("conflicting Dirichlet values at vertex {vertex}: {first} vs {second}")]
    ConflictingDirichlet {
        vertex: usize,
        first: f64,
        second: f64,
    },
    #[error("dimension mismatch: expected {expected}, got {got}")]
    Dimension { expected: usize, got: usize },
}

pub fn compile_scalar_h1_p1(
    mesh: &ReferenceMesh2,
    p: &ScalarH1Problem,
) -> Result<ReferenceOperator, ReferenceError> {
    if !mesh.regions.is_empty() && mesh.regions.len() != mesh.triangles.len() {
        return Err(ReferenceError::RegionCount {
            expected: mesh.triangles.len(),
            got: mesh.regions.len(),
        });
    }
    let dofs = build_dofs(mesh, &p.dirichlet)?;
    let n = mesh.vertices.len();
    let mut full_k = vec![];
    let mut mass_full = vec![];
    let mut load = vec![0.0; n];
    for (ci, tri) in mesh.triangles.iter().enumerate() {
        let pts = tri_points(mesh, ci, *tri)?;
        let (area, grad) = p1_geometry(ci, &pts)?;
        let region = mesh.regions.get(ci).map(String::as_str).unwrap_or("");
        let k = p.diffusion.value(region);
        for a in 0..3 {
            for b in 0..3 {
                let dot = grad[a][0] * grad[b][0] + grad[a][1] * grad[b][1];
                full_k.push((tri[a], tri[b], k * area * dot));
            }
        }
        if let Some(mass) = &p.mass {
            let c = mass.value(region);
            for a in 0..3 {
                for b in 0..3 {
                    let w = if a == b { 2.0 } else { 1.0 };
                    mass_full.push((tri[a], tri[b], c * area * w / 12.0));
                }
            }
        }
        if let Some(source) = &p.source {
            let s = source.value(region);
            for &v in tri {
                load[v] += s * area / 3.0;
            }
        }
    }
    for edge in &mesh.boundary_edges {
        for &v in &edge.vertices {
            if v >= n {
                return Err(ReferenceError::BadBoundaryVertex(v));
            }
        }
        let a = mesh.vertices[edge.vertices[0]];
        let b = mesh.vertices[edge.vertices[1]];
        let len = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
        if let Some(q) = p.neumann.get(&edge.tag) {
            for &v in &edge.vertices {
                load[v] += q * len / 2.0
            }
        }
        if let Some(r) = p.robin.get(&edge.tag) {
            let local = r.coefficient * len / 6.0;
            full_k.extend([
                (edge.vertices[0], edge.vertices[0], 2.0 * local),
                (edge.vertices[0], edge.vertices[1], local),
                (edge.vertices[1], edge.vertices[0], local),
                (edge.vertices[1], edge.vertices[1], 2.0 * local),
            ]);
            for &v in &edge.vertices {
                load[v] += r.coefficient * r.ambient * len / 2.0
            }
        }
    }
    let full_stiffness = SparseMatrix::from_coo(n, n, full_k);
    let full_mass = (!mass_full.is_empty()).then(|| SparseMatrix::from_coo(n, n, mass_full));
    let (stiffness, rhs) = condense(&full_stiffness, &load, &dofs);
    let mass = full_mass.as_ref().map(|m| condense_matrix(m, &dofs));
    Ok(ReferenceOperator {
        dofs,
        stiffness,
        mass,
        rhs,
        full_stiffness,
    })
}

fn build_dofs(
    mesh: &ReferenceMesh2,
    dirichlet: &BTreeMap<String, f64>,
) -> Result<DofLayout, ReferenceError> {
    let mut prescribed = vec![None; mesh.vertices.len()];
    for edge in &mesh.boundary_edges {
        if let Some(value) = dirichlet.get(&edge.tag) {
            for &v in &edge.vertices {
                if v >= mesh.vertices.len() {
                    return Err(ReferenceError::BadBoundaryVertex(v));
                }
                match prescribed[v] {
                    Some(old) if old != *value => {
                        return Err(ReferenceError::ConflictingDirichlet {
                            vertex: v,
                            first: old,
                            second: *value,
                        });
                    }
                    _ => prescribed[v] = Some(*value),
                }
            }
        }
    }
    let mut free_of_vertex = vec![None; mesh.vertices.len()];
    let mut free_vertices = vec![];
    for (v, p) in prescribed.iter().enumerate() {
        if p.is_none() {
            free_of_vertex[v] = Some(free_vertices.len());
            free_vertices.push(v)
        }
    }
    Ok(DofLayout {
        free_of_vertex,
        prescribed,
        free_vertices,
    })
}
fn condense(k: &SparseMatrix, load: &[f64], d: &DofLayout) -> (SparseMatrix, Vec<f64>) {
    let mut e = vec![];
    let mut rhs = vec![0.0; d.n_free()];
    for (v, &g) in d.free_of_vertex.iter().enumerate() {
        if let Some(g) = g {
            rhs[g] += load[v]
        }
    }
    for &(r, c, v) in &k.entries {
        match (d.free_of_vertex[r], d.free_of_vertex[c]) {
            (Some(gr), Some(gc)) => e.push((gr, gc, v)),
            (Some(gr), None) => rhs[gr] -= v * d.prescribed[c].unwrap_or(0.0),
            _ => {}
        }
    }
    (SparseMatrix::from_coo(d.n_free(), d.n_free(), e), rhs)
}
fn condense_matrix(k: &SparseMatrix, d: &DofLayout) -> SparseMatrix {
    SparseMatrix::from_coo(
        d.n_free(),
        d.n_free(),
        k.entries
            .iter()
            .filter_map(|&(r, c, v)| Some((d.free_of_vertex[r]?, d.free_of_vertex[c]?, v)))
            .collect(),
    )
}
fn tri_points(
    mesh: &ReferenceMesh2,
    cell: usize,
    t: [usize; 3],
) -> Result<[[f64; 2]; 3], ReferenceError> {
    for &v in &t {
        if v >= mesh.vertices.len() {
            return Err(ReferenceError::BadVertex { cell, vertex: v });
        }
    }
    Ok([
        mesh.vertices[t[0]],
        mesh.vertices[t[1]],
        mesh.vertices[t[2]],
    ])
}
fn p1_geometry(cell: usize, p: &[[f64; 2]; 3]) -> Result<(f64, [[f64; 2]; 3]), ReferenceError> {
    let signed =
        (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
    if signed.abs() < 1e-300 {
        return Err(ReferenceError::Degenerate {
            cell,
            signed_area: 0.5 * signed,
        });
    }
    let inv = 1.0 / signed;
    let g = [
        [(p[1][1] - p[2][1]) * inv, (p[2][0] - p[1][0]) * inv],
        [(p[2][1] - p[0][1]) * inv, (p[0][0] - p[2][0]) * inv],
        [(p[0][1] - p[1][1]) * inv, (p[1][0] - p[0][0]) * inv],
    ];
    Ok((0.5 * signed.abs(), g))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mesh() -> ReferenceMesh2 {
        ReferenceMesh2 {
            vertices: vec![[0., 0.], [1., 0.], [0., 1.]],
            triangles: vec![[0, 1, 2]],
            regions: vec!["bulk".into()],
            boundary_edges: vec![
                BoundaryEdge {
                    vertices: [0, 1],
                    tag: "fixed".into(),
                },
                BoundaryEdge {
                    vertices: [1, 2],
                    tag: "flux".into(),
                },
                BoundaryEdge {
                    vertices: [2, 0],
                    tag: "fixed".into(),
                },
            ],
        }
    }
    #[test]
    fn p1_diffusion_mass_source_and_boundary_compile() {
        let mut dir = BTreeMap::new();
        dir.insert("fixed".into(), 0.);
        let mut neu = BTreeMap::new();
        neu.insert("flux".into(), 2.);
        let p = ScalarH1Problem {
            diffusion: PiecewiseConstant::uniform(3.),
            mass: Some(PiecewiseConstant::uniform(4.)),
            source: Some(PiecewiseConstant::uniform(6.)),
            dirichlet: dir,
            neumann: neu,
            robin: BTreeMap::new(),
        };
        let op = compile_scalar_h1_p1(&mesh(), &p).unwrap();
        assert_eq!(op.dofs.n_free(), 1);
        assert!(op.stiffness.get(0, 0) > 0.);
        assert!(op.mass.as_ref().unwrap().get(0, 0) > 0.);
        assert!(op.rhs[0] > 0.);
        assert_eq!(op.jvp(&[2.]).unwrap(), op.vjp(&[2.]).unwrap());
    }
    #[test]
    fn shifted_contains_mass_and_stiffness() {
        let p = ScalarH1Problem {
            diffusion: PiecewiseConstant::uniform(1.),
            mass: Some(PiecewiseConstant::uniform(1.)),
            source: None,
            dirichlet: BTreeMap::new(),
            neumann: BTreeMap::new(),
            robin: BTreeMap::new(),
        };
        let op = compile_scalar_h1_p1(&mesh(), &p).unwrap();
        let s = op.shifted(2., 3.);
        assert_eq!(s.rows, 3);
        assert_eq!(s.cols, 3);
    }
}
