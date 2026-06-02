//! Sparse matrix representation for R1CS.
//!
//! Optimized for matrices where >99% of entries are zero.

use crate::error::{Result, RorahError};
use crate::field::FieldElement;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sparse matrix stored as (row, col, value) triples.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparseMatrix {
    num_rows: usize,
    num_cols: usize,
    entries: Vec<(usize, usize, FieldElement)>,
    
    // Cache for faster row access
    #[serde(skip)]
    row_index: Option<HashMap<usize, Vec<usize>>>,
}

impl SparseMatrix {
    /// Create a new sparse matrix.
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        Self {
            num_rows,
            num_cols,
            entries: Vec::new(),
            row_index: None,
        }
    }

    /// Add a non-zero entry.
    ///
    /// Security: Validates row and column indices.
    pub fn add_entry(&mut self, row: usize, col: usize, value: FieldElement) -> Result<()> {
        if row >= self.num_rows {
            return Err(RorahError::DimensionMismatch {
                details: format!("Row index {} exceeds matrix rows {}", row, self.num_rows),
            });
        }

        if col >= self.num_cols {
            return Err(RorahError::DimensionMismatch {
                details: format!("Col index {} exceeds matrix cols {}", col, self.num_cols),
            });
        }

        if !value.is_zero() {
            self.entries.push((row, col, value));
            // Invalidate cache
            self.row_index = None;
        }

        Ok(())
    }

    /// Build row index for faster access.
    fn build_row_index(&mut self) {
        let mut index = HashMap::new();

        for (i, &(row, _, _)) in self.entries.iter().enumerate() {
            index.entry(row).or_insert_with(Vec::new).push(i);
        }

        self.row_index = Some(index);
    }

    /// Multiply matrix by vector: result = A * v
    ///
    /// Security: Validates vector length matches matrix columns.
    pub fn multiply_vector(&self, v: &[FieldElement]) -> Result<Vec<FieldElement>> {
        if v.len() != self.num_cols {
            return Err(RorahError::DimensionMismatch {
                details: format!(
                    "Vector length {} does not match matrix columns {}",
                    v.len(),
                    self.num_cols
                ),
            });
        }

        let mut result = vec![FieldElement::zero(); self.num_rows];

        for &(row, col, value) in &self.entries {
            result[row] = result[row] + value * v[col];
        }

        Ok(result)
    }

    /// Get entries for a specific row.
    pub fn get_row_entries(&mut self, row: usize) -> Vec<(usize, FieldElement)> {
        if self.row_index.is_none() {
            self.build_row_index();
        }

        if let Some(ref index) = self.row_index {
            if let Some(indices) = index.get(&row) {
                return indices
                    .iter()
                    .map(|&i| {
                        let (_, col, value) = self.entries[i];
                        (col, value)
                    })
                    .collect();
            }
        }

        Vec::new()
    }

    /// Number of rows.
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    /// Number of columns.
    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    /// Number of non-zero entries.
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    /// Sparsity ratio (percentage of zero entries).
    pub fn sparsity(&self) -> f64 {
        let total = self.num_rows * self.num_cols;
        if total == 0 {
            return 0.0;
        }
        1.0 - (self.entries.len() as f64 / total as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_matrix_creation() {
        let matrix = SparseMatrix::new(10, 20);
        assert_eq!(matrix.num_rows(), 10);
        assert_eq!(matrix.num_cols(), 20);
        assert_eq!(matrix.num_entries(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut matrix = SparseMatrix::new(3, 3);
        
        matrix.add_entry(0, 0, FieldElement::from_u64(1)).unwrap();
        matrix.add_entry(1, 1, FieldElement::from_u64(2)).unwrap();
        matrix.add_entry(2, 2, FieldElement::from_u64(3)).unwrap();

        assert_eq!(matrix.num_entries(), 3);
    }

    #[test]
    fn test_zero_entries_not_stored() {
        let mut matrix = SparseMatrix::new(3, 3);
        
        matrix.add_entry(0, 0, FieldElement::zero()).unwrap();
        matrix.add_entry(1, 1, FieldElement::from_u64(5)).unwrap();

        assert_eq!(matrix.num_entries(), 1);
    }

    #[test]
    fn test_multiply_vector() {
        let mut matrix = SparseMatrix::new(2, 3);
        
        // Row 0: 1*col0 + 2*col1
        matrix.add_entry(0, 0, FieldElement::from_u64(1)).unwrap();
        matrix.add_entry(0, 1, FieldElement::from_u64(2)).unwrap();
        
        // Row 1: 3*col2
        matrix.add_entry(1, 2, FieldElement::from_u64(3)).unwrap();

        let vector = vec![
            FieldElement::from_u64(10),
            FieldElement::from_u64(20),
            FieldElement::from_u64(30),
        ];

        let result = matrix.multiply_vector(&vector).unwrap();

        // Row 0: 1*10 + 2*20 = 50
        assert_eq!(result[0], FieldElement::from_u64(50));
        // Row 1: 3*30 = 90
        assert_eq!(result[1], FieldElement::from_u64(90));
    }

    #[test]
    fn test_sparsity() {
        let mut matrix = SparseMatrix::new(100, 100);
        
        // Add 50 entries out of 10,000
        for i in 0..50 {
            matrix.add_entry(i, i, FieldElement::from_u64(1)).unwrap();
        }

        let sparsity = matrix.sparsity();
        assert!((sparsity - 0.995).abs() < 0.001); // 99.5% sparse
    }
}