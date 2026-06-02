//! Witness vector for R1CS instances.

//! Witness vector for R1CS instances.

use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _; // Import trait for methods
use serde::{Deserialize, Serialize};


/// Witness vector containing variable assignments.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    variables: Vec<FieldElement>,
    public_len: usize,
}

impl Witness {
    /// Create a new witness.
    pub fn new(variables: Vec<FieldElement>, public_len: usize) -> Result<Self> {
        if variables.is_empty() {
            return Err(RorahError::WitnessSizeMismatch {
                expected: 1,
                actual: 0,
            });
        }

        if variables[0] != FieldElement::one() {
            return Err(RorahError::FieldError(
                "First witness element must be 1".to_string(),
            ));
        }

        if public_len > variables.len() {
            return Err(RorahError::WitnessSizeMismatch {
                expected: variables.len(),
                actual: public_len,
            });
        }

        Ok(Self {
            variables,
            public_len,
        })
    }

    /// Get all variables.
    pub fn variables(&self) -> &[FieldElement] {
        &self.variables
    }

    /// Get public inputs (excluding constant 1).
    pub fn public_inputs(&self) -> &[FieldElement] {
        &self.variables[1..self.public_len]
    }

    /// Get private witness.
    pub fn private_witness(&self) -> &[FieldElement] {
        &self.variables[self.public_len..]
    }

    /// Total number of variables.
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if witness is empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Get variable at index.
    pub fn get(&self, index: usize) -> Option<FieldElement> {
        self.variables.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_creation() {
        let vars = vec![
            FieldElement::one(),
            FieldElement::from_u64(10),
            FieldElement::from_u64(20),
            FieldElement::from_u64(30),
        ];

        let witness = Witness::new(vars.clone(), 2).unwrap();

        assert_eq!(witness.len(), 4);
        assert_eq!(witness.public_inputs(), &[FieldElement::from_u64(10)]);
        assert_eq!(
            witness.private_witness(),
            &[FieldElement::from_u64(20), FieldElement::from_u64(30)]
        );
    }

    #[test]
    fn test_witness_must_start_with_one() {
        let vars = vec![FieldElement::from_u64(0)];
        let result = Witness::new(vars, 1);
        assert!(result.is_err());
    }
}