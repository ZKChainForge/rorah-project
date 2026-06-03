//! Commitment parameters: pre-generated group element generators.

use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement;
use crate::field::traits::FieldElement;
use ark_bn254::{G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_serialize::CanonicalDeserialize;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use sha3::{Digest, Sha3_256};

#[derive(Clone, Debug)]
pub struct CommitmentParams {
    pub generators: Vec<G1Affine>,
    pub blinding_generator: G1Affine,
}

impl CommitmentParams {
    /// Generate parameters for vectors up to `max_size`.
    pub fn new(max_size: usize) -> Self {
        let generators = (0..max_size)
            .map(|i| hash_to_g1(&format!("RORAH_COMMIT_GEN_{}", i)))
            .collect();
        let blinding_generator = hash_to_g1("RORAH_COMMIT_BLIND");

        Self { generators, blinding_generator }
    }

    pub fn max_size(&self) -> usize {
        self.generators.len()
    }

    /// Pedersen commit: C = Σ mᵢ·Gᵢ + r·H
    pub fn commit(
        &self,
        message: &[BN254FieldElement],
        blinding: BN254FieldElement,
    ) -> Result<G1Affine> {
        if message.len() > self.generators.len() {
            return Err(RorahError::CommitmentError(format!(
                "message length {} exceeds max_size {}",
                message.len(),
                self.generators.len()
            )));
        }

        // Build scalars and bases for MSM
        // ark 0.5: VariableBaseMSM::msm takes &[G], &[F]
        let scalars: Vec<ark_bn254::Fr> = message
            .iter()
            .map(|f| *f.inner())
            .chain(std::iter::once(*blinding.inner()))
            .collect();

        let bases: Vec<G1Affine> = self.generators[..message.len()]
            .iter()
            .cloned()
            .chain(std::iter::once(self.blinding_generator))
            .collect();

        // ark 0.5 MSM API
        let result = G1Projective::msm(&bases, &scalars)
            .map_err(|e| RorahError::CommitmentError(format!("MSM error: {}", e)))?;

        Ok(result.into_affine())
    }

    pub fn commit_unblinded(&self, message: &[BN254FieldElement]) -> Result<G1Affine> {
        self.commit(message, BN254FieldElement::zero())
    }

    pub fn verify(
        &self,
        commitment: &G1Affine,
        message: &[BN254FieldElement],
        blinding: BN254FieldElement,
    ) -> Result<()> {
        let computed = self.commit(message, blinding)?;
        if &computed != commitment {
            return Err(RorahError::CommitmentError(
                "commitment verification failed".to_string(),
            ));
        }
        Ok(())
    }
}

/// Hash string to G1 curve point via try-and-increment.
pub(crate) fn hash_to_g1(input: &str) -> G1Affine {
    let mut counter: u64 = 0;

    loop {
        let mut hasher = Sha3_256::new();
        hasher.update(input.as_bytes());
        hasher.update(&counter.to_le_bytes());
        let hash = hasher.finalize();

        // Try compressed deserialization
        if let Ok(point) = G1Affine::deserialize_compressed(&hash[..]) {
            if point.is_on_curve()
                && point.is_in_correct_subgroup_assuming_on_curve()
                && !point.is_zero()
            {
                return point;
            }
        }

        counter += 1;

        // Fall back to RNG after many failures
        if counter > 10_000 {
            let seed: [u8; 32] = hash.into();
            let mut rng = ChaCha20Rng::from_seed(seed);
            loop {
                use ark_std::UniformRand;
                let point = G1Affine::rand(&mut rng);
                if point.is_on_curve()
                    && point.is_in_correct_subgroup_assuming_on_curve()
                    && !point.is_zero()
                {
                    return point;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::AffineRepr;

    #[test]
    fn test_params_creation() {
        let params = CommitmentParams::new(8);
        assert_eq!(params.max_size(), 8);
    }

    #[test]
    fn test_generators_valid() {
        let params = CommitmentParams::new(4);
        for g in &params.generators {
            assert!(g.is_on_curve());
            assert!(g.is_in_correct_subgroup_assuming_on_curve());
            assert!(!g.is_zero());
        }
        assert!(params.blinding_generator.is_on_curve());
        assert!(!params.blinding_generator.is_zero());
    }

    #[test]
    fn test_generators_distinct() {
        let params = CommitmentParams::new(6);
        for i in 0..params.generators.len() {
            for j in (i + 1)..params.generators.len() {
                assert_ne!(params.generators[i], params.generators[j]);
            }
        }
    }

    #[test]
    fn test_deterministic() {
        let p1 = CommitmentParams::new(4);
        let p2 = CommitmentParams::new(4);
        for i in 0..4 {
            assert_eq!(p1.generators[i], p2.generators[i]);
        }
        assert_eq!(p1.blinding_generator, p2.blinding_generator);
    }

    #[test]
    fn test_commit_and_verify() {
        let params = CommitmentParams::new(4);
        let msg = vec![
            BN254FieldElement::from_u64(10),
            BN254FieldElement::from_u64(20),
        ];
        let r = BN254FieldElement::from_u64(99);

        let c = params.commit(&msg, r).unwrap();
        assert!(params.verify(&c, &msg, r).is_ok());
    }

    #[test]
    fn test_wrong_message_fails() {
        let params = CommitmentParams::new(4);
        let msg = vec![BN254FieldElement::from_u64(10)];
        let r = BN254FieldElement::from_u64(1);

        let c = params.commit(&msg, r).unwrap();

        let wrong = vec![BN254FieldElement::from_u64(11)];
        assert!(params.verify(&c, &wrong, r).is_err());
    }

    #[test]
    fn test_too_large_fails() {
        let params = CommitmentParams::new(2);
        let msg = vec![
            BN254FieldElement::from_u64(1),
            BN254FieldElement::from_u64(2),
            BN254FieldElement::from_u64(3),
        ];
        assert!(params.commit(&msg, BN254FieldElement::zero()).is_err());
    }
}