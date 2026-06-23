use crate::boojum::types::BoojumPublicInputs;

pub struct PublicInputValidator;

impl PublicInputValidator {
    pub fn validate_inputs(inputs: &BoojumPublicInputs) -> anyhow::Result<bool> {
        inputs.validate()?;

        if inputs.new_state_root.len() != 32 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn compute_public_input_hash(inputs: &BoojumPublicInputs) -> Vec<u8> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&inputs.new_state_root);
        hasher.update(inputs.block_number.to_le_bytes());
        hasher.update(inputs.tx_count.to_le_bytes());
        hasher.finalize().to_vec()
    }

    pub fn verify_input_consistency(
        claimed_hash: &[u8],
        computed_hash: &[u8],
    ) -> bool {
        claimed_hash == computed_hash
    }
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_input_validation() {
        let inputs = BoojumPublicInputs::new(vec![0u8; 32], 100, 50);
        let result = PublicInputValidator::validate_inputs(&inputs);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}