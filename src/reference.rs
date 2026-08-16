use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SparseEntry { pub row: usize, pub col: usize, pub value: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SparseMatrix { pub rows: usize, pub cols: usize, pub entries: Vec<SparseEntry> }

impl SparseMatrix {
    pub fn from_triplets(rows: usize, cols: usize, triplets: impl IntoIterator<Item = SparseEntry>) -> Self {
        let mut merged = BTreeMap::<(usize, usize), f64>::new();
        for entry in triplets { *merged.entry((entry.row, entry.col)).or_default() += entry.value; }
        let entries = merged.into_iter().filter_map(|((row,col),value)| (value != 0.0).then_some(SparseEntry { row, col, value })).collect();
        Self { rows, cols, entries }
    }
    pub fn zeros(rows: usize, cols: usize) -> Self { Self { rows, cols, entries: Vec::new() } }
    pub fn apply(&self, x: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if x.len() != self.cols { return Err(ReferenceError::Dimension { expected: self.cols, got: x.len() }); }
        let mut out = vec![0.0; self.rows]; for e in &self.entries { out[e.row] += e.value * x[e.col]; } Ok(out)
    }
    pub fn transpose_apply(&self, x: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if x.len() != self.rows { return Err(ReferenceError::Dimension { expected: self.rows, got: x.len() }); }
        let mut out = vec![0.0; self.cols]; for e in &self.entries { out[e.col] += e.value * x[e.row]; } Ok(out)
    }
    pub fn scaled_add(&self, a: f64, rhs: &Self, b: f64) -> Result<Self, ReferenceError> {
        if self.rows != rhs.rows || self.cols != rhs.cols { return Err(ReferenceError::MatrixShape); }
        Ok(Self::from_triplets(self.rows, self.cols, self.entries.iter().map(|e| SparseEntry { row: e.row, col: e.col, value: a * e.value }).chain(rhs.entries.iter().map(|e| SparseEntry { row: e.row, col: e.col, value: b * e.value }))))
    }
    pub fn diagonal(&self) -> Vec<f64> { let mut d = vec![0.0; self.rows.min(self.cols)]; for e in &self.entries { if e.row == e.col && e.row < d.len() { d[e.row] += e.value; } } d }
    pub fn is_symmetric(&self, tolerance: f64) -> bool {
        if self.rows != self.cols { return false; }
        let map: BTreeMap<(usize,usize),f64> = self.entries.iter().map(|e| ((e.row,e.col),e.value)).collect();
        map.iter().all(|(&(r,c), &v)| (v - map.get(&(c,r)).copied().unwrap_or(0.0)).abs() <= tolerance)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriangleCell { pub vertices: [usize; 3], pub region: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundaryEdge { pub vertices: [usize; 2], pub tag: u32 }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriangleMesh { pub vertices: Vec<[f64; 2]>, pub cells: Vec<TriangleCell>, #[serde(default)] pub boundaries: Vec<BoundaryEdge> }

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TetCell { pub vertices: [usize; 4], pub region: u32 }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TetMesh { pub vertices: Vec<[f64; 3]>, pub cells: Vec<TetCell> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PiecewiseConstant { pub background: f64, #[serde(default)] pub per_region: BTreeMap<u32, f64> }
impl PiecewiseConstant { pub fn uniform(value: f64) -> Self { Self { background: value, per_region: BTreeMap::new() } } pub fn at(&self, region: u32) -> f64 { self.per_region.get(&region).copied().unwrap_or(self.background) } }

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirichletValue { pub boundary: u32, pub value: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeumannValue { pub boundary: u32, pub flux: f64 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofLayout { pub free_of_vertex: Vec<Option<usize>>, pub prescribed: Vec<Option<f64>>, pub free_vertices: Vec<usize> }
impl DofLayout {
    pub fn from_triangle_mesh(mesh: &TriangleMesh, values: &[DirichletValue]) -> Result<Self, ReferenceError> {
        let by_tag: BTreeMap<u32,f64> = values.iter().map(|v| (v.boundary,v.value)).collect();
        let mut prescribed = vec![None; mesh.vertices.len()];
        for edge in &mesh.boundaries {
            if let Some(&value) = by_tag.get(&edge.tag) {
                for &vertex in &edge.vertices {
                    if vertex >= prescribed.len() { return Err(ReferenceError::BadVertex(vertex)); }
                    match prescribed[vertex] {
                        Some(old) if old.to_bits() != value.to_bits() => return Err(ReferenceError::ConflictingDirichlet { vertex, first: old, second: value }),
                        _ => prescribed[vertex] = Some(value),
                    }
                }
            }
        }
        Ok(Self::from_prescribed(prescribed))
    }
    pub fn unconstrained(n: usize) -> Self { Self::from_prescribed(vec![None; n]) }
    fn from_prescribed(prescribed: Vec<Option<f64>>) -> Self {
        let mut free_of_vertex = vec![None; prescribed.len()]; let mut free_vertices = Vec::new();
        for (vertex, value) in prescribed.iter().enumerate() { if value.is_none() { free_of_vertex[vertex] = Some(free_vertices.len()); free_vertices.push(vertex); } }
        Self { free_of_vertex, prescribed, free_vertices }
    }
    pub fn n_free(&self) -> usize { self.free_vertices.len() }
    pub fn scatter(&self, free: &[f64]) -> Result<Vec<f64>, ReferenceError> {
        if free.len() != self.n_free() { return Err(ReferenceError::Dimension { expected: self.n_free(), got: free.len() }); }
        Ok(self.prescribed.iter().enumerate().map(|(v,p)| p.unwrap_or_else(|| free[self.free_of_vertex[v].unwrap()])).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssembledScalarOperator {
    pub dofs: DofLayout,
    pub stiffness_full: SparseMatrix,
    pub stiffness_free: SparseMatrix,
    pub rhs_full: Vec<f64>,
    pub rhs_free: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssembledMass { pub full: SparseMatrix, pub free: SparseMatrix, pub lumped: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarEllipticProblem2d {
    pub mesh: TriangleMesh,
    pub coefficient: PiecewiseConstant,
    #[serde(default)] pub source: Option<PiecewiseConstant>,
    #[serde(default)] pub dirichlet: Vec<DirichletValue>,
    #[serde(default)] pub neumann: Vec<NeumannValue>,
}

pub fn assemble_scalar_elliptic_p1(problem: &ScalarEllipticProblem2d) -> Result<AssembledScalarOperator, ReferenceError> {
    let mesh = &problem.mesh; let dofs = DofLayout::from_triangle_mesh(mesh, &problem.dirichlet)?; let mut triplets = Vec::new(); let mut rhs_full = vec![0.0; mesh.vertices.len()];
    for (cell_index, cell) in mesh.cells.iter().enumerate() {
        let points = triangle_points(mesh, cell, cell_index)?; let (area, gradients) = triangle_geometry(cell_index, &points)?; let coefficient = problem.coefficient.at(cell.region);
        if !coefficient.is_finite() { return Err(ReferenceError::NonFiniteCoefficient { region: cell.region }); }
        for a in 0..3 { for b in 0..3 { let dot = gradients[a][0]*gradients[b][0] + gradients[a][1]*gradients[b][1]; triplets.push(SparseEntry { row: cell.vertices[a], col: cell.vertices[b], value: coefficient * area * dot }); } }
        if let Some(source) = &problem.source { let load = source.at(cell.region) * area / 3.0; for &vertex in &cell.vertices { rhs_full[vertex] += load; } }
    }
    let flux_by_tag: BTreeMap<u32,f64> = problem.neumann.iter().map(|n| (n.boundary,n.flux)).collect();
    for edge in &mesh.boundaries { if let Some(&flux) = flux_by_tag.get(&edge.tag) { let a = mesh.vertices.get(edge.vertices[0]).ok_or(ReferenceError::BadVertex(edge.vertices[0]))?; let b = mesh.vertices.get(edge.vertices[1]).ok_or(ReferenceError::BadVertex(edge.vertices[1]))?; let length = ((b[0]-a[0]).powi(2)+(b[1]-a[1]).powi(2)).sqrt(); let each = flux * length / 2.0; rhs_full[edge.vertices[0]] += each; rhs_full[edge.vertices[1]] += each; } }
    let stiffness_full = SparseMatrix::from_triplets(mesh.vertices.len(), mesh.vertices.len(), triplets);
    let (stiffness_free, rhs_free) = condense(&stiffness_full, &rhs_full, &dofs)?;
    Ok(AssembledScalarOperator { dofs, stiffness_full, stiffness_free, rhs_full, rhs_free })
}

pub fn assemble_mass_p1(mesh: &TriangleMesh, coefficient: &PiecewiseConstant, dofs: &DofLayout, lumped: bool) -> Result<AssembledMass, ReferenceError> {
    let mut triplets = Vec::new();
    for (cell_index, cell) in mesh.cells.iter().enumerate() {
        let points = triangle_points(mesh, cell, cell_index)?; let (area, _) = triangle_geometry(cell_index, &points)?; let c = coefficient.at(cell.region);
        if lumped { for a in 0..3 { triplets.push(SparseEntry { row: cell.vertices[a], col: cell.vertices[a], value: c * area / 3.0 }); } }
        else { for a in 0..3 { for b in 0..3 { triplets.push(SparseEntry { row: cell.vertices[a], col: cell.vertices[b], value: c * area * if a == b { 1.0/6.0 } else { 1.0/12.0 } }); } } }
    }
    let full = SparseMatrix::from_triplets(mesh.vertices.len(), mesh.vertices.len(), triplets); let free = restrict_free(&full, dofs); Ok(AssembledMass { full, free, lumped })
}

pub fn assemble_tet_diffusion_p1(mesh: &TetMesh, coefficient: &PiecewiseConstant) -> Result<SparseMatrix, ReferenceError> {
    let mut triplets = Vec::new();
    for (cell_index, cell) in mesh.cells.iter().enumerate() {
        let p = [*mesh.vertices.get(cell.vertices[0]).ok_or(ReferenceError::BadVertex(cell.vertices[0]))?, *mesh.vertices.get(cell.vertices[1]).ok_or(ReferenceError::BadVertex(cell.vertices[1]))?, *mesh.vertices.get(cell.vertices[2]).ok_or(ReferenceError::BadVertex(cell.vertices[2]))?, *mesh.vertices.get(cell.vertices[3]).ok_or(ReferenceError::BadVertex(cell.vertices[3]))?];
        let (volume, gradients) = tet_geometry(cell_index, &p)?; let c = coefficient.at(cell.region);
        for a in 0..4 { for b in 0..4 { let dot = gradients[a][0]*gradients[b][0] + gradients[a][1]*gradients[b][1] + gradients[a][2]*gradients[b][2]; triplets.push(SparseEntry { row: cell.vertices[a], col: cell.vertices[b], value: c*volume*dot }); } }
    }
    Ok(SparseMatrix::from_triplets(mesh.vertices.len(), mesh.vertices.len(), triplets))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvolutionOperator { pub mass: SparseMatrix, pub stiffness: SparseMatrix, pub forcing: Vec<f64> }
impl EvolutionOperator {
    pub fn residual(&self, state: &[f64]) -> Result<Vec<f64>, ReferenceError> { let mut out = self.stiffness.apply(state)?; if out.len() != self.forcing.len() { return Err(ReferenceError::Dimension { expected: out.len(), got: self.forcing.len() }); } for (r,f) in out.iter_mut().zip(&self.forcing) { *r -= f; } Ok(out) }
    pub fn charge(&self, state: &[f64]) -> Result<Vec<f64>, ReferenceError> { self.mass.apply(state) }
    pub fn iteration_matrix(&self, mass_scale: f64, stiffness_scale: f64) -> Result<SparseMatrix, ReferenceError> { self.mass.scaled_add(mass_scale, &self.stiffness, stiffness_scale) }
}

fn condense(full: &SparseMatrix, rhs_full: &[f64], dofs: &DofLayout) -> Result<(SparseMatrix,Vec<f64>), ReferenceError> {
    if full.rows != dofs.prescribed.len() || rhs_full.len() != full.rows { return Err(ReferenceError::MatrixShape); }
    let mut rhs = vec![0.0; dofs.n_free()]; for &vertex in &dofs.free_vertices { rhs[dofs.free_of_vertex[vertex].unwrap()] = rhs_full[vertex]; }
    let mut triplets = Vec::new();
    for e in &full.entries {
        if let Some(row) = dofs.free_of_vertex[e.row] {
            if let Some(col) = dofs.free_of_vertex[e.col] { triplets.push(SparseEntry { row, col, value: e.value }); }
            else if let Some(value) = dofs.prescribed[e.col] { rhs[row] -= e.value * value; }
        }
    }
    Ok((SparseMatrix::from_triplets(dofs.n_free(), dofs.n_free(), triplets), rhs))
}
fn restrict_free(full: &SparseMatrix, dofs: &DofLayout) -> SparseMatrix { SparseMatrix::from_triplets(dofs.n_free(), dofs.n_free(), full.entries.iter().filter_map(|e| Some(SparseEntry { row: dofs.free_of_vertex[e.row]?, col: dofs.free_of_vertex[e.col]?, value: e.value }))) }
fn triangle_points(mesh: &TriangleMesh, cell: &TriangleCell, _cell_index: usize) -> Result<[[f64;2];3], ReferenceError> { Ok([*mesh.vertices.get(cell.vertices[0]).ok_or(ReferenceError::BadVertex(cell.vertices[0]))?, *mesh.vertices.get(cell.vertices[1]).ok_or(ReferenceError::BadVertex(cell.vertices[1]))?, *mesh.vertices.get(cell.vertices[2]).ok_or(ReferenceError::BadVertex(cell.vertices[2]))?]) }
fn triangle_geometry(cell: usize, p: &[[f64;2];3]) -> Result<(f64, [[f64;2];3]), ReferenceError> {
    let signed_2a = (p[1][0]-p[0][0])*(p[2][1]-p[0][1])-(p[2][0]-p[0][0])*(p[1][1]-p[0][1]); if signed_2a.abs() < 1e-300 { return Err(ReferenceError::DegenerateCell(cell)); } let inv = 1.0/signed_2a;
    let g = [[(p[1][1]-p[2][1])*inv,(p[2][0]-p[1][0])*inv],[(p[2][1]-p[0][1])*inv,(p[0][0]-p[2][0])*inv],[(p[0][1]-p[1][1])*inv,(p[1][0]-p[0][0])*inv]]; Ok((0.5*signed_2a.abs(),g))
}
fn tet_geometry(cell: usize, p: &[[f64;3];4]) -> Result<(f64, [[f64;3];4]), ReferenceError> {
    let a = [[p[1][0]-p[0][0],p[2][0]-p[0][0],p[3][0]-p[0][0]],[p[1][1]-p[0][1],p[2][1]-p[0][1],p[3][1]-p[0][1]],[p[1][2]-p[0][2],p[2][2]-p[0][2],p[3][2]-p[0][2]]];
    let det = a[0][0]*(a[1][1]*a[2][2]-a[1][2]*a[2][1])-a[0][1]*(a[1][0]*a[2][2]-a[1][2]*a[2][0])+a[0][2]*(a[1][0]*a[2][1]-a[1][1]*a[2][0]); if det.abs() < 1e-300 { return Err(ReferenceError::DegenerateCell(cell)); }
    let inv = invert3(a, det); let g1 = [inv[0][0],inv[0][1],inv[0][2]]; let g2 = [inv[1][0],inv[1][1],inv[1][2]]; let g3 = [inv[2][0],inv[2][1],inv[2][2]]; let g0 = [-(g1[0]+g2[0]+g3[0]),-(g1[1]+g2[1]+g3[1]),-(g1[2]+g2[2]+g3[2])]; Ok((det.abs()/6.0,[g0,g1,g2,g3]))
}
fn invert3(a: [[f64;3];3], det: f64) -> [[f64;3];3] { let d=1.0/det; [[(a[1][1]*a[2][2]-a[1][2]*a[2][1])*d,(a[0][2]*a[2][1]-a[0][1]*a[2][2])*d,(a[0][1]*a[1][2]-a[0][2]*a[1][1])*d],[(a[1][2]*a[2][0]-a[1][0]*a[2][2])*d,(a[0][0]*a[2][2]-a[0][2]*a[2][0])*d,(a[0][2]*a[1][0]-a[0][0]*a[1][2])*d],[(a[1][0]*a[2][1]-a[1][1]*a[2][0])*d,(a[0][1]*a[2][0]-a[0][0]*a[2][1])*d,(a[0][0]*a[1][1]-a[0][1]*a[1][0])*d]] }

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ReferenceError {
    #[error("dimension mismatch: expected {expected}, got {got}")] Dimension { expected: usize, got: usize },
    #[error("matrix shapes are incompatible")] MatrixShape,
    #[error("mesh references nonexistent vertex {0}")] BadVertex(usize),
    #[error("degenerate simplex cell {0}")] DegenerateCell(usize),
    #[error("conflicting Dirichlet values at vertex {vertex}: {first} vs {second}")] ConflictingDirichlet { vertex: usize, first: f64, second: f64 },
    #[error("non-finite coefficient for region {region}")] NonFiniteCoefficient { region: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mesh() -> TriangleMesh { TriangleMesh { vertices: vec![[0.,0.],[1.,0.],[0.,1.]], cells: vec![TriangleCell { vertices:[0,1,2], region:0 }], boundaries: vec![BoundaryEdge{vertices:[0,2],tag:1},BoundaryEdge{vertices:[0,1],tag:1},BoundaryEdge{vertices:[1,2],tag:2}] } }
    #[test] fn p1_diffusion_is_symmetric() { let p = ScalarEllipticProblem2d { mesh:mesh(), coefficient:PiecewiseConstant::uniform(2.0), source:None, dirichlet:vec![], neumann:vec![] }; let a=assemble_scalar_elliptic_p1(&p).unwrap(); assert!(a.stiffness_full.is_symmetric(0.0)); assert_eq!(a.stiffness_full.entries.len(),7); }
    #[test] fn condensation_moves_fixed_column_to_rhs() { let p=ScalarEllipticProblem2d { mesh:mesh(), coefficient:PiecewiseConstant::uniform(1.0), source:None, dirichlet:vec![DirichletValue{boundary:1,value:3.0}], neumann:vec![] }; let a=assemble_scalar_elliptic_p1(&p).unwrap(); assert_eq!(a.dofs.n_free(),1); assert_eq!(a.rhs_free.len(),1); }
    #[test] fn mass_conserves_total_measure() { let m=mesh(); let d=DofLayout::unconstrained(3); let a=assemble_mass_p1(&m,&PiecewiseConstant::uniform(2.0),&d,false).unwrap(); let sum:f64=a.full.entries.iter().map(|e|e.value).sum(); assert!((sum-1.0).abs()<1e-14); }
    #[test] fn tet_unit_simplex_volume_kernel() { let m=TetMesh { vertices:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]], cells:vec![TetCell{vertices:[0,1,2,3],region:0}] }; let k=assemble_tet_diffusion_p1(&m,&PiecewiseConstant::uniform(1.0)).unwrap(); assert!(k.is_symmetric(1e-15)); }
}
