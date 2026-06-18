use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CairoProofData {
    pub trace_commitments: Vec<Vec<u8>>,
    pub composition_polynomial: Vec<u8>,
    pub fri_proof: Vec<u8>,
    pub decommitment_values: Vec<Vec<u8>>,
    pub num_steps: u64,
}

impl CairoProofData {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.trace_commitments.is_empty() {
            anyhow::bail!("trace_commitments cannot be empty");
        }
        if self.composition_polynomial.is_empty() {
            anyhow::bail!("composition_polynomial cannot be empty");
        }
        if self.fri_proof.is_empty() {
            anyhow::bail!("fri_proof cannot be empty");
        }
        if self.num_steps == 0 {
            anyhow::bail!("num_steps must be > 0");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        self.trace_commitments.iter().map(|c| c.len()).sum::<usize>()
            + self.composition_polynomial.len()
            + self.fri_proof.len()
            + self.decommitment_values.iter().map(|d| d.len()).sum::<usize>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CairoVK {
    pub program_hash: Vec<u8>,
    pub output_size: u32,
    pub public_memory_size: u32,
}

impl CairoVK {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.program_hash.len() != 32 {
            anyhow::bail!("program_hash must be 32 bytes");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CairoPublicInputs {
    pub program_hash: Vec<u8>,
    pub public_memory: Vec<Vec<u8>>,
    pub output_hash: Vec<u8>,
}

impl CairoPublicInputs {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.program_hash.len() != 32 {
            anyhow::bail!("program_hash must be 32 bytes");
        }
        if self.public_memory.is_empty() {
            anyhow::bail!("public_memory cannot be empty");
        }
        if self.output_hash.len() != 32 {
            anyhow::bail!("output_hash must be 32 bytes");
        }
        Ok(())
    }
}