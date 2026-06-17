use crate::common::merkle::MerkleProof;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct FRIProof {
    pub layers: Vec<FRILayer>,
    pub final_value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FRILayer {
    pub merkle_proof: MerkleProof,
    pub evaluations: Vec<Vec<u8>>,
    pub folding_factor: u32,
}

impl FRIProof {
    pub fn new(layers: Vec<FRILayer>, final_value: Vec<u8>) -> Self {
        FRIProof { layers, final_value }
    }

    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    pub fn query_index(&self) -> u64 {
        if self.layers.is_empty() {
            0
        } else {
            self.layers[0].merkle_proof.leaf_index()
        }
    }
}

pub struct FRIVerifier;

impl FRIVerifier {
    pub fn verify_layer(layer: &FRILayer, commitment: &[u8]) -> anyhow::Result<bool> {
        let merkle_valid = layer.merkle_proof.verify(commitment)?;

        if !merkle_valid {
            return Ok(false);
        }

        if layer.evaluations.is_empty() {
            anyhow::bail!("evaluations cannot be empty");
        }

        let expected_evaluations = 1 << (layer.folding_factor - 1);
        if layer.evaluations.len() != expected_evaluations {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn verify_full_proof(
        proof: &FRIProof,
        initial_commitment: &[u8],
    ) -> anyhow::Result<bool> {
        if proof.layers.is_empty() {
            anyhow::bail!("FRI proof must have at least one layer");
        }

        let mut current_commitment = initial_commitment.to_vec();

        for layer in &proof.layers {
            if !Self::verify_layer(layer, &current_commitment)? {
                return Ok(false);
            }

            current_commitment = Self::fold_commitment(&current_commitment, &layer.evaluations)?;
        }

        Ok(true)
    }

    fn fold_commitment(
        commitment: &[u8],
        evaluations: &[Vec<u8>],
    ) -> anyhow::Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(commitment);

        for eval in evaluations {
            hasher.update(eval);
        }

        Ok(hasher.finalize().to_vec())
    }

    pub fn compute_folded_commitment(
        original: &[u8],
        folding_factor: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let mut result = original.to_vec();

        for _ in 0..folding_factor {
            let mut hasher = Sha256::new();
            hasher.update(&result);
            result = hasher.finalize().to_vec();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fri_layer_validation() {
        let leaf = b"leaf".to_vec();
        let path = vec![b"sibling".to_vec()];
        let merkle_proof = MerkleProof::new(leaf, path, 0);

        let layer = FRILayer {
            merkle_proof,
            evaluations: vec![vec![1u8; 32], vec![2u8; 32]],
            folding_factor: 2,
        };

        let commitment = b"commitment";
        let result = FRIVerifier::verify_layer(&layer, commitment);
        
        assert!(result.is_ok());
    }
}