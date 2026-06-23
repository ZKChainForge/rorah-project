use crate::boojum::types::FRILayerData;

pub struct FRILayerVerifier;

impl FRILayerVerifier {
    pub fn verify_layer(
        layer: &FRILayerData,
        query_indices: &[u64],
    ) -> anyhow::Result<bool> {
        if layer.evaluations.is_empty() {
            anyhow::bail!("evaluations cannot be empty");
        }

        if query_indices.is_empty() {
            anyhow::bail!("query_indices cannot be empty");
        }

        for &idx in query_indices {
            if (idx as usize) >= layer.evaluations.len() {
                return Ok(false);
            }
        }

        for eval in &layer.evaluations {
            if eval.is_empty() || eval.len() > 1024 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn fold_layer(
        layer: &FRILayerData,
        challenge: &[u8],
    ) -> anyhow::Result<FRILayerData> {
        if layer.evaluations.len() < 2 {
            anyhow::bail!("Cannot fold layer with less than 2 evaluations");
        }

        let mut folded = Vec::new();

        for i in (0..layer.evaluations.len()).step_by(2) {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&layer.evaluations[i]);
            if i + 1 < layer.evaluations.len() {
                hasher.update(&layer.evaluations[i + 1]);
            }
            hasher.update(challenge);

            folded.push(hasher.finalize().to_vec());
        }

        Ok(FRILayerData {
            evaluations: folded,
            merkle_root: layer.merkle_root.clone(),
            depth: layer.depth - 1,
        })
    }

    pub fn consistency_check(
        prev_layer: &FRILayerData,
        next_layer: &FRILayerData,
    ) -> anyhow::Result<bool> {
        if prev_layer.evaluations.len() != next_layer.evaluations.len() * 2 {
            return Ok(false);
        }

        if next_layer.depth != prev_layer.depth - 1 {
            return Ok(false);
        }

        Ok(true)
    }
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fri_layer_verification() {
        let evaluations = vec![vec![1u8; 32], vec![2u8; 32], vec![3u8; 32]];
        let layer = FRILayerData {
            evaluations,
            merkle_root: vec![0u8; 32],
            depth: 3,
        };

        let query_indices = vec![0, 1];
        let result = FRILayerVerifier::verify_layer(&layer, &query_indices);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}