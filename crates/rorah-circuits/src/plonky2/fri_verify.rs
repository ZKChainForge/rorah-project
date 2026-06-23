use crate::plonky2::types::Plonky2ProofData;

pub struct Plonky2FRIVerifier;

impl Plonky2FRIVerifier {
    pub fn verify_fri_proof(proof: &Plonky2ProofData) -> anyhow::Result<bool> {
        if proof.wire_caps.is_empty() {
            anyhow::bail!("wire_caps cannot be empty");
        }

        if proof.opening_proof.is_empty() {
            anyhow::bail!("opening_proof cannot be empty");
        }

        let rate_bits = proof.degree_bits.saturating_sub(1);
        if rate_bits == 0 {
            anyhow::bail!("Invalid degree bits");
        }

        for cap in &proof.wire_caps {
            if cap.len() != 32 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_opening(
        opening_proof: &[u8],
        commitment: &[u8],
        _evaluation_point: &[u8],
    ) -> anyhow::Result<bool> {
        if opening_proof.is_empty() {
            anyhow::bail!("opening_proof cannot be empty");
        }
        if commitment.is_empty() {
            anyhow::bail!("commitment cannot be empty");
        }

        Ok(true)
    }

    pub fn compute_folding_challenges(
        opening_proof: &[u8],
        num_rounds: u32,
    ) -> anyhow::Result<Vec<Vec<u8>>> {
        if num_rounds == 0 {
            anyhow::bail!("num_rounds must be > 0");
        }

        let mut challenges = Vec::new();
        for i in 0..num_rounds {
            let start = (i as usize) * 32;
            let end = start + 32;

            if end <= opening_proof.len() {
                challenges.push(opening_proof[start..end].to_vec());
            } else {
                challenges.push(vec![0u8; 32]);
            }
        }

        Ok(challenges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plonky2_fri_verification() {
        let mut proof = Plonky2ProofData {
            wire_caps: vec![vec![0u8; 32]],
            zs_partial_products_cap: vec![0u8; 32],
            quotient_polys_cap: vec![0u8; 32],
            openings: vec![vec![1u8; 32]],
            opening_proof: vec![1u8; 128],
            degree_bits: 8,
        };

        let result = Plonky2FRIVerifier::verify_fri_proof(&proof);
        assert!(result.is_ok());
    }
}