use crate::registry::{RollupId, RollupConfig};
use crate::traits::{VerifierCircuit, ProofSystem};
use crate::boojum::{BoojumVerifier, BoojumVK};
use crate::plonky2::{Plonky2Verifier, Plonky2VK, FRIParams};
use crate::halo2::{Halo2Verifier, Halo2VK};
use crate::groth16::{Groth16Verifier, Groth16VK};
use crate::cairo::{CairoVerifier, CairoVK};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CircuitRegistry {
    configs: HashMap<String, RollupConfig>,
    verifiers: HashMap<String, Arc<dyn VerifierCircuit>>,
}

impl CircuitRegistry {
    pub fn new() -> Self {
        CircuitRegistry {
            configs: HashMap::new(),
            verifiers: HashMap::new(),
        }
    }

    pub fn load_from_config() -> anyhow::Result<Self> {
        let mut registry = Self::new();

        registry.register_rollup(RollupConfig::new(
            crate::registry::rollup_ids::ZKSYNC_ERA.to_string(),
            ProofSystem::Boojum,
            vec![0u8; 32],
        ))?;

        registry.register_rollup(RollupConfig::new(
            crate::registry::rollup_ids::POLYGON_ZKEVM.to_string(),
            ProofSystem::Plonky2,
            vec![0u8; 32],
        ))?;

        registry.register_rollup(RollupConfig::new(
            crate::registry::rollup_ids::SCROLL.to_string(),
            ProofSystem::Halo2,
            vec![0u8; 32],
        ))?;

        registry.register_rollup(RollupConfig::new(
            crate::registry::rollup_ids::ARBITRUM_ONE.to_string(),
            ProofSystem::Groth16,
            vec![0u8; 32],
        ))?;

        registry.register_rollup(RollupConfig::new(
            crate::registry::rollup_ids::STARKNET.to_string(),
            ProofSystem::Cairo,
            vec![0u8; 32],
        ))?;

        Ok(registry)
    }

    pub fn register_rollup(&mut self, config: RollupConfig) -> anyhow::Result<()> {
        config.validate()?;

        let verifier: Arc<dyn VerifierCircuit> = match config.proof_system {
            ProofSystem::Boojum => {
                let vk = BoojumVK::new(
                    config.verification_key_hash.clone(),
                    1000,
                    2048,
                );
                Arc::new(BoojumVerifier::new(vk)?)
            }
            ProofSystem::Plonky2 => {
                let vk = Plonky2VK {
                    circuit_digest: config.verification_key_hash.clone(),
                    fri_params: FRIParams {
                        rate_bits: 2,
                        cap_height: 4,
                        num_queries: 8,
                    },
                    gate_types: vec!["arithmetic".to_string()],
                    num_gates: 1000,
                };
                Arc::new(Plonky2Verifier::new(vk)?)
            }
            ProofSystem::Halo2 => {
                let vk = Halo2VK {
                    num_advice_columns: 4,
                    num_fixed_columns: 2,
                    num_instance_columns: 1,
                    degree: 16,
                    has_lookup: false,
                };
                Arc::new(Halo2Verifier::new(vk)?)
            }
            ProofSystem::Groth16 => {
                let vk = Groth16VK {
                    alpha: vec![1u8; 64],
                    beta: vec![2u8; 128],
                    gamma: vec![3u8; 128],
                    delta: vec![4u8; 128],
                    gamma_abc: vec![config.verification_key_hash.clone()],
                };
                Arc::new(Groth16Verifier::new(vk)?)
            }
            ProofSystem::Cairo => {
                let vk = CairoVK {
                    program_hash: config.verification_key_hash.clone(),
                    output_size: 32,
                    public_memory_size: 1024,
                };
                Arc::new(CairoVerifier::new(vk)?)
            }
        };

        let rollup_id = config.id.clone();
        self.configs.insert(rollup_id.clone(), config);
        self.verifiers.insert(rollup_id, verifier);

        Ok(())
    }

