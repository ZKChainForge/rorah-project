use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoojumProofData {
    pub fri_layers: Vec<FRILayerData>,
    pub merkle_paths: Vec<Vec<Vec<u8>>>,
    pub lde_evaluations: Vec<Vec<u8>>,
    pub quotient_poly: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

impl BoojumProofData {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.fri_layers.is_empty() {
            anyhow::bail!("FRI layers cannot be empty");
        }
        if self.merkle_paths.is_empty() {
            anyhow::bail!("Merkle paths cannot be empty");
        }
        if self.lde_evaluations.is_empty() {
            anyhow::bail!("LDE evaluations cannot be empty");
        }
        if self.quotient_poly.is_empty() {
            anyhow::bail!("Quotient polynomial cannot be empty");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> usize {
        let fri_size: usize = self.fri_layers.iter().map(|l| l.size_bytes()).sum();
        let merkle_size: usize = self.merkle_paths.iter().flatten().map(|p| p.len()).sum();
        fri_size + merkle_size + self.lde_evaluations.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FRILayerData {
    pub evaluations: Vec<Vec<u8>>,
    pub merkle_root: Vec<u8>,
    pub depth: u32,
}

impl FRILayerData {
    pub fn size_bytes(&self) -> usize {
        self.evaluations.iter().map(|e| e.len()).sum::<usize>() + self.merkle_root.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoojumVK {
    pub commitment_tree_root: Vec<u8>,
    pub constraint_count: u32,
    pub domain_size: u32,
    pub fri_rate: u32,
    pub security_bits: u32,
}

impl BoojumVK {
    pub fn new(
        commitment_tree_root: Vec<u8>,
        constraint_count: u32,
        domain_size: u32,
    ) -> Self {
        BoojumVK {
            commitment_tree_root,
            constraint_count,
            domain_size,
            fri_rate: 2,
            security_bits: 100,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.commitment_tree_root.len() != 32 {
            anyhow::bail!("commitment_tree_root must be 32 bytes");
        }
        if self.constraint_count == 0 {
            anyhow::bail!("constraint_count must be > 0");
        }
        if self.domain_size == 0 {
            anyhow::bail!("domain_size must be > 0");
        }
        if self.security_bits < 80 {
            anyhow::bail!("security_bits must be >= 80");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoojumPublicInputs {
    pub new_state_root: Vec<u8>,
    pub block_number: u64,
    pub tx_count: u32,
}

impl BoojumPublicInputs {
    pub fn new(new_state_root: Vec<u8>, block_number: u64, tx_count: u32) -> Self {
        BoojumPublicInputs {
            new_state_root,
            block_number,
            tx_count,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.new_state_root.len() != 32 {
            anyhow::bail!("new_state_root must be 32 bytes");
        }
        if self.block_number == 0 {
            anyhow::bail!("block_number must be > 0");
        }
        Ok(())
    }
}