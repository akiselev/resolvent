use super::P1Error;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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
    pub(super) fn from_triplets(
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

        // `sort_by` is stable. Duplicate element contributions to one matrix entry therefore
        // remain in assembly insertion order, making floating-point accumulation deterministic.
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
            while cursor < triplets.len() && triplets[cursor].0 == row && triplets[cursor].1 == col
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
