//! R1CS instance representation.

//! R1CS instance representation.

use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _; // Import trait for methods
use crate::r1cs::{Constraint, Witness};
use serde::{Deserialize, Serialize};


/// Sparse matrix representation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparseMatrix {
    num_rows: usize,
    num_cols: usize,
    entries: Vec<(usize, usize, FieldElement)>,
}

impl SparseMatrix {
    /// Create a new sparse matrix.
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        Self {
            num_rows,
            num_cols,
            entries: Vec::new(),
        }
    }

    /// Add a non-zero entry.
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
        }

        Ok(())
    }

    /// Multiply matrix by vector: result = A * v
    pub fn multiply_vector(&self, z: &[FieldElement]) -> Result<Vec<FieldElement>> {
        if z.len() != self.num_cols {
            return Err(RorahError::DimensionMismatch {
                details: format!(
                    "Vector length {} does not match matrix columns {}",
                    z.len(),
                    self.num_cols
                ),
            });
        }

        let mut result = vec![FieldElement::zero(); self.num_rows];

        for &(row, col, value) in &self.entries {
            result[row] = result[row] + value * z[col];
        }

        Ok(result)
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
}

/// Standard R1CS instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct R1CSInstance {
    pub num_constraints: usize,
    pub num_variables: usize,
    pub num_public_inputs: usize,
    pub a_matrix: SparseMatrix,
    pub b_matrix: SparseMatrix,
    pub c_matrix: SparseMatrix,
    pub public_inputs: Vec<FieldElement>,
}

impl R1CSInstance {
    /// Create from constraints.
    pub fn from_constraints(
        constraints: Vec<Constraint>,
        num_variables: usize,
        public_inputs: Vec<FieldElement>,
    ) -> Result<Self> {
        let num_constraints = constraints.len();
        let num_public_inputs = public_inputs.len();

        let mut a_matrix = SparseMatrix::new(num_constraints, num_variables);
        let mut b_matrix = SparseMatrix::new(num_constraints, num_variables);
        let mut c_matrix = SparseMatrix::new(num_constraints, num_variables);

        for (row, constraint) in constraints.iter().enumerate() {
            for (&col, &coeff) in constraint.a.terms() {
                a_matrix.add_entry(row, col, coeff)?;
            }
            for (&col, &coeff) in constraint.b.terms() {
                b_matrix.add_entry(row, col, coeff)?;
            }
            for (&col, &coeff) in constraint.c.terms() {
                c_matrix.add_entry(row, col, coeff)?;
            }
        }

        Ok(Self {
            num_constraints,
            num_variables,
            num_public_inputs,
            a_matrix,
            b_matrix,
            c_matrix,
            public_inputs,
        })
    }

    /// Check if witness satisfies this instance.
    pub fn is_satisfied(&self, witness: &Witness) -> Result<()> {
        if witness.len() != self.num_variables {
            return Err(RorahError::WitnessSizeMismatch {
                expected: self.num_variables,
                actual: witness.len(),
            });
        }

        if witness.public_inputs() != &self.public_inputs[..] {
            return Err(RorahError::PublicInputMismatch { index: 0 });
        }

        let z = witness.variables();

        let az = self.a_matrix.multiply_vector(z)?;
        let bz = self.b_matrix.multiply_vector(z)?;
        let cz = self.c_matrix.multiply_vector(z)?;

        for i in 0..self.num_constraints {
            if az[i] * bz[i] != cz[i] {
                return Err(RorahError::ConstraintNotSatisfied {
                    index: i,
                    reason: format!("constraint {} failed", i),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r1cs::constraint::LinearCombination;

    #[test]
    fn test_sparse_matrix_multiply() {
        let mut matrix = SparseMatrix::new(2, 3);
        matrix
            .add_entry(0, 0, FieldElement::from_u64(1))
            .unwrap();
        matrix
            .add_entry(0, 1, FieldElement::from_u64(2))
            .unwrap();
        matrix
            .add_entry(1, 2, FieldElement::from_u64(3))
            .unwrap();

        let vector = vec![
            FieldElement::from_u64(4),
            FieldElement::from_u64(5),
            FieldElement::from_u64(6),
        ];

        let result = matrix.multiply_vector(&vector).unwrap();

        assert_eq!(result[0], FieldElement::from_u64(14));
        assert_eq!(result[1], FieldElement::from_u64(18));
    }

    #[test]
    fn test_r1cs_simple() {
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());

        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());

        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());

        let constraint = Constraint::new(a, b, c);

        let public_inputs = vec![FieldElement::from_u64(5)];
        let instance =
            R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(5),
                FieldElement::from_u64(25),
            ],
            2,
        )
        .unwrap();

        assert!(instance.is_satisfied(&witness).is_ok());

        let bad_witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(5),
                FieldElement::from_u64(26),
            ],
            2,
        )
        .unwrap();

        assert!(instance.is_satisfied(&bad_witness).is_err());
    }
}