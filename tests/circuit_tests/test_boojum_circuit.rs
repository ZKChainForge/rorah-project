use rorah_circuits::boojum::{BoojumVK, BoojumVerifier};
use rorah_circuits::traits::{VerifierCircuit, ProofData, ProofSystem};
use rorah_circuits::boojum::BoojumProofData;

#[test]
fn test_boojum_verifier_initialization() {
    let vk = BoojumVK::new(vec![0u8; 32], 1000, 2048);
    let result = BoojumVerifier::new(vk);
    assert!(result.is_ok());
}

#[test]
fn test_boojum_constraint_metrics() {
    let vk = BoojumVK::new(vec![0u8; 32], 1000, 2048);
    let verifier = BoojumVerifier::new(vk).unwrap();

    assert_eq!(verifier.constraint_count(), 5_200_000);
    assert_eq!(verifier.public_input_count(), 64);
    assert_eq!(verifier.proof_system_name(), "boojum");
}

#[test]
fn test_boojum_proof_validation() {
    let proof = BoojumProofData {
        fri_layers: vec![],
        merkle_paths: vec![],
        lde_evaluations: vec![],
        quotient_poly: vec![],
        public_inputs: vec![],
    };

    let result = proof.validate();
    assert!(result.is_err());
}

#[test]
fn test_boojum_witness_generation() {
    let vk = BoojumVK::new(vec![0u8; 32], 1000, 2048);
    let verifier = BoojumVerifier::new(vk).unwrap();

    let proof_data = BoojumProofData {
        fri_layers: vec![],
        merkle_paths: vec![vec![vec![0u8; 32]]],
        lde_evaluations: vec![vec![1u8; 64]],
        quotient_poly: vec![1u8; 64],
        public_inputs: vec![0u8; 32],
    };

    let result = verifier.generate_witness(&ProofData::Boojum(proof_data), &[]);
    assert!(result.is_ok());
}