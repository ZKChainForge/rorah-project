//! Pedersen commitment scheme over BN254 G1.

//! Pedersen commitment scheme over BN254 G1.

use crate::commitment::params::CommitmentParams;
use crate::commitment::traits::{CommitmentScheme, Opening};
use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement;
use ark_bn254::{G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::{Deserialize, Serialize};


/// A Pedersen commitment: a single G1Affine point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PedersenCommitment(pub(crate) G1Affine);

impl crate::commitment::traits::Commitment for PedersenCommitment {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.0
            .serialize_compressed(&mut bytes)
            .expect("G1Affine serialization cannot fail");
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(RorahError::CommitmentError(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let point = G1Affine::deserialize_compressed(bytes).map_err(|e| {
            RorahError::CommitmentError(format!("G1Affine deserialize: {}", e))
        })?;

        if !point.is_on_curve() {
            return Err(RorahError::CommitmentError(
                "point not on curve".to_string(),
            ));
        }

        if !point.is_in_correct_subgroup_assuming_on_curve() {
            return Err(RorahError::CommitmentError(
                "point not in correct subgroup".to_string(),
            ));
        }

        Ok(Self(point))
    }
}

impl Serialize for PedersenCommitment {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use crate::commitment::traits::Commitment;
        let hex: String = self
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        s.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for PedersenCommitment {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use crate::commitment::traits::Commitment;
        let hex_str = String::deserialize(d)?;
        let bytes = hex::decode(&hex_str)
            .map_err(|e| serde::de::Error::custom(format!("hex: {}", e)))?;
        Self::from_bytes(&bytes)
            .map_err(|e| serde::de::Error::custom(format!("commitment: {}", e)))
    }
}

impl PedersenCommitment {
    pub fn inner(&self) -> &G1Affine { &self.0 }

    pub fn from_inner(point: G1Affine) -> Self { Self(point) }

    pub fn is_identity(&self) -> bool { self.0.is_zero() }

    /// Homomorphic addition: C1 + C2
    pub fn add(&self, other: &Self) -> Self {
        let sum = G1Projective::from(self.0) + G1Projective::from(other.0);
        Self(sum.into_affine())
    }

    /// Homomorphic scalar multiply: s * C
    pub fn scale(&self, scalar: &BN254FieldElement) -> Self {
        let scaled = self.0 * scalar.inner();
        Self(scaled.into_affine())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CommitmentScheme impl
// ─────────────────────────────────────────────────────────────────────────────

pub struct PedersenScheme {
    params: CommitmentParams,
}

impl PedersenScheme {
    pub fn new(params: CommitmentParams) -> Self { Self { params } }

    pub fn with_max_size(max_size: usize) -> Self {
        Self::new(CommitmentParams::new(max_size))
    }

    pub fn params(&self) -> &CommitmentParams { &self.params }
}

impl CommitmentScheme for PedersenScheme {
    type Field      = BN254FieldElement;
    type Commitment = PedersenCommitment;

    fn commit(
        &self,
        message: &[BN254FieldElement],
        blinding: BN254FieldElement,
    ) -> Result<PedersenCommitment> {
        let point = self.params.commit(message, blinding)?;
        Ok(PedersenCommitment(point))
    }

    fn verify(
        &self,
        commitment: &PedersenCommitment,
        opening: &Opening<BN254FieldElement>,
    ) -> Result<()> {
        self.params.verify(&commitment.0, &opening.message, opening.blinding)
    }

    fn max_message_len(&self) -> usize {
        self.params.max_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::traits::Commitment;

    fn scheme(n: usize) -> PedersenScheme {
        PedersenScheme::with_max_size(n)
    }

    #[test]
    fn test_commit_and_verify() {
        let s = scheme(8);
        let msg = vec![BN254FieldElement::from_u64(10), BN254FieldElement::from_u64(20)];
        let r = BN254FieldElement::from_u64(42);

        let c = s.commit(&msg, r).unwrap();
        let opening = Opening { message: msg.clone(), blinding: r };
        assert!(s.verify(&c, &opening).is_ok());
    }

    #[test]
    fn test_wrong_message_rejected() {
        let s = scheme(4);
        let msg = vec![BN254FieldElement::from_u64(5)];
        let r = BN254FieldElement::from_u64(1);

        let c = s.commit(&msg, r).unwrap();
        let wrong = Opening {
            message: vec![BN254FieldElement::from_u64(6)],
            blinding: r,
        };
        assert!(s.verify(&c, &wrong).is_err());
    }

    #[test]
    fn test_homomorphic_addition() {
        let s = scheme(4);

        let m1 = vec![BN254FieldElement::from_u64(10)];
        let m2 = vec![BN254FieldElement::from_u64(20)];
        let m_sum = vec![BN254FieldElement::from_u64(30)];

        let r1 = BN254FieldElement::from_u64(5);
        let r2 = BN254FieldElement::from_u64(7);
        let r_sum = BN254FieldElement::from_u64(12);

        let c1 = s.commit(&m1, r1).unwrap();
        let c2 = s.commit(&m2, r2).unwrap();
        let c_sum = s.commit(&m_sum, r_sum).unwrap();

        assert_eq!(c1.add(&c2), c_sum);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let s = scheme(4);
        let msg = vec![BN254FieldElement::from_u64(77)];
        let r = BN254FieldElement::from_u64(13);

        let c = s.commit(&msg, r).unwrap();
        let bytes = c.to_bytes();
        assert_eq!(bytes.len(), 32);

        let recovered = PedersenCommitment::from_bytes(&bytes).unwrap();
        assert_eq!(c, recovered);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = scheme(4);
        let msg = vec![BN254FieldElement::from_u64(55)];
        let r = BN254FieldElement::from_u64(9);

        let c = s.commit(&msg, r).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let recovered: PedersenCommitment = serde_json::from_str(&json).unwrap();
        assert_eq!(c, recovered);
    }

    #[test]
    fn test_wrong_length_rejected() {
        let bad = vec![0u8; 31];
        assert!(PedersenCommitment::from_bytes(&bad).is_err());
    }
}