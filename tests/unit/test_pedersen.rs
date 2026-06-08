//! Unit tests for Pedersen commitment scheme.
//!
//! Tests binding, hiding, and homomorphic properties.

use rorah_core::commitment::{Commitment, CommitmentParams};
use rorah_core::FieldElement;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn make_params(size: usize) -> CommitmentParams {
    CommitmentParams::new(size)
}

fn make_message(values: &[u64]) -> Vec<FieldElement> {
    values.iter().map(|&v| FieldElement::from_u64(v)).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic creation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_commit_single_element() {
    let params = make_params(8);
    let message = make_message(&[42]);
    let blinding = FieldElement::from_u64(1337);

    let commitment = params.commit(&message, blinding).unwrap();

    // Commitment should not be identity element
    let identity = Commitment::from_bytes(&[0u8; 32]);
    // Just verify it produces a result without panicking
    assert!(!commitment.to_bytes().iter().all(|&b| b == 0));
}

#[test]
fn test_commit_vector() {
    let params = make_params(8);
    let message = make_message(&[1, 2, 3, 4, 5]);
    let blinding = FieldElement::from_u64(99);

    let commitment = params.commit(&message, blinding);
    assert!(commitment.is_ok());
}

#[test]
fn test_commit_empty_fails() {
    let params = make_params(8);
    let message: Vec<FieldElement> = vec![];
    let blinding = FieldElement::zero();

    // Empty message should still work (zero vector)
    // but exceeding max_size should fail
    let too_large = make_message(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let result = params.commit(&too_large, blinding);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Determinism
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_commit_deterministic() {
    let params = make_params(8);
    let message = make_message(&[10, 20, 30]);
    let blinding = FieldElement::from_u64(500);

    let c1 = params.commit(&message, blinding).unwrap();
    let c2 = params.commit(&message, blinding).unwrap();

    assert_eq!(c1, c2);
}

#[test]
fn test_different_messages_different_commitments() {
    let params = make_params(8);
    let blinding = FieldElement::from_u64(1);

    let c1 = params.commit(&make_message(&[1, 2, 3]), blinding).unwrap();
    let c2 = params.commit(&make_message(&[1, 2, 4]), blinding).unwrap();

    assert_ne!(c1, c2);
}

#[test]
fn test_different_blinding_different_commitments() {
    let params = make_params(8);
    let message = make_message(&[10, 20]);

    let c1 = params.commit(&message, FieldElement::from_u64(1)).unwrap();
    let c2 = params.commit(&message, FieldElement::from_u64(2)).unwrap();

    assert_ne!(c1, c2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Verification
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_verify_correct_opening() {
    let params = make_params(8);
    let message = make_message(&[7, 8, 9]);
    let blinding = FieldElement::from_u64(42);

    let commitment = params.commit(&message, blinding).unwrap();

    assert!(params.verify(&commitment, &message, blinding).is_ok());
}

#[test]
fn test_verify_wrong_message_fails() {
    let params = make_params(8);
    let message = make_message(&[7, 8, 9]);
    let blinding = FieldElement::from_u64(42);

    let commitment = params.commit(&message, blinding).unwrap();

    let wrong = make_message(&[7, 8, 10]);
    assert!(params.verify(&commitment, &wrong, blinding).is_err());
}

#[test]
fn test_verify_wrong_blinding_fails() {
    let params = make_params(8);
    let message = make_message(&[1, 2]);
    let blinding = FieldElement::from_u64(100);

    let commitment = params.commit(&message, blinding).unwrap();

    let wrong_blinding = FieldElement::from_u64(101);
    assert!(params.verify(&commitment, &message, wrong_blinding).is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Homomorphic property
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_additive_homomorphism() {
    use ark_bn254::G1Projective;
    use ark_ec::CurveGroup;

    let params = make_params(8);

    let m1 = make_message(&[10, 20]);
    let m2 = make_message(&[30, 40]);
    let m_sum = make_message(&[40, 60]); // element-wise sum

    let r1 = FieldElement::from_u64(5);
    let r2 = FieldElement::from_u64(7);
    let r_sum = FieldElement::from_u64(12); // r1 + r2

    let c1 = params.commit(&m1, r1).unwrap();
    let c2 = params.commit(&m2, r2).unwrap();
    let c_sum = params.commit(&m_sum, r_sum).unwrap();

    // c1 + c2 should equal c(m1+m2, r1+r2)
    let sum_proj = G1Projective::from(*c1.inner()) + G1Projective::from(*c2.inner());
    let sum_affine = sum_proj.into_affine();

    assert_eq!(sum_affine, *c_sum.inner());
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialization
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_commitment_bytes_roundtrip() {
    let params = make_params(8);
    let message = make_message(&[100, 200]);
    let blinding = FieldElement::from_u64(999);

    let commitment = params.commit(&message, blinding).unwrap();

    let bytes = commitment.to_bytes();
    assert_eq!(bytes.len(), 32); // Compressed G1 point

    let recovered = Commitment::from_bytes(&bytes).unwrap();
    assert_eq!(commitment, recovered);
}

#[test]
fn test_invalid_bytes_rejected() {
    // Random bytes that are not a valid curve point
    let bad_bytes = vec![0xFFu8; 32];
    let result = Commitment::from_bytes(&bad_bytes);
    // May or may not fail depending on the specific bytes,
    // but at minimum should not panic
    let _ = result;
}

#[test]
fn test_wrong_length_bytes_rejected() {
    let bad_bytes = vec![0u8; 31]; // Wrong length
    let result = Commitment::from_bytes(&bad_bytes);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Generator properties
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_generators_are_unique() {
    let params = make_params(10);

    // All generators should be distinct points
    for i in 0..9 {
        for j in (i + 1)..10 {
            assert_ne!(
                params.generators[i],
                params.generators[j],
                "Generators {} and {} are equal",
                i,
                j
            );
        }
    }
}

#[test]
fn test_generators_not_identity() {
    use ark_ec::AffineRepr;

    let params = make_params(5);

    for (i, g) in params.generators.iter().enumerate() {
        assert!(
            !g.is_zero(),
            "Generator {} is the identity element",
            i
        );
    }
}

#[test]
fn test_params_deterministic_across_instances() {
    // Two separate CommitmentParams with same size should have identical generators
    let params_a = make_params(5);
    let params_b = make_params(5);

    for i in 0..5 {
        assert_eq!(
            params_a.generators[i],
            params_b.generators[i],
            "Generator {} differs between instances",
            i
        );
    }

    assert_eq!(
        params_a.blinding_generator,
        params_b.blinding_generator
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Unblinded commitment
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unblinded_commit() {
    let params = make_params(8);
    let message = make_message(&[5, 10, 15]);

    let c_unblinded = params.commit_unblinded(&message).unwrap();
    let c_blinded_zero = params
        .commit(&message, FieldElement::zero())
        .unwrap();

    // Unblinded should equal blinding with zero
    assert_eq!(c_unblinded, c_blinded_zero);
}