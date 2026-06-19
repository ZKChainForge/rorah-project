use crate::traits::ProofData;
use rorah_core::r1cs::{R1CSInstance, Witness};

pub trait VerifierCircuit: Send + Sync {
    fn name(&self) -> &'static str;

    fn constraint_count(&self) -> usize;

    fn public_input_count(&self) -> usize;

    fn compile_to_r1cs(&self) -> anyhow::Result<R1CSInstance>;

    fn generate_witness(
        &self,
        proof_data: &ProofData,
        verification_key_bytes: &[u8],
    ) -> anyhow::Result<Witness>;

    fn verify_native(&self, proof_data: &ProofData) -> anyhow::Result<bool>;

    fn proof_system_name(&self) -> &'static str;
}

pub struct VerifierMetrics {
    pub constraint_count: usize,
    pub public_input_count: usize,
    pub estimated_proving_time_ms: u64,
    pub proof_size_bytes: usize,
}

impl VerifierMetrics {
    pub fn from_circuit(circuit: &dyn VerifierCircuit) -> Self {
        let constraint_count = circuit.constraint_count();
        let estimated_proving_time_ms = estimate_proving_time(constraint_count);

        VerifierMetrics {
            constraint_count,
            public_input_count: circuit.public_input_count(),
            estimated_proving_time_ms,
            proof_size_bytes: estimate_proof_size(constraint_count),
        }
    }
}

fn estimate_proving_time(constraint_count: usize) -> u64 {
    match constraint_count {
        0..=100_000 => 500,
        100_001..=500_000 => 1_000,
        500_001..=1_000_000 => 2_000,
        1_000_001..=3_000_000 => 5_000,
        3_000_001..=5_000_000 => 8_000,
        _ => 12_000,
    }
}

fn estimate_proof_size(constraint_count: usize) -> usize {
    (constraint_count as f64 * 0.001).ceil() as usize
}