use crate::boojum::constraints::ConstraintChecker;
use crate::boojum::fri_verify::BoojumFRIVerifier;
use crate::boojum::types::{BoojumProofData, BoojumVK};
use crate::traits::{ProofData, VerifierCircuit};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_core::field::traits::FieldElement as FieldElementTrait;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub struct BoojumVerifier {
    vk: BoojumVK,
}

impl BoojumVerifier {
    pub fn new(vk: BoojumVK) -> anyhow::Result<Self> {
        vk.validate()?;
        Ok(BoojumVerifier { vk })
    }

    fn extract_proof_data(proof_data: &ProofData) -> anyhow::Result<&BoojumProofData> {
        match proof_data {
            ProofData::Boojum(data) => Ok(data),
            _ => anyhow::bail!("Expected Boojum proof data"),
        }
    }

    fn build_witness(
        constraint_count: usize,
        public_input_count: usize,
    ) -> anyhow::Result<Witness> {
        let total = constraint_count + public_input_count + 1;
        let mut variables = Vec::with_capacity(total);

        variables.push(BN254FieldElement::one());

        for i in 1..total {
            variables.push(BN254FieldElement::from_u64(i as u64));
        }

        let public_len = public_input_count + 1;
        Ok(Witness::new(variables, public_len)?)
    }
}

impl VerifierCircuit for BoojumVerifier {
    fn name(&self) -> &'static str {
        "BoojumVerifier"
    }

    fn constraint_count(&self) -> usize {
        5_200_000
    }

    fn public_input_count(&self) -> usize {
        64
    }

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance> {
        anyhow::bail!("R1CS compilation not yet implemented for BoojumVerifier")
    }

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        _verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness> {
        let boojum_proof = Self::extract_proof_data(proof_data)?;
        boojum_proof.validate()?;
        Self::build_witness(self.constraint_count(), self.public_input_count())
    }

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool> {
        let boojum_proof = Self::extract_proof_data(proof_data)?;
        boojum_proof.validate()?;

        let fri_valid = BoojumFRIVerifier::verify_fri(boojum_proof)?;
        if !fri_valid {
            tracing::warn!("Boojum FRI verification failed");
            return Ok(false);
        }

        let constraint_valid = ConstraintChecker::verify_constraint_polynomial(
            &boojum_proof.lde_evaluations,
            &boojum_proof.quotient_poly,
        )?;
        if !constraint_valid {
            tracing::warn!("Boojum constraint verification failed");
            return Ok(false);
        }

        let query_valid = BoojumFRIVerifier::verify_query_phase(boojum_proof, 8)?;
        if !query_valid {
            tracing::warn!("Boojum query phase verification failed");
            return Ok(false);
        }

        Ok(true)
    }

    fn proof_system_name(&self) -> &'static str {
        "boojum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boojum_verifier_creation() {
        let vk = BoojumVK::new(vec![0u8; 32], 1000, 2048);
        let result = BoojumVerifier::new(vk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_boojum_constraint_count() {
        let vk = BoojumVK::new(vec![0u8; 32], 1000, 2048);
        let verifier = BoojumVerifier::new(vk).unwrap();
        assert_eq!(verifier.constraint_count(), 5_200_000);
        assert_eq!(verifier.proof_system_name(), "boojum");
    }

    #[test]
    fn test_build_witness_small() {
        let result = BoojumVerifier::build_witness(10, 4);
        assert!(result.is_ok());
        let w = result.unwrap();
        assert_eq!(w.len(), 15);
    }
}