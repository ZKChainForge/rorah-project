//! Relaxed R1CS for Nova folding.

//! Relaxed R1CS for Nova folding.

use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _; // Import trait for methods
use crate::r1cs::instance::SparseMatrix;
use crate::r1cs::{R1CSInstance, Witness};
use serde::{Deserialize, Serialize};


/// Relaxed R1CS instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelaxedR1CSInstance {
    pub num_constraints: usize,
    pub num_variables: usize,
    pub num_public_inputs: usize,
    pub a_matrix: SparseMatrix,
    pub b_matrix: SparseMatrix,
    pub c_matrix: SparseMatrix,
    pub public_inputs: Vec<FieldElement>,
    pub u: FieldElement,
    pub error_vector: Vec<FieldElement>,
}

impl RelaxedR1CSInstance {
    /// Create from a standard R1CS instance.
    pub fn from_r1cs(instance: R1CSInstance) -> Self {
        let error_vector = vec![FieldElement::zero(); instance.num_constraints];

        Self {
            num_constraints: instance.num_constraints,
            num_variables: instance.num_variables,
            num_public_inputs: instance.num_public_inputs,
            a_matrix: instance.a_matrix,
            b_matrix: instance.b_matrix,
            c_matrix: instance.c_matrix,
            public_inputs: instance.public_inputs,
            u: FieldElement::one(),
            error_vector,
        }
    }

    /// Create an empty accumulator for initialization.
    pub fn empty(num_variables: usize) -> Self {
        Self {
            num_constraints: 0,
            num_variables,
            num_public_inputs: 0,
            a_matrix: SparseMatrix::new(0, num_variables),
            b_matrix: SparseMatrix::new(0, num_variables),
            c_matrix: SparseMatrix::new(0, num_variables),
            public_inputs: Vec::new(),
            u: FieldElement::zero(),
            error_vector: Vec::new(),
        }
    }

    /// Check if witness satisfies this relaxed instance.
    pub fn is_satisfied(&self, witness: &Witness) -> Result<()> {
        if self.num_constraints == 0 {
            return Ok(());
        }

        if witness.len() != self.num_variables {
            return Err(RorahError::WitnessSizeMismatch {
                expected: self.num_variables,
                actual: witness.len(),
            });
        }

        let z = witness.variables();

        let az = self.a_matrix.multiply_vector(z)?;
        let bz = self.b_matrix.multiply_vector(z)?;
        let cz = self.c_matrix.multiply_vector(z)?;

        for i in 0..self.num_constraints {
            let lhs = az[i] * bz[i];
            let rhs = self.u * cz[i] + self.error_vector[i];

            if lhs != rhs {
                return Err(RorahError::RelaxedR1CSNotSatisfied {
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
    use crate::r1cs::constraint::{Constraint, LinearCombination};

    #[test]
    fn test_relaxed_from_standard() {
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());

        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());

        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());

        let constraint = Constraint::new(a, b, c);

        let public_inputs = vec![FieldElement::from_u64(5)];
        let r1cs = R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

        let relaxed = RelaxedR1CSInstance::from_r1cs(r1cs);

        assert_eq!(relaxed.u, FieldElement::one());
        assert_eq!(relaxed.error_vector.len(), 1);
        assert!(relaxed.error_vector[0].is_zero());

        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(5),
                FieldElement::from_u64(25),
            ],
            2,
        )
        .unwrap();

        assert!(relaxed.is_satisfied(&witness).is_ok());
    }

    #[test]
    fn test_empty_accumulator() {
        let empty = RelaxedR1CSInstance::empty(10);
        assert_eq!(empty.num_constraints, 0);
        assert!(empty.u.is_zero());

        let witness = Witness::new(vec![FieldElement::one(); 10], 1).unwrap();

        assert!(empty.is_satisfied(&witness).is_ok());
    }
}