use serde::{Deserialize, Serialize};

use crate::boojum::types::BoojumProofData;
use crate::cairo::types::CairoProofData;
use crate::groth16::types::Groth16ProofData;
use crate::halo2::types::Halo2ProofData;
use crate::plonky2::types::Plonky2ProofData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProofSystem {
    Boojum,
    Plonky2,
    Halo2,
    Groth16,
    Cairo,
}

impl ProofSystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProofSystem::Boojum => "boojum",
            ProofSystem::Plonky2 => "plonky2",
            ProofSystem::Halo2 => "halo2",
            ProofSystem::Groth16 => "groth16",
            ProofSystem::Cairo => "cairo",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "boojum" => Some(ProofSystem::Boojum),
            "plonky2" => Some(ProofSystem::Plonky2),
            "halo2" => Some(ProofSystem::Halo2),
            "groth16" => Some(ProofSystem::Groth16),
            "cairo" => Some(ProofSystem::Cairo),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofData {
    Boojum(BoojumProofData),
    Plonky2(Plonky2ProofData),
    Halo2(Halo2ProofData),
    Groth16(Groth16ProofData),
    Cairo(CairoProofData),
}

impl ProofData {
    pub fn proof_system(&self) -> ProofSystem {
        match self {
            ProofData::Boojum(_) => ProofSystem::Boojum,
            ProofData::Plonky2(_) => ProofSystem::Plonky2,
            ProofData::Halo2(_) => ProofSystem::Halo2,
            ProofData::Groth16(_) => ProofSystem::Groth16,
            ProofData::Cairo(_) => ProofSystem::Cairo,
        }
    }

    pub fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub rollup_id: String,
    pub proof_system: ProofSystem,
    pub block_number: u64,
    pub state_root: Vec<u8>,
    pub fee_wei: u128,
    pub timestamp: u64,
}

impl ProofMetadata {
    pub fn new(
        rollup_id: String,
        proof_system: ProofSystem,
        block_number: u64,
        state_root: Vec<u8>,
        fee_wei: u128,
    ) -> Self {
        ProofMetadata {
            rollup_id,
            proof_system,
            block_number,
            state_root,
            fee_wei,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.rollup_id.is_empty() {
            anyhow::bail!("rollup_id cannot be empty");
        }
        if self.state_root.len() != 32 {
            anyhow::bail!("state_root must be 32 bytes");
        }
        if self.block_number == 0 {
            anyhow::bail!("block_number must be > 0");
        }
        Ok(())
    }
}