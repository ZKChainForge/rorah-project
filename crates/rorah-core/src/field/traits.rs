//! FieldElement trait definition.
//!
//! Defines the interface that all field implementations must satisfy.
//! This enables generic code that works over any field.
//!
//! # Security
//! Implementors must ensure:
//! - All operations are constant-time where feasible
//! - No panics on valid inputs
//! - Proper handling of edge cases (zero, modulus boundary)

use std::fmt::Debug;
use std::ops::{Add, Mul, Neg, Sub};

/// Core trait for field element operations.
///
/// Any type implementing this trait can be used in R1CS constraints,
/// commitment schemes, and the Nova folding algorithm.
///
/// # Mathematical Requirements
/// Implementing type must form a finite field F_p where:
/// - Addition forms an abelian group
/// - Multiplication forms an abelian group (excluding zero)
/// - Distributivity holds: a(b+c) = ab + ac
pub trait FieldElement:
    Sized
    + Clone
    + Copy
    + Debug
    + PartialEq
    + Eq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + Send
    + Sync
    + 'static
{
    // ─────────────────────────────────────────────────────────────────
    // Constants
    // ─────────────────────────────────────────────────────────────────

    /// The additive identity: 0
    fn zero() -> Self;

    /// The multiplicative identity: 1
    fn one() -> Self;

    // ─────────────────────────────────────────────────────────────────
    // Predicates
    // ─────────────────────────────────────────────────────────────────

    /// Returns true if element is zero.
    ///
    /// Security: Should be implemented in constant time.
    fn is_zero(&self) -> bool;

    /// Returns true if element is one.
    fn is_one(&self) -> bool;

    // ─────────────────────────────────────────────────────────────────
    // Arithmetic
    // ─────────────────────────────────────────────────────────────────

    /// Additive inverse: -self
    fn negate(&self) -> Self;

    /// Double: 2 * self (optimized)
    fn double(&self) -> Self;

    /// Square: self * self (optimized)
    fn square(&self) -> Self;

    /// Multiplicative inverse.
    ///
    /// Returns None if self is zero (no inverse exists).
    fn inverse(&self) -> Option<Self>;

    /// Exponentiation by u64 scalar.
    fn pow_u64(&self, exp: u64) -> Self;

    // ─────────────────────────────────────────────────────────────────
    // Serialization
    // ─────────────────────────────────────────────────────────────────

    /// Serialize to canonical 32-byte big-endian representation.
    fn to_bytes(&self) -> [u8; 32];

    /// Deserialize from 32-byte big-endian representation.
    ///
    /// # Security
    /// Must validate input is a valid field element (less than modulus).
    fn from_bytes(bytes: &[u8; 32]) -> Option<Self>;

    // ─────────────────────────────────────────────────────────────────
    // Conversion helpers
    // ─────────────────────────────────────────────────────────────────

    /// Create from u64 (always valid, u64 < any field modulus we use).
    fn from_u64(value: u64) -> Self;

    /// Create from i64 (handles negative values as field negation).
    fn from_i64(value: i64) -> Self {
        if value >= 0 {
            Self::from_u64(value as u64)
        } else {
            Self::from_u64((-value) as u64).negate()
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Derived operations (provided with default implementations)
    // ─────────────────────────────────────────────────────────────────

    /// Subtract and assign: self = self - other
    fn sub_assign(&mut self, other: Self)
    where
        Self: Copy,
    {
        *self = *self - other;
    }

    /// Add and assign: self = self + other
    fn add_assign(&mut self, other: Self)
    where
        Self: Copy,
    {
        *self = *self + other;
    }

    /// Multiply and assign: self = self * other
    fn mul_assign(&mut self, other: Self)
    where
        Self: Copy,
    {
        *self = *self * other;
    }

    /// Inner product of two vectors: sum of element-wise products.
    ///
    /// Returns None if slices have different lengths.
    fn inner_product(a: &[Self], b: &[Self]) -> Option<Self> {
        if a.len() != b.len() {
            return None;
        }

        let result = a
            .iter()
            .zip(b.iter())
            .map(|(&ai, &bi)| ai * bi)
            .fold(Self::zero(), |acc, val| acc + val);

        Some(result)
    }

    /// Hadamard (element-wise) product of two vectors.
    ///
    /// Returns None if slices have different lengths.
    fn hadamard_product(a: &[Self], b: &[Self]) -> Option<Vec<Self>> {
        if a.len() != b.len() {
            return None;
        }

        Some(a.iter().zip(b.iter()).map(|(&ai, &bi)| ai * bi).collect())
    }

    /// Linear combination: sum of coeff_i * elem_i
    fn linear_combination(coeffs: &[Self], elems: &[Self]) -> Option<Self> {
        Self::inner_product(coeffs, elems)
    }

    /// Scale a vector: result[i] = scalar * v[i]
    fn scale_vector(scalar: Self, v: &[Self]) -> Vec<Self> {
        v.iter().map(|&vi| scalar * vi).collect()
    }

    /// Add two vectors element-wise.
    ///
    /// Returns None if lengths differ.
    fn add_vectors(a: &[Self], b: &[Self]) -> Option<Vec<Self>> {
        if a.len() != b.len() {
            return None;
        }

        Some(a.iter().zip(b.iter()).map(|(&ai, &bi)| ai + bi).collect())
    }

    /// Subtract two vectors element-wise.
    ///
    /// Returns None if lengths differ.
    fn sub_vectors(a: &[Self], b: &[Self]) -> Option<Vec<Self>> {
        if a.len() != b.len() {
            return None;
        }

        Some(a.iter().zip(b.iter()).map(|(&ai, &bi)| ai - bi).collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test utilities for any FieldElement implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Run standard field axiom tests against any FieldElement implementation.
///
/// Call this from the implementing module's test section.
#[cfg(test)]
pub fn run_field_axiom_tests<F: FieldElement>() {
    let zero = F::zero();
    let one = F::one();
    let two = F::from_u64(2);
    let three = F::from_u64(3);
    let five = F::from_u64(5);

    // Additive identity
    assert_eq!(zero + one, one, "0 + 1 = 1");
    assert_eq!(one + zero, one, "1 + 0 = 1");

    // Multiplicative identity
    assert_eq!(one * five, five, "1 * 5 = 5");
    assert_eq!(five * one, five, "5 * 1 = 5");

    // Additive inverse
    assert_eq!(one + one.negate(), zero, "1 + (-1) = 0");
    assert_eq!(five + five.negate(), zero, "5 + (-5) = 0");

    // Zero absorb
    assert_eq!(zero * five, zero, "0 * 5 = 0");

    // Commutativity
    assert_eq!(two + three, three + two, "addition commutative");
    assert_eq!(two * three, three * two, "multiplication commutative");

    // Associativity
    let a = F::from_u64(7);
    let b = F::from_u64(11);
    let c = F::from_u64(13);
    assert_eq!((a + b) + c, a + (b + c), "addition associative");
    assert_eq!((a * b) * c, a * (b * c), "multiplication associative");

    // Distributivity
    assert_eq!(a * (b + c), a * b + a * c, "distributive law");

    // Inverse
    let x = F::from_u64(17);
    let x_inv = x.inverse().unwrap();
    assert_eq!(x * x_inv, one, "x * x^-1 = 1");

    // Zero has no inverse
    assert!(zero.inverse().is_none(), "zero has no inverse");

    // Square
    let sq = F::from_u64(6);
    assert_eq!(sq.square(), F::from_u64(36), "6^2 = 36");

    // Double
    assert_eq!(F::from_u64(4).double(), F::from_u64(8), "double(4) = 8");

    // pow_u64
    assert_eq!(two.pow_u64(10), F::from_u64(1024), "2^10 = 1024");
    assert_eq!(F::from_u64(3).pow_u64(3), F::from_u64(27), "3^3 = 27");

    // Serialization roundtrip
    let val = F::from_u64(999999);
    let bytes = val.to_bytes();
    let recovered = F::from_bytes(&bytes).unwrap();
    assert_eq!(val, recovered, "serialization roundtrip");

    // Hadamard product
    let v1 = vec![F::from_u64(2), F::from_u64(3)];
    let v2 = vec![F::from_u64(4), F::from_u64(5)];
    let hp = F::hadamard_product(&v1, &v2).unwrap();
    assert_eq!(hp[0], F::from_u64(8));
    assert_eq!(hp[1], F::from_u64(15));

    // Inner product
    let ip = F::inner_product(&v1, &v2).unwrap();
    assert_eq!(ip, F::from_u64(23)); // 2*4 + 3*5

    println!("All field axiom tests passed.");
}