    pub fn get_verifier(&self, rollup_id: &RollupId) -> anyhow::Result<Arc<dyn VerifierCircuit>> {
        self.verifiers
            .get(rollup_id.as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Verifier not found for rollup: {}", rollup_id))
    }

    pub fn get_config(&self, rollup_id: &RollupId) -> anyhow::Result<&RollupConfig> {
        self.configs
            .get(rollup_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("Config not found for rollup: {}", rollup_id))
    }

    pub fn list_rollups(&self) -> Vec<RollupId> {
        self.configs
            .keys()
            .map(|id: &String| RollupId::new(id.clone()))
            .collect()
    }

    pub fn is_active(&self, rollup_id: &RollupId) -> bool {
        self.configs
            .get(rollup_id.as_str())
            .map(|c| c.active)
            .unwrap_or(false)
    }

    pub fn get_active_rollups(&self) -> Vec<RollupId> {
        self.configs
            .iter()
            .filter(|(_id, config)| config.active)
            .map(|(id, _config): (&String, &RollupConfig)| RollupId::new(id.clone()))
            .collect()
    }

    pub fn get_rollups_by_proof_system(&self, ps: ProofSystem) -> Vec<RollupId> {
        self.configs
            .iter()
            .filter(|(_id, config)| config.proof_system == ps)
            .map(|(id, _config): (&String, &RollupConfig)| RollupId::new(id.clone()))
            .collect()
    }

    pub fn deactivate_rollup(&mut self, rollup_id: &RollupId) -> anyhow::Result<()> {
        if let Some(config) = self.configs.get_mut(rollup_id.as_str()) {
            config.active = false;
            Ok(())
        } else {
            anyhow::bail!("Rollup not found: {}", rollup_id)
        }
    }

    pub fn reactivate_rollup(&mut self, rollup_id: &RollupId) -> anyhow::Result<()> {
        if let Some(config) = self.configs.get_mut(rollup_id.as_str()) {
            config.active = true;
            Ok(())
        } else {
            anyhow::bail!("Rollup not found: {}", rollup_id)
        }
    }

    pub fn get_registry_stats(&self) -> RegistryStats {
        let total = self.configs.len();
        let active = self.configs.values().filter(|c| c.active).count();

        let by_proof_system: HashMap<ProofSystem, usize> = self
            .configs
            .values()
            .fold(HashMap::new(), |mut acc: HashMap<ProofSystem, usize>, config| {
                *acc.entry(config.proof_system).or_insert(0) += 1;
                acc
            });

        RegistryStats {
            total_rollups: total,
            active_rollups: active,
            by_proof_system,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_rollups: usize,
    pub active_rollups: usize,
    pub by_proof_system: HashMap<ProofSystem, usize>,
}

impl Default for CircuitRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = CircuitRegistry::new();
        assert_eq!(registry.list_rollups().len(), 0);
    }

    #[test]
    fn test_registry_load_from_config() {
        let result = CircuitRegistry::load_from_config();
        assert!(result.is_ok());

        let reg = result.unwrap();
        assert!(reg.list_rollups().len() >= 5);
    }

    #[test]
    fn test_get_active_rollups() {
        let registry = CircuitRegistry::load_from_config().unwrap();
        let active = registry.get_active_rollups();
        assert!(!active.is_empty());
    }

    #[test]
    fn test_get_verifier_returns_arc() {
        let registry = CircuitRegistry::load_from_config().unwrap();
        let rollup_id = RollupId::from(crate::registry::rollup_ids::ZKSYNC_ERA);
        let result = registry.get_verifier(&rollup_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry_stats() {
        let registry = CircuitRegistry::load_from_config().unwrap();
        let stats = registry.get_registry_stats();
        assert_eq!(stats.total_rollups, stats.active_rollups);
        assert!(stats.by_proof_system.len() == 5);
    }
}