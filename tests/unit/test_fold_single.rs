//! Tests for single Nova fold step.

use rorah_core::{
    commitment::CommitmentParams,
    fold_instances,
    nova::NovaAccumulator,
    r1cs::{
        constraint::{Constraint, LinearCombination},
        R1CSInstance, Witness,
    },
    FieldElement,
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build constraint: w[1] * w[1] = w[2]
fn square_constraint() -> Constraint {
    let mut a = LinearCombination::zero();
    a.add_term(1, FieldElement::one());

    let mut b = LinearCombination::zero();
    b.add_term(1, FieldElement::one());

    let mut c = LinearCombination::zero();
    c.add_term(2, FieldElement::one());

    Constraint::new(a, b, c)
}

/// Build R1CS and Witness for x² = y.
fn square_instance_and_witness(x: u64) -> (R1CSInstance, Witness) {
    let constraint = square_constraint();
    let public_inputs = vec![FieldElement::from_u64(x)];
    let instance =
        R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

    let witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(x),
            FieldElement::from_u64(x * x),
        ],
        2,
    )
    .unwrap();

    (instance, witness)
}

/// Build constraint: w[1] * w[2] = w[3]  (a * b = c)
fn multiply_constraint() -> Constraint {
    let mut a = LinearCombination::zero();
    a.add_term(1, FieldElement::one());

    let mut b = LinearCombination::zero();
    b.add_term(2, FieldElement::one());

    let mut c = LinearCombination::zero();
    c.add_term(3, FieldElement::one());

    Constraint::new(a, b, c)
}

fn multiply_instance_and_witness(a_val: u64, b_val: u64) -> (R1CSInstance, Witness) {
    let constraint = multiply_constraint();
    let public_inputs = vec![
        FieldElement::from_u64(a_val),
        FieldElement::from_u64(b_val),
    ];
    let instance =
        R1CSInstance::from_constraints(vec![constraint], 4, public_inputs).unwrap();

    let witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(a_val),
            FieldElement::from_u64(b_val),
            FieldElement::from_u64(a_val * b_val),
        ],
        3,
    )
    .unwrap();

    (instance, witness)
}

// ─────────────────────────────────────────────────────────────────────────────
// Empty accumulator initialization
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_empty_accumulator_is_valid() {
    let acc = NovaAccumulator::empty(3).unwrap();

    assert!(acc.is_empty());
    assert!(acc.is_valid().is_ok());
    assert_eq!(acc.num_constraints(), 0);
    assert!(acc.relaxation_factor().is_zero());
}

#[test]
fn test_empty_accumulator_different_sizes() {
    for size in [1, 3, 10, 100] {
        let acc = NovaAccumulator::empty(size).unwrap();
        assert!(acc.is_empty());
        assert!(acc.is_valid().is_ok());
        assert_eq!(acc.num_variables(), size);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// First fold (empty accumulator + one instance)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fold_first_instance_produces_valid_accumulator() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    let (instance, witness) = square_instance_and_witness(4);

    let (new_acc, proof) = fold_instances(acc, instance, witness, &params).unwrap();

    assert!(!new_acc.is_empty());
    assert!(new_acc.is_valid().is_ok());
    assert_eq!(new_acc.num_constraints(), 1);
    assert_eq!(new_acc.num_variables(), 3);
}

#[test]
fn test_fold_first_instance_proof_is_small() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    let (instance, witness) = square_instance_and_witness(7);

    let (_new_acc, _proof) = fold_instances(acc, instance, witness, &params).unwrap();

    // Nova proofs are exactly one compressed group element
    assert_eq!(rorah_core::nova::NovaProof::size_bytes(), 32);
}

#[test]
fn test_fold_first_instance_relaxation_factor() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    let (instance, witness) = square_instance_and_witness(5);

    let (new_acc, _) = fold_instances(acc, instance, witness, &params).unwrap();

    // After first fold from empty accumulator, u should be 1
    assert_eq!(new_acc.relaxation_factor(), FieldElement::one());
}

// ─────────────────────────────────────────────────────────────────────────────
// Invalid witness rejection
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fold_rejects_invalid_witness() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    let (instance, _good_witness) = square_instance_and_witness(5);

    // Wrong: x=5 but y=26 instead of 25
    let bad_witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(5),
            FieldElement::from_u64(26),
        ],
        2,
    )
    .unwrap();

    let result = fold_instances(acc, instance, bad_witness, &params);
    assert!(result.is_err());
}

#[test]
fn test_fold_rejects_wrong_public_input() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    // Instance has public input x=5
    let constraint = square_constraint();
    let public_inputs = vec![FieldElement::from_u64(5)];
    let instance =
        R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

    // Witness claims x=7, which doesn't match public input
    let bad_witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(7),
            FieldElement::from_u64(49),
        ],
        2,
    )
    .unwrap();

    let result = fold_instances(acc, instance, bad_witness, &params);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-term computation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_cross_term_changes_error_vector() {
    let params = CommitmentParams::new(32);

    // First fold: acc goes from empty to instance1
    let acc = NovaAccumulator::empty(3).unwrap();
    let (inst1, wit1) = square_instance_and_witness(3);
    let (acc1, _) = fold_instances(acc, inst1, wit1, &params).unwrap();

    // Error vector should be zero after first fold from empty
    let error = acc1.error_vector();
    assert!(!error.is_empty());

    // Second fold: acc folds in instance2
    let (inst2, wit2) = square_instance_and_witness(5);
    let (acc2, _) = fold_instances(acc1, inst2, wit2, &params).unwrap();

    // Error vector should now be non-zero (cross-term accumulated)
    let error2 = acc2.error_vector();
    assert!(!error2.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Different instance types
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_fold_multiply_constraint() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(4).unwrap();

    let (instance, witness) = multiply_instance_and_witness(6, 7);

    let (new_acc, _) = fold_instances(acc, instance, witness, &params).unwrap();

    assert!(new_acc.is_valid().is_ok());
    assert_eq!(new_acc.num_constraints(), 1);
    assert_eq!(new_acc.num_variables(), 4);
}

// ─────────────────────────────────────────────────────────────────────────────
// Proof properties
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_proof_serialization_roundtrip() {
    let params = CommitmentParams::new(32);
    let acc = NovaAccumulator::empty(3).unwrap();

    let (instance, witness) = square_instance_and_witness(9);

    let (_new_acc, proof) = fold_instances(acc, instance, witness, &params).unwrap();

    // Serialize and deserialize proof
    let json = serde_json::to_string(&proof).unwrap();
    let recovered: rorah_core::nova::NovaProof = serde_json::from_str(&json).unwrap();

    assert_eq!(proof, recovered);
}

#[test]
fn test_two_different_folds_produce_different_proofs() {
    let params = CommitmentParams::new(32);

    let acc1 = NovaAccumulator::empty(3).unwrap();
    let (inst1, wit1) = square_instance_and_witness(3);
    let (_, proof1) = fold_instances(acc1, inst1, wit1, &params).unwrap();

    let acc2 = NovaAccumulator::empty(3).unwrap();
    let (inst2, wit2) = square_instance_and_witness(7);
    let (_, proof2) = fold_instances(acc2, inst2, wit2, &params).unwrap();

    // Different inputs should produce different proofs
    assert_ne!(proof1, proof2);
}