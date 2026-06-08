//! Unit tests for R1CS constraints and instances.

use rorah_core::r1cs::{
    constraint::{Constraint, LinearCombination},
    R1CSInstance, Witness,
};
use rorah_core::FieldElement;

#[test]
fn test_linear_combination_evaluation() {
    let mut lc = LinearCombination::zero();
    lc.add_term(0, FieldElement::from_u64(2));
    lc.add_term(1, FieldElement::from_u64(3));
    lc.add_term(2, FieldElement::from_u64(4));

    let witness = vec![
        FieldElement::from_u64(10),
        FieldElement::from_u64(20),
        FieldElement::from_u64(30),
    ];

    // 2*10 + 3*20 + 4*30 = 20 + 60 + 120 = 200
    let result = lc.evaluate(&witness);
    assert_eq!(result, FieldElement::from_u64(200));
}

#[test]
fn test_simple_multiplication_constraint() {
    // Constraint: a * b = c
    // Variables: [1, a, b, c]

    let mut a_lc = LinearCombination::zero();
    a_lc.add_term(1, FieldElement::one());

    let mut b_lc = LinearCombination::zero();
    b_lc.add_term(2, FieldElement::one());

    let mut c_lc = LinearCombination::zero();
    c_lc.add_term(3, FieldElement::one());

    let constraint = Constraint::new(a_lc, b_lc, c_lc);

    // Satisfying witness: a=6, b=7, c=42
    let witness = vec![
        FieldElement::one(),
        FieldElement::from_u64(6),
        FieldElement::from_u64(7),
        FieldElement::from_u64(42),
    ];

    assert!(constraint.is_satisfied(&witness));

    // Non-satisfying witness: a=6, b=7, c=43
    let bad_witness = vec![
        FieldElement::one(),
        FieldElement::from_u64(6),
        FieldElement::from_u64(7),
        FieldElement::from_u64(43),
    ];

    assert!(!constraint.is_satisfied(&bad_witness));
}

#[test]
fn test_r1cs_instance_validation() {
    // Simple constraint: x² = y
    let mut a = LinearCombination::zero();
    a.add_term(1, FieldElement::one());

    let mut b = LinearCombination::zero();
    b.add_term(1, FieldElement::one());

    let mut c = LinearCombination::zero();
    c.add_term(2, FieldElement::one());

    let constraint = Constraint::new(a, b, c);

    let public_inputs = vec![FieldElement::from_u64(5)];
    let instance =
        R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

    // Correct witness: [1, 5, 25]
    let witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(5),
            FieldElement::from_u64(25),
        ],
        2,
    )
    .unwrap();

    assert!(instance.is_satisfied(&witness).is_ok());

    // Wrong witness: [1, 5, 26]
    let bad_witness = Witness::new(
        vec![
            FieldElement::one(),
            FieldElement::from_u64(5),
            FieldElement::from_u64(26),
        ],
        2,
    )
    .unwrap();

    assert!(instance.is_satisfied(&bad_witness).is_err());
}