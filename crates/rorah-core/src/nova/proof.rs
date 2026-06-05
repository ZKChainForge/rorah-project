
//! Nova proof structure.

use crate::commitment::pedersen::PedersenCommitment;
use serde::{Deserialize, Serialize};


/// Proof that a Nova folding step was performed correctly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovaProof {
    pub cross_term_commitment: PedersenCommitment,
}

impl NovaProof {
    /// Create a new Nova proof.
    pub fn new(cross_term_commitment: PedersenCommitment) -> Self {
        Self {
            cross_term_commitment,
        }
    }

    /// Get the commitment to T.
    pub fn commitment(&self) -> &PedersenCommitment {
        &self.cross_term_commitment
    }

    /// Size of proof in bytes.
    pub fn size_bytes() -> usize {
        32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::params::CommitmentParams;
    use crate::field::bn254::BN254FieldElement;

    #[test]
    fn test_proof_creation() {
        let params = CommitmentParams::new(10);
        let message = vec![BN254FieldElement::from_u64(42)];
        let commitment = params.commit_unblinded(&message).unwrap();

        let proof = NovaProof::new(PedersenCommitment(commitment));

        assert_eq!(proof.commitment(), &PedersenCommitment(commitment));
    }

    #[test]
    fn test_proof_serialization() {
        let params = CommitmentParams::new(5);
        let message = vec![BN254FieldElement::from_u64(100)];
        let commitment = params.commit_unblinded(&message).unwrap();

        let proof = NovaProof::new(PedersenCommitment(commitment));

        let json = serde_json::to_string(&proof).unwrap();
        let recovered: NovaProof = serde_json::from_str(&json).unwrap();

        assert_eq!(proof, recovered);
    }
}