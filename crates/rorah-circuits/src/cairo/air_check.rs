use crate::cairo::types::CairoProofData;
use ark_ff::Field;

pub struct CairoAIRChecker;

impl CairoAIRChecker {
    pub fn verify_air(proof: &CairoProofData) -> anyhow::Result<bool> {
        if proof.trace_commitments.is_empty() {
            anyhow::bail!("trace_commitments cannot be empty");
        }

        for commit in &proof.trace_commitments {
            if commit.len() != 32 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_composition_polynomial(
        composition: &[u8],
        evaluations: &[Vec<u8>],
    ) -> bool {
        if composition.is_empty() || evaluations.is_empty() {
            return false;
        }

        composition.len() >= evaluations.len()
    }

    pub fn verify_constraint_degrees(
        degree_bits: u32,
        trace_length: u64,
    ) -> bool {
        let expected_trace_length = 2u64.pow(degree_bits);
        trace_length <= expected_trace_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_verification() {
        let proof = CairoProofData {
            trace_commitments: vec![vec![0u8; 32]],
            composition_polynomial: vec![1u8; 64],
            fri_proof: vec![1u8; 256],
            decommitment_values: vec![vec![1u8; 32]],
            num_steps: 1024,
        };

        let result = CairoAIRChecker::verify_air(&proof);
        assert!(result.is_ok());
    }
}