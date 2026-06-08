//! Unit tests for field arithmetic operations.

use rorah_core::FieldElement;

#[test]
fn test_basic_arithmetic() {
    let a = FieldElement::from_u64(5);
    let b = FieldElement::from_u64(3);

    // Addition
    assert_eq!(a + b, FieldElement::from_u64(8));

    // Subtraction
    assert_eq!(a - b, FieldElement::from_u64(2));

    // Multiplication
    assert_eq!(a * b, FieldElement::from_u64(15));
}

#[test]
fn test_zero_identity() {
    let zero = FieldElement::zero();
    let x = FieldElement::from_u64(42);

    assert_eq!(x + zero, x);
    assert_eq!(x - zero, x);
    assert_eq!(x * zero, zero);
}

#[test]
fn test_one_identity() {
    let one = FieldElement::one();
    let x = FieldElement::from_u64(42);

    assert_eq!(x * one, x);
}

#[test]
fn test_inverse() {
    let x = FieldElement::from_u64(7);
    let x_inv = x.inverse().unwrap();

    assert_eq!(x * x_inv, FieldElement::one());
}

#[test]
fn test_zero_has_no_inverse() {
    let zero = FieldElement::zero();
    assert!(zero.inverse().is_none());
}

#[test]
fn test_negation() {
    let x = FieldElement::from_u64(5);
    let neg_x = x.negate();

    assert_eq!(x + neg_x, FieldElement::zero());
}

#[test]
fn test_exponentiation() {
    let x = FieldElement::from_u64(2);
    let x_cubed = x.pow(&[3u64]);

    assert_eq!(x_cubed, FieldElement::from_u64(8));
}

#[test]
fn test_square() {
    let x = FieldElement::from_u64(5);
    assert_eq!(x.square(), FieldElement::from_u64(25));
}

#[test]
fn test_double() {
    let x = FieldElement::from_u64(7);
    assert_eq!(x.double(), FieldElement::from_u64(14));
}