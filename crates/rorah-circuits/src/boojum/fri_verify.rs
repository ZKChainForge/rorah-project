use crate::boojum::types::BoojumProofData;
use crate::boojum::fri_layer::FRILayerVerifier;
use crate::common::hash::sha256;

pub struct BoojumFRIVerifier;

impl BoojumFRIVerifier {
    pub fn verify_fri(proof: &BoojumProofData) -> anyhow::Result<bool> {
        if proof.fri_layers.is_empty() {
            anyhow::bail!("FRI proof must contain at least one layer");
        }

        let mut current_commitment = sha256(&proof.fri_layers[0].merkle_root);

        for (i, layer) in proof.fri_layers.iter().enumerate() {
            if !FRILayerVerifier::verify_layer(layer, &[i as u64])? {
                return Ok(false);
            }

            if i < proof.fri_layers.len() - 1 {
                current_commitment = sha256(&current_commitment);
            }
        }

        let final_layer = &proof.fri_layers[proof.fri_layers.len() - 1];
        if final_layer.depth != 0 {
            anyhow::bail!("Final FRI layer must have depth 0");
        }

        Ok(true)
    }

    pub fn verify_query_phase(
        proof: &BoojumProofData,
        num_queries: usize,
    ) -> anyhow::Result<bool> {
        if proof.fri_layers.is_empty() {
            return Ok(false);
        }

        for layer in &proof.fri_layers {
            if layer.evaluations.len() < num_queries {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_lde(lde_evals: &[Vec<u8>], fri_rate: u32) -> anyhow::Result<bool> {
        if lde_evals.is_empty() {
            anyhow::bail!("LDE evaluations cannot be empty");
        }

        let expected_size_multiplier = fri_rate as usize;

        for eval in lde_evals {
            if eval.is_empty() {
                return Ok(false);
            }
        }

        let first_size = lde_evals[0].len();
        for eval in lde_evals {
            if eval.len() < first_size / expected_size_multiplier {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boojum::types::{FRILayerData, BoojumProofData};

    #[test]
    fn test_fri_verification() {
        let layer = FRILayerData {
            evaluations: vec![vec![1u8; 32]],
            merkle_root: vec![0u8; 32],
            depth: 0,
        };

        let proof = BoojumProofData {
            fri_layers: vec![layer],
            merkle_paths: vec![vec![vec![0u8; 32]]],
            lde_evaluations: vec![vec![1u8; 64]],
            quotient_poly: vec![1u8; 64],
            public_inputs: vec![0u8; 32],
        };

        let result = BoojumFRIVerifier::verify_fri(&proof);
        assert!(result.is_ok());
    }
}