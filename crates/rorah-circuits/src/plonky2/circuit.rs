use crate::plonky2::fri_verify::Plonky2FRIVerifier;
use crate::plonky2::types::{Plonky2ProofData, Plonky2VK};
use crate::traits::{ProofData, VerifierCircuit};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_core::field::traits::FieldElement as FieldElementTrait;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub struct Plonky2Verifier {
    vk: Plonky2VK,
}

impl Plonky2Verifier {
    pub fn new(vk: Plonky2VK) -> anyhow::Result<Self> {
        vk.validate()?;
        Ok(Plonky2Verifier { vk })
    }

    fn extract_proof_data(proof_data: &ProofData) -> anyhow::Result<&Plonky2ProofData> {
        match proof_data {
            ProofData::Plonky2(data) => Ok(data),
            _ => anyhow::bail!("Expected Plonky2 proof data"),
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

impl VerifierCircuit for Plonky2Verifier {
    fn name(&self) -> &'static str {
        "Plonky2Verifier"
    }

    fn constraint_count(&self) -> usize {
        3_200_000
    }

    fn public_input_count(&self) -> usize {
        32
    }

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance> {
        anyhow::bail!("R1CS compilation not yet implemented for Plonky2Verifier")
    }

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        _verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness> {
        let plonky2_proof = Self::extract_proof_data(proof_data)?;
        plonky2_proof.validate()?;
        Self::build_witness(self.constraint_count(), self.public_input_count())
    }

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool> {
        let plonky2_proof = Self::extract_proof_data(proof_data)?;
        plonky2_proof.validate()?;

        let fri_valid = Plonky2FRIVerifier::verify_fri_proof(plonky2_proof)?;
        if !fri_valid {
            tracing::warn!("Plonky2 FRI verification failed");
            return Ok(false);
        }

        if plonky2_proof.wire_caps.len() != 4 {
            tracing::warn!(
                "Plonky2 wire caps count mismatch: expected 4, got {}",
                plonky2_proof.wire_caps.len()
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn proof_system_name(&self) -> &'static str {
        "plonky2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plonky2::types::FRIParams;

    #[test]
    fn test_plonky2_verifier_creation() {
        let vk = Plonky2VK {
            circuit_digest: vec![0u8; 32],
            fri_params: FRIParams {
                rate_bits: 2,
                cap_height: 4,
                num_queries: 8,
            },
            gate_types: vec!["arithmetic".to_string()],
            num_gates: 1000,
        };
        let result = Plonky2Verifier::new(vk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_witness_small() {
        let result = Plonky2Verifier::build_witness(10, 4);
        assert!(result.is_ok());
    }
}