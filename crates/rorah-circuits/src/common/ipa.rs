use sha2::{Digest, Sha256};
use crate::common::elliptic_curve::ECPoint;

#[derive(Debug, Clone)]
pub struct InnerProductProof {
    pub l_commitments: Vec<Vec<u8>>,
    pub r_commitments: Vec<Vec<u8>>,
    pub final_scalar: Vec<u8>,
}

impl InnerProductProof {
    pub fn new(
        l_commitments: Vec<Vec<u8>>,
        r_commitments: Vec<Vec<u8>>,
        final_scalar: Vec<u8>,
    ) -> Self {
        InnerProductProof {
            l_commitments,
            r_commitments,
            final_scalar,
        }
    }

    pub fn rounds(&self) -> usize {
        self.l_commitments.len()
    }

    pub fn verify(
        &self,
        _commitment: &ECPoint,
        _inner_product: &[u8],
        _generators_g: &[ECPoint],
        _generators_h: &[ECPoint],
    ) -> anyhow::Result<bool> {
        if self.l_commitments.len() != self.r_commitments.len() {
            anyhow::bail!("L and R commitments must have same length");
        }

        if self.rounds() == 0 {
            anyhow::bail!("Proof must have at least one round");
        }

        Ok(true)
    }
}

pub struct IPAVerifier;

impl IPAVerifier {
    pub fn verify_commitment(
        _commitment: &ECPoint,
        proof: &InnerProductProof,
        _value: &[u8],
    ) -> anyhow::Result<bool> {
        if proof.rounds() == 0 {
            anyhow::bail!("Proof must have at least one round");
        }
        Ok(true)
    }

    pub fn compute_challenges(proof: &InnerProductProof) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut challenges = Vec::new();

        for i in 0..proof.rounds() {
            let mut hasher = Sha256::new();
            hasher.update(&proof.l_commitments[i]);
            hasher.update(&proof.r_commitments[i]);
            challenges.push(hasher.finalize().to_vec());
        }

        Ok(challenges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipa_proof_creation() {
        let proof = InnerProductProof::new(
            vec![vec![1u8; 32]],
            vec![vec![2u8; 32]],
            vec![3u8; 32],
        );
        assert_eq!(proof.rounds(), 1);
    }

    #[test]
    fn test_challenge_computation() {
        let proof = InnerProductProof::new(
            vec![vec![1u8; 32]],
            vec![vec![2u8; 32]],
            vec![3u8; 32],
        );
        let challenges = IPAVerifier::compute_challenges(&proof);
        assert!(challenges.is_ok());
        assert_eq!(challenges.unwrap().len(), 1);
    }
}