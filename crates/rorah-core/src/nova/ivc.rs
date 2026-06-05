//! Incremental Verifiable Computation (IVC) chain.
//!
//! Chains multiple fold steps into a complete IVC proof.

use crate::commitment::CommitmentParams;
use crate::error::{Result, RorahError};
use crate::nova::{fold_instances, NovaAccumulator, NovaProof};
use crate::r1cs::{R1CSInstance, Witness};
use serde::{Deserialize, Serialize};

/// A complete IVC chain: accumulator + all fold proofs.
#[derive(Debug, Serialize, Deserialize)]
pub struct IVCProof {
    /// Final accumulator after all folds.
    pub accumulator: NovaAccumulator,
    /// Proofs for each fold step.
    pub fold_proofs: Vec<NovaProof>,
    /// Number of instances folded.
    pub num_instances: usize,
}

impl IVCProof {
    /// Total number of fold steps.
    pub fn num_steps(&self) -> usize {
        self.fold_proofs.len()
    }

    /// Total size of all fold proofs in bytes.
    pub fn total_proof_bytes(&self) -> usize {
        self.fold_proofs.len() * NovaProof::size_bytes()
    }

    /// Check the accumulator is valid.
    pub fn verify_accumulator(&self) -> Result<()> {
        self.accumulator.is_valid()
    }
}

/// Builder for IVC proofs.
pub struct IVCBuilder {
    accumulator: NovaAccumulator,
    fold_proofs: Vec<NovaProof>,
    params: CommitmentParams,
}

impl IVCBuilder {
    /// Create a new IVC builder.
    pub fn new(num_variables: usize, max_generators: usize) -> Result<Self> {
        let accumulator = NovaAccumulator::empty(num_variables)?;
        let params = CommitmentParams::new(max_generators);

        Ok(Self {
            accumulator,
            fold_proofs: Vec::new(),
            params,
        })
    }

    /// Fold in the next instance.
    ///
    /// Returns the fold proof for this step.
    pub fn fold_next(
        &mut self,
        instance: R1CSInstance,
        witness: Witness,
    ) -> Result<&NovaProof> {
        let acc = std::mem::replace(
            &mut self.accumulator,
            NovaAccumulator::empty(1)
                .expect("Empty accumulator creation cannot fail"),
        );

        let (new_acc, proof) = fold_instances(acc, instance, witness, &self.params)?;

        self.accumulator = new_acc;
        self.fold_proofs.push(proof);

        Ok(self.fold_proofs.last().unwrap())
    }

    /// Finalize and produce the IVC proof.
    pub fn finalize(self) -> Result<IVCProof> {
        // Verify final accumulator is valid
        self.accumulator.is_valid()?;

        let num_instances = self.fold_proofs.len();

        Ok(IVCProof {
            accumulator: self.accumulator,
            fold_proofs: self.fold_proofs,
            num_instances,
        })
    }

    /// Get current accumulator state.
    pub fn current_accumulator(&self) -> &NovaAccumulator {
        &self.accumulator
    }

    /// Number of instances folded so far.
    pub fn num_folded(&self) -> usize {
        self.fold_proofs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r1cs::constraint::{Constraint, LinearCombination};
    use crate::FieldElement;

    fn square_instance(x: u64) -> (R1CSInstance, Witness) {
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());
        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());
        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());

        let constraint = Constraint::new(a, b, c);
        let public_inputs = vec![FieldElement::from_u64(x)];
        let instance =
            R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();
        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(x),
                FieldElement::from_u64(x * x),
            ],
            2,
        )
        .unwrap();

        (instance, witness)
    }

    #[test]
    fn test_ivc_builder_single() {
        let mut builder = IVCBuilder::new(3, 32).unwrap();

        let (instance, witness) = square_instance(5);
        builder.fold_next(instance, witness).unwrap();

        let proof = builder.finalize().unwrap();

        assert_eq!(proof.num_steps(), 1);
        assert_eq!(proof.num_instances, 1);
        assert!(proof.verify_accumulator().is_ok());
    }

    #[test]
    fn test_ivc_builder_multiple() {
        let mut builder = IVCBuilder::new(3, 32).unwrap();

        for x in [2u64, 3, 5, 7] {
            let (instance, witness) = square_instance(x);
            builder.fold_next(instance, witness).unwrap();
        }

        let proof = builder.finalize().unwrap();

        assert_eq!(proof.num_steps(), 4);
        assert_eq!(proof.num_instances, 4);
        assert!(proof.verify_accumulator().is_ok());
    }

    #[test]
    fn test_ivc_proof_bytes() {
        let mut builder = IVCBuilder::new(3, 32).unwrap();

        for x in [2u64, 3, 5] {
            let (instance, witness) = square_instance(x);
            builder.fold_next(instance, witness).unwrap();
        }

        let proof = builder.finalize().unwrap();

        // Total proof bytes: 3 steps * 32 bytes each
        assert_eq!(proof.total_proof_bytes(), 3 * 32);
    }
}