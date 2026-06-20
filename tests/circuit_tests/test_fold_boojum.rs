use rorah_circuits::registry::CircuitRegistry;
use rorah_circuits::traits::{ProofSystem, VerifierCircuit};

#[test]
fn test_boojum_in_registry() {
    let registry = CircuitRegistry::load_from_config();
    assert!(registry.is_ok());

    let reg = registry.unwrap();
    let boojum_rollups = reg.get_rollups_by_proof_system(ProofSystem::Boojum);

    assert!(!boojum_rollups.is_empty());
}

#[test]
fn test_mixed_proof_system_registry() {
    let registry = CircuitRegistry::load_from_config().unwrap();

    let boojum = registry.get_rollups_by_proof_system(ProofSystem::Boojum);
    let plonky2 = registry.get_rollups_by_proof_system(ProofSystem::Plonky2);
    let halo2 = registry.get_rollups_by_proof_system(ProofSystem::Halo2);
    let groth16 = registry.get_rollups_by_proof_system(ProofSystem::Groth16);
    let cairo = registry.get_rollups_by_proof_system(ProofSystem::Cairo);

    assert!(!boojum.is_empty());
    assert!(!plonky2.is_empty());
    assert!(!halo2.is_empty());
    assert!(!groth16.is_empty());
    assert!(!cairo.is_empty());
}

#[test]
fn test_registry_statistics() {
    let registry = CircuitRegistry::load_from_config().unwrap();
    let stats = registry.get_registry_stats();

    assert!(stats.total_rollups >= 5);
    assert!(stats.active_rollups >= 5);
    assert_eq!(stats.by_proof_system.len(), 5);
}

#[test]
fn test_verifier_lookup() {
    let registry = CircuitRegistry::load_from_config().unwrap();

    for rollup_id in registry.get_active_rollups() {
        let verifier_result = registry.get_verifier(&rollup_id);
        assert!(verifier_result.is_ok());

        let verifier = verifier_result.unwrap();
        assert!(verifier.constraint_count() > 0);
        assert!(verifier.public_input_count() > 0);
    }
}