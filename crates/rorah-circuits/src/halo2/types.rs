use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Halo2ProofData {
    pub advice_commitments: Vec<Vec<u8>>,
    pub permutation_product_commitment: Vec<u8>,
    pub lookup_product_commitment: Option<Vec<u8>>,
    pub vanishing_commitment: Vec<u8>,
    pub evaluations: Vec<Vec<u8>>,
    pub ipa_proof: Vec<u8>,
    pub num_advice_columns: u32,
}

impl Halo2ProofData {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.advice_commitments.is_empty() {
            anyhow::bail!("advice_commitments cannot be empty");
        }
        if self.permutation_product_commitment.is_empty() {
            anyhow::bail!("permutation_product_commitment cannot be empty");
        }
        if self.vanishing_commitment.is_empty() {
            anyhow::bail!("vanishing_commitment cannot be empty");
        }
        if self.evaluations.is_empty() {
            anyhow::bail!("evaluations cannot be empty");
        }
        if self.ipa_proof.is_empty() {
            anyhow::bail!("ipa_proof cannot be empty");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        self.advice_commitments.iter().map(|c| c.len()).sum::<usize>()
            + self.permutation_product_commitment.len()
            + self.lookup_product_commitment.as_ref().map(|l| l.len()).unwrap_or(0)
            + self.vanishing_commitment.len()
            + self.evaluations.iter().map(|e| e.len()).sum::<usize>()
            + self.ipa_proof.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Halo2VK {
    pub num_advice_columns: u32,
    pub num_fixed_columns: u32,
    pub num_instance_columns: u32,
    pub degree: u32,
    pub has_lookup: bool,
}

impl Halo2VK {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.num_advice_columns == 0 {
            anyhow::bail!("num_advice_columns must be > 0");
        }
        if self.degree == 0 || self.degree > 20 {
            anyhow::bail!("degree must be between 1 and 20");
        }
        Ok(())
    }
}