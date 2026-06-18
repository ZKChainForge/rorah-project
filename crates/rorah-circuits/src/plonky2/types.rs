use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plonky2ProofData {
    pub wire_caps: Vec<Vec<u8>>,
    pub zs_partial_products_cap: Vec<u8>,
    pub quotient_polys_cap: Vec<u8>,
    pub openings: Vec<Vec<u8>>,
    pub opening_proof: Vec<u8>,
    pub degree_bits: u32,
}

impl Plonky2ProofData {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.wire_caps.is_empty() {
            anyhow::bail!("wire_caps cannot be empty");
        }
        if self.zs_partial_products_cap.is_empty() {
            anyhow::bail!("zs_partial_products_cap cannot be empty");
        }
        if self.quotient_polys_cap.is_empty() {
            anyhow::bail!("quotient_polys_cap cannot be empty");
        }
        if self.openings.is_empty() {
            anyhow::bail!("openings cannot be empty");
        }
        if self.opening_proof.is_empty() {
            anyhow::bail!("opening_proof cannot be empty");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        self.wire_caps.iter().map(|c| c.len()).sum::<usize>()
            + self.zs_partial_products_cap.len()
            + self.quotient_polys_cap.len()
            + self.openings.iter().map(|o| o.len()).sum::<usize>()
            + self.opening_proof.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plonky2VK {
    pub circuit_digest: Vec<u8>,
    pub fri_params: FRIParams,
    pub gate_types: Vec<String>,
    pub num_gates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FRIParams {
    pub rate_bits: u32,
    pub cap_height: u32,
    pub num_queries: u32,
}

impl Plonky2VK {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.circuit_digest.len() != 32 {
            anyhow::bail!("circuit_digest must be 32 bytes");
        }
        if self.gate_types.is_empty() {
            anyhow::bail!("gate_types cannot be empty");
        }
        Ok(())
    }
}