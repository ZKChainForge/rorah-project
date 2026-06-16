pub mod boojum;
pub mod cairo;
pub mod common;
pub mod groth16;
pub mod halo2;
pub mod plonky2;
pub mod registry;
pub mod traits;

pub use registry::{CircuitRegistry, RollupConfig, RollupId};
pub use traits::{
    Circuit, CircuitMetadata, ProofData, ProofMetadata, ProofSystem, VerifierCircuit,
    VerifierMetrics,
};

use std::sync::Arc;

pub struct CircuitLibrary {
    registry: CircuitRegistry,
}

impl CircuitLibrary {
    pub fn new() -> anyhow::Result<Self> {
        let registry = CircuitRegistry::load_from_config()?;
        Ok(CircuitLibrary { registry })
    }

    pub fn get_verifier(
        &self,
        rollup_id: &RollupId,
    ) -> anyhow::Result<Arc<dyn VerifierCircuit>> {
        self.registry.get_verifier(rollup_id)
    }

    pub fn register_rollup(&mut self, config: RollupConfig) -> anyhow::Result<()> {
        self.registry.register_rollup(config)
    }

    pub fn list_rollups(&self) -> Vec<RollupId> {
        self.registry.list_rollups()
    }
}

impl Default for CircuitLibrary {
    fn default() -> Self {
        Self::new().expect("Failed to initialize circuit library")
    }
}