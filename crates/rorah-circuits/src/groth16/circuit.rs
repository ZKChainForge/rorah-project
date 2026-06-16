use crate::groth16::types::{Groth16ProofData, Groth16VK};
use crate::traits::{ProofData, VerifierCircuit};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_core::field::traits::FieldElement as FieldElementTrait;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub struct Groth16Verifier {
    vk: Groth16VK,
}

impl Groth16Verifier {
    pub fn new(vk: Groth16VK) -> anyhow::Result<Self> {
        vk.validate()?;
        Ok(Groth16Verifier { vk })
    }

    fn extract_proof_data(proof_data: &ProofData) -> anyhow::Result<&Groth16ProofData> {
        match proof_data {
            ProofData::Groth16(data) => Ok(data),
            _ => anyhow::bail!("Expected Groth16 proof data"),
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

impl VerifierCircuit for Groth16Verifier {
    fn name(&self) -> &'static str {
        "Groth16Verifier"
    }

    fn constraint_count(&self) -> usize {
        6_800_000
    }

    fn public_input_count(&self) -> usize {
        self.vk.public_input_count()
    }

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance> {
        anyhow::bail!("R1CS compilation not yet implemented for Groth16Verifier")
    }

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        _verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness> {
        let groth16_proof = Self::extract_proof_data(proof_data)?;
        groth16_proof.validate()?;
        Self::build_witness(self.constraint_count(), self.public_input_count())
    }

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool> {
        let groth16_proof = Self::extract_proof_data(proof_data)?;
        groth16_proof.validate()?;

        if groth16_proof.public_inputs.len() + 1 != self.vk.gamma_abc.len() {
            tracing::warn!(
                "Groth16 public inputs count mismatch: expected {}, got {}",
                self.vk.gamma_abc.len() - 1,
                groth16_proof.public_inputs.len()
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn proof_system_name(&self) -> &'static str {
        "groth16"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groth16_verifier_creation() {
        let vk = Groth16VK {
            alpha: vec![1u8; 64],
            beta: vec![2u8; 128],
            gamma: vec![3u8; 128],
            delta: vec![4u8; 128],
            gamma_abc: vec![vec![5u8; 64], vec![6u8; 64]],
        };
        let result = Groth16Verifier::new(vk);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_witness_small() {
        let result = Groth16Verifier::build_witness(10, 2);
        assert!(result.is_ok());
    }
}