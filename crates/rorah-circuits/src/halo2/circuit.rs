use crate::halo2::ipa_verify::Halo2IPAVerifier;
use crate::halo2::types::{Halo2ProofData, Halo2VK};
use crate::traits::{ProofData, VerifierCircuit};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_core::field::traits::FieldElement as FieldElementTrait;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub struct Halo2Verifier {
    vk: Halo2VK,
}

impl Halo2Verifier {
    pub fn new(vk: Halo2VK) -> anyhow::Result<Self> {
        vk.validate()?;
        Ok(Halo2Verifier { vk })
    }

    fn extract_proof_data(proof_data: &ProofData) -> anyhow::Result<&Halo2ProofData> {
        match proof_data {
            ProofData::Halo2(data) => Ok(data),
            _ => anyhow::bail!("Expected Halo2 proof data"),
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

impl VerifierCircuit for Halo2Verifier {
    fn name(&self) -> &'static str {
        "Halo2Verifier"
    }

    fn constraint_count(&self) -> usize {
        1_900_000
    }

    fn public_input_count(&self) -> usize {
        32
    }

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance> {
        anyhow::bail!("R1CS compilation not yet implemented for Halo2Verifier")
    }

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        _verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness> {
        let halo2_proof = Self::extract_proof_data(proof_data)?;
        halo2_proof.validate()?;
        Self::build_witness(self.constraint_count(), self.public_input_count())
    }

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool> {
        let halo2_proof = Self::extract_proof_data(proof_data)?;
        halo2_proof.validate()?;

        let ipa_valid = Halo2IPAVerifier::verify_ipa(halo2_proof)?;
        if !ipa_valid {
            tracing::warn!("Halo2 IPA verification failed");
            return Ok(false);
        }

        if halo2_proof.advice_commitments.len() as u32 != self.vk.num_advice_columns {
            tracing::warn!(
                "Halo2 advice columns mismatch: expected {}, got {}",
                self.vk.num_advice_columns,
                halo2_proof.advice_commitments.len()
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn proof_system_name(&self) -> &'static str {
        "halo2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halo2_verifier_creation() {
        let vk = Halo2VK {
            num_advice_columns: 4,
            num_fixed_columns: 2,
            num_instance_columns: 1,
            degree: 16,
            has_lookup: false,
        };
        let result = Halo2Verifier::new(vk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_witness_small() {
        let result = Halo2Verifier::build_witness(10, 4);
        assert!(result.is_ok());
    }
}