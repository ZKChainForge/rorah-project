use crate::cairo::air_check::CairoAIRChecker;
use crate::cairo::execution_check::ExecutionVerifier;
use crate::cairo::types::{CairoProofData, CairoVK};
use crate::traits::{ProofData, VerifierCircuit};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_core::field::traits::FieldElement as FieldElementTrait;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub struct CairoVerifier {
    vk: CairoVK,
}

impl CairoVerifier {
    pub fn new(vk: CairoVK) -> anyhow::Result<Self> {
        vk.validate()?;
        Ok(CairoVerifier { vk })
    }

    fn extract_proof_data(proof_data: &ProofData) -> anyhow::Result<&CairoProofData> {
        match proof_data {
            ProofData::Cairo(data) => Ok(data),
            _ => anyhow::bail!("Expected Cairo proof data"),
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

impl VerifierCircuit for CairoVerifier {
    fn name(&self) -> &'static str {
        "CairoVerifier"
    }

    fn constraint_count(&self) -> usize {
        4_300_000
    }

    fn public_input_count(&self) -> usize {
        64
    }

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance> {
        anyhow::bail!("R1CS compilation not yet implemented for CairoVerifier")
    }

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        _verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness> {
        let cairo_proof = Self::extract_proof_data(proof_data)?;
        cairo_proof.validate()?;
        Self::build_witness(self.constraint_count(), self.public_input_count())
    }

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool> {
        let cairo_proof = Self::extract_proof_data(proof_data)?;
        cairo_proof.validate()?;

        let air_valid = CairoAIRChecker::verify_air(cairo_proof)?;
        if !air_valid {
            tracing::warn!("Cairo AIR verification failed");
            return Ok(false);
        }

        let execution_valid = ExecutionVerifier::verify_execution_trace(cairo_proof)?;
        if !execution_valid {
            tracing::warn!("Cairo execution verification failed");
            return Ok(false);
        }

        Ok(true)
    }

    fn proof_system_name(&self) -> &'static str {
        "cairo"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cairo_verifier_creation() {
        let vk = CairoVK {
            program_hash: vec![0u8; 32],
            output_size: 32,
            public_memory_size: 1024,
        };
        let result = CairoVerifier::new(vk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_witness_small() {
        let result = CairoVerifier::build_witness(10, 4);
        assert!(result.is_ok());
    }
}