//! Nova fold step verifier.
//!
//! Verifies that a fold was performed correctly given:
//! - The old accumulator
//! - The new instance
//! - The fold proof
//! - The claimed new accumulator

use crate::commitment::CommitmentParams;
use crate::error::{Result, RorahError};
use crate::field::FieldElement;
use crate::nova::{NovaAccumulator, NovaProof};
use crate::r1cs::{RelaxedR1CSInstance, Witness};
use crate::transcript::Transcript;

/// Result of verifying a fold step.
#[derive(Debug)]
pub struct FoldVerificationResult {
    pub is_valid: bool,
    pub challenge: FieldElement,
}

/// Verify a single Nova fold step.
///
/// Given:
/// - Old accumulator (before fold)
/// - New relaxed instance (what was folded in)
/// - The fold proof (commitment to T)
/// - New accumulator (claimed result)
///
/// Checks that new_acc = Fold(old_acc, new_instance) is consistent.
pub fn verify_fold_step(
    old_accumulator: &NovaAccumulator,
    new_instance: &RelaxedR1CSInstance,
    new_instance_witness: &Witness,
    proof: &NovaProof,
    new_accumulator: &NovaAccumulator,
    params: &CommitmentParams,
) -> Result<FoldVerificationResult> {
    // Step 1: Regenerate the Fiat-Shamir challenge
    let challenge = regenerate_challenge(old_accumulator, new_instance, proof)?;

    // Step 2: Check that folded public inputs are consistent
    verify_public_input_fold(
        old_accumulator,
        new_instance,
        new_accumulator,
        challenge,
    )?;

    // Step 3: Check folded u is consistent
    verify_u_fold(old_accumulator, new_instance, new_accumulator, challenge)?;

    // Step 4: Verify the new accumulator satisfies its own relaxed R1CS
    new_accumulator.is_valid()?;

    Ok(FoldVerificationResult {
        is_valid: true,
        challenge,
    })
}

/// Regenerate the Fiat-Shamir challenge from public data.
fn regenerate_challenge(
    old_accumulator: &NovaAccumulator,
    new_instance: &RelaxedR1CSInstance,
    proof: &NovaProof,
) -> Result<FieldElement> {
    let mut transcript = Transcript::new(b"RORAH_NOVA_FOLD_V1");

    transcript.absorb_field_vec(
        b"acc_public_inputs",
        old_accumulator.public_inputs(),
    );
    transcript.absorb_field(b"acc_u", &old_accumulator.instance.u);
    transcript.absorb_field_vec(
        b"new_public_inputs",
        &new_instance.public_inputs,
    );
    transcript.absorb_field(b"new_u", &new_instance.u);
    transcript.absorb(
        b"cross_term_commitment",
        &proof.cross_term_commitment.to_bytes(),
    );

    transcript.squeeze(b"folding_challenge")
}

/// Verify folded public inputs: x' = x₁ + r·x₂
fn verify_public_input_fold(
    old_accumulator: &NovaAccumulator,
    new_instance: &RelaxedR1CSInstance,
    new_accumulator: &NovaAccumulator,
    challenge: FieldElement,
) -> Result<()> {
    let old_x = old_accumulator.public_inputs();
    let new_x = &new_instance.public_inputs;
    let folded_x = new_accumulator.public_inputs();

    let max_len = old_x.len().max(new_x.len());

    for i in 0..max_len {
        let x1 = old_x.get(i).copied().unwrap_or_else(FieldElement::zero);
        let x2 = new_x.get(i).copied().unwrap_or_else(FieldElement::zero);
        let expected = x1 + challenge * x2;

        let actual = folded_x.get(i).copied().unwrap_or_else(FieldElement::zero);

        if expected != actual {
            return Err(RorahError::InvalidProof {
                reason: format!("Public input {} fold is incorrect", i),
            });
        }
    }

    Ok(())
}

/// Verify folded u: u' = u₁ + r·u₂
fn verify_u_fold(
    old_accumulator: &NovaAccumulator,
    new_instance: &RelaxedR1CSInstance,
    new_accumulator: &NovaAccumulator,
    challenge: FieldElement,
) -> Result<()> {
    let expected_u = old_accumulator.instance.u + challenge * new_instance.u;
    let actual_u = new_accumulator.instance.u;

    if expected_u != actual_u {
        return Err(RorahError::InvalidProof {
            reason: format!(
                "Relaxation factor u fold is incorrect: expected {:?}, got {:?}",
                expected_u, actual_u
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::CommitmentParams;
    use crate::fold_instances;
    use crate::r1cs::constraint::{Constraint, LinearCombination};
    use crate::r1cs::{R1CSInstance, RelaxedR1CSInstance, Witness};
    use crate::FieldElement;

    fn square_instance_and_witness(x: u64) -> (R1CSInstance, Witness) {
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
    fn test_verify_valid_fold() {
        let params = CommitmentParams::new(32);

        let old_acc = NovaAccumulator::empty(3).unwrap();
        let (instance, witness) = square_instance_and_witness(5);

        let relaxed = RelaxedR1CSInstance::from_r1cs(instance.clone());
        let (new_acc, proof) = fold_instances(old_acc.clone(), instance, witness.clone(), &params).unwrap();

        let result = verify_fold_step(
            &old_acc,
            &relaxed,
            &witness,
            &proof,
            &new_acc,
            &params,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_valid);
    }

    #[test]
    fn test_verify_catches_wrong_accumulator() {
        let params = CommitmentParams::new(32);

        let old_acc = NovaAccumulator::empty(3).unwrap();
        let (instance, witness) = square_instance_and_witness(5);

        let relaxed = RelaxedR1CSInstance::from_r1cs(instance.clone());
        let (_, proof) = fold_instances(old_acc.clone(), instance, witness.clone(), &params).unwrap();

        // Create a different accumulator (wrong)
        let (fake_acc, _) = {
            let acc2 = NovaAccumulator::empty(3).unwrap();
            let (i2, w2) = square_instance_and_witness(7); // Different value
            fold_instances(acc2, i2, w2, &params).unwrap()
        };

        // Verifying with wrong accumulator should fail
        let result = verify_fold_step(
            &old_acc,
            &relaxed,
            &witness,
            &proof,
            &fake_acc,
            &params,
        );

        assert!(result.is_err());
    }
}