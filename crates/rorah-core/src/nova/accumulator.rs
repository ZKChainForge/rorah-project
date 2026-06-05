//! Nova accumulator — running state during folding.

//! Nova accumulator — running state during folding.

use crate::error::Result;
use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _; // Import trait for methods
use crate::r1cs::{RelaxedR1CSInstance, Witness};
use serde::{Deserialize, Serialize};


/// The Nova accumulator holds the current relaxed R1CS instance
/// and its satisfying witness after folding N instances.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovaAccumulator {
    pub instance: RelaxedR1CSInstance,
    pub witness: Witness,
}

impl NovaAccumulator {
    /// Create accumulator from a relaxed instance and witness.
    pub fn new(instance: RelaxedR1CSInstance, witness: Witness) -> Result<Self> {
        instance.is_satisfied(&witness)?;
        Ok(Self { instance, witness })
    }

    /// Create an empty accumulator (starting state before any folds).
    pub fn empty(num_variables: usize) -> Result<Self> {
        let instance = RelaxedR1CSInstance::empty(num_variables);

        let mut vars = vec![FieldElement::zero(); num_variables];
        if num_variables > 0 {
            vars[0] = FieldElement::one();
        }
        let witness = Witness::new(vars, 1)?;

        Ok(Self { instance, witness })
    }

    /// Check the accumulator satisfies its own relaxed R1CS.
    pub fn is_valid(&self) -> Result<()> {
        self.instance.is_satisfied(&self.witness)
    }

    /// True only for the initial empty accumulator.
    pub fn is_empty(&self) -> bool {
        self.instance.num_constraints == 0 && self.instance.u.is_zero()
    }

    /// Current relaxation factor u.
    pub fn relaxation_factor(&self) -> FieldElement {
        self.instance.u
    }

    /// Current error vector E.
    pub fn error_vector(&self) -> &[FieldElement] {
        &self.instance.error_vector
    }

    /// Public inputs of the folded instance.
    pub fn public_inputs(&self) -> &[FieldElement] {
        &self.instance.public_inputs
    }

    /// Number of constraints in the accumulated instance.
    pub fn num_constraints(&self) -> usize {
        self.instance.num_constraints
    }

    /// Number of variables in the accumulated instance.
    pub fn num_variables(&self) -> usize {
        self.instance.num_variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r1cs::constraint::{Constraint, LinearCombination};
    use crate::r1cs::R1CSInstance;

    #[test]
    fn test_empty_accumulator() {
        let acc = NovaAccumulator::empty(5).unwrap();

        assert!(acc.is_empty());
        assert!(acc.is_valid().is_ok());
        assert_eq!(acc.num_constraints(), 0);
        assert_eq!(acc.num_variables(), 5);
        assert!(acc.relaxation_factor().is_zero());
    }

    #[test]
    fn test_accumulator_from_valid_instance() {
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());

        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());

        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());

        let constraint = Constraint::new(a, b, c);
        let public_inputs = vec![FieldElement::from_u64(5)];
        let r1cs = R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(5),
                FieldElement::from_u64(25),
            ],
            2,
        )
        .unwrap();

        let relaxed = RelaxedR1CSInstance::from_r1cs(r1cs);
        let acc = NovaAccumulator::new(relaxed, witness).unwrap();

        assert!(!acc.is_empty());
        assert!(acc.is_valid().is_ok());
        assert_eq!(acc.num_constraints(), 1);
        assert_eq!(acc.relaxation_factor(), FieldElement::one());
    }
}