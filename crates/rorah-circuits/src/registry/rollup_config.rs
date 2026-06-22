use crate::traits::ProofSystem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub id: String,
    pub proof_system: ProofSystem,
    pub verification_key_hash: Vec<u8>,
    pub fee_wei: u128,
    pub active: bool,
    pub priority: u32,
}

impl RollupConfig {
    pub fn new(
        id: String,
        proof_system: ProofSystem,
        verification_key_hash: Vec<u8>,
    ) -> Self {
        RollupConfig {
            id,
            proof_system,
            verification_key_hash,
            fee_wei: 5_000_000_000_000_000,
            active: true,
            priority: 50,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.id.is_empty() {
            anyhow::bail!("id cannot be empty");
        }
        if self.verification_key_hash.len() != 32 {
            anyhow::bail!("verification_key_hash must be 32 bytes, got {}", self.verification_key_hash.len());
        }
        Ok(())
    }

    pub fn with_fee(mut self, fee_wei: u128) -> Self {
        self.fee_wei = fee_wei;
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority.max(1).min(100);
        self
    }

    pub fn deactivate(mut self) -> Self {
        self.active = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollup_config_creation() {
        let config = RollupConfig::new(
            "test-rollup".to_string(),
            ProofSystem::Boojum,
            vec![0u8; 32],
        );
        assert_eq!(config.id, "test-rollup");
        assert_eq!(config.proof_system, ProofSystem::Boojum);
        assert!(config.active);
    }

    #[test]
    fn test_rollup_config_validation() {
        let config = RollupConfig::new(
            "test".to_string(),
            ProofSystem::Plonky2,
            vec![0u8; 32],
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_vk_hash_length() {
        let config = RollupConfig::new(
            "test".to_string(),
            ProofSystem::Groth16,
            vec![0u8; 16],
        );
        assert!(config.validate().is_err());
    }
}