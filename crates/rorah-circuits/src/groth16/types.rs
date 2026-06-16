use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16ProofData {
    pub a: Vec<u8>,
    pub b: Vec<u8>,
    pub c: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
}

impl Groth16ProofData {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.a.len() != 64 {
            anyhow::bail!("a must be 64 bytes (2 field elements)");
        }
        if self.b.len() != 128 {
            anyhow::bail!("b must be 128 bytes (4 field elements for G2)");
        }
        if self.c.len() != 64 {
            anyhow::bail!("c must be 64 bytes (2 field elements)");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        self.a.len() + self.b.len() + self.c.len() + self.public_inputs.iter().map(|p| p.len()).sum::<usize>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groth16VK {
    pub alpha: Vec<u8>,
    pub beta: Vec<u8>,
    pub gamma: Vec<u8>,
    pub delta: Vec<u8>,
    pub gamma_abc: Vec<Vec<u8>>,
}

impl Groth16VK {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.alpha.is_empty() {
            anyhow::bail!("alpha cannot be empty");
        }
        if self.beta.is_empty() {
            anyhow::bail!("beta cannot be empty");
        }
        if self.gamma.is_empty() {
            anyhow::bail!("gamma cannot be empty");
        }
        if self.delta.is_empty() {
            anyhow::bail!("delta cannot be empty");
        }
        if self.gamma_abc.is_empty() {
            anyhow::bail!("gamma_abc cannot be empty");
        }
        Ok(())
    }

    pub fn public_input_count(&self) -> usize {
        self.gamma_abc.len()
    }
}