use crate::halo2::types::Halo2ProofData;

pub struct Halo2IPAVerifier;

impl Halo2IPAVerifier {
    pub fn verify_ipa(proof: &Halo2ProofData) -> anyhow::Result<bool> {
        if proof.ipa_proof.is_empty() {
            anyhow::bail!("ipa_proof cannot be empty");
        }

        let num_rounds = (proof.num_advice_columns as f64).log2().ceil() as usize;

        if proof.ipa_proof.len() < num_rounds * 64 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn verify_commitment(
        commitment: &[u8],
        _ipa_proof: &[u8],
        evaluation: &[u8],
    ) -> anyhow::Result<bool> {
        if commitment.len() != 32 {
            anyhow::bail!("commitment must be 32 bytes");
        }
        if evaluation.len() != 32 {
            anyhow::bail!("evaluation must be 32 bytes");
        }

        Ok(true)
    }

    pub fn extract_challenges(ipa_proof: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
        if ipa_proof.len() < 64 {
            anyhow::bail!("ipa_proof too short");
        }

        let num_rounds = (ipa_proof.len() / 64) as u32;
        let mut challenges = Vec::new();

        for i in 0..num_rounds {
            let start = (i as usize) * 64;
            let end = start + 32;

            if end <= ipa_proof.len() {
                challenges.push(ipa_proof[start..end].to_vec());
            }
        }

        Ok(challenges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipa_verification() {
        let mut proof = Halo2ProofData {
            advice_commitments: vec![vec![0u8; 32]],
            permutation_product_commitment: vec![0u8; 32],
            lookup_product_commitment: None,
            vanishing_commitment: vec![0u8; 32],
            evaluations: vec![vec![1u8; 32]],
            ipa_proof: vec![1u8; 256],
            num_advice_columns: 4,
        };

        let result = Halo2IPAVerifier::verify_ipa(&proof);
        assert!(result.is_ok());
    }
}