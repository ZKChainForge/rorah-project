//! Goldilocks field implementation.
//!
//! The Goldilocks field is used by Plonky2 (Polygon zkEVM).
//! When RORAH folds a Plonky2 proof, witness elements from the
//! Goldilocks field must be converted to BN254 for the R1CS wrapper.
//!
//! # Prime
//! p = 2^64 - 2^32 + 1 = 18446744069414584321
//!
//! # Properties
//! - Fits in a single u64 (fast arithmetic)
//! - No bignum needed
//! - SIMD-friendly for batch operations
//!
//! # Security Note
//! Goldilocks provides ~64-bit security in its native form.
//! When embedded into BN254 R1CS, the full 128-bit security of BN254 applies.

use super::traits::FieldElement as FieldElementTrait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// The Goldilocks prime: p = 2^64 - 2^32 + 1
pub const GOLDILOCKS_PRIME: u64 = 0xFFFF_FFFF_0000_0001;

/// Goldilocks field element.
///
/// All arithmetic is done modulo GOLDILOCKS_PRIME.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldilocksFieldElement(u64);

impl GoldilocksFieldElement {
    /// Create directly from a u64 value (must be < GOLDILOCKS_PRIME).
    pub fn new(value: u64) -> Self {
        Self(value % GOLDILOCKS_PRIME)
    }

    /// Raw u64 value.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Reduce a u128 modulo the Goldilocks prime.
    ///
    /// Uses the special structure of p = 2^64 - 2^32 + 1
    /// for fast reduction without division.
    fn reduce_u128(x: u128) -> u64 {
        // Split x into low and high 64-bit parts
        let lo = x as u64;
        let hi = (x >> 64) as u64;

        // Use: 2^64 ≡ 2^32 - 1 (mod p)
        // So: hi * 2^64 ≡ hi * (2^32 - 1) (mod p)
        let hi_lo = hi as u128;
        let hi_contribution = (hi_lo << 32).wrapping_sub(hi_lo);

        let result = (lo as u128) + hi_contribution;

        // Result might still be >= p, reduce once more
        let result_lo = result as u64;
        let result_hi = (result >> 64) as u64;

        if result_hi > 0 {
            // Another reduction needed
            let extra = (result_hi as u128) * ((1u128 << 32) - 1);
            let final_val = result_lo as u128 + extra;
            (final_val as u64) % GOLDILOCKS_PRIME
        } else if result_lo >= GOLDILOCKS_PRIME {
            result_lo - GOLDILOCKS_PRIME
        } else {
            result_lo
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FieldElement trait implementation
// ─────────────────────────────────────────────────────────────────────────────

impl FieldElementTrait for GoldilocksFieldElement {
    fn zero() -> Self {
        Self(0)
    }

    fn one() -> Self {
        Self(1)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn is_one(&self) -> bool {
        self.0 == 1
    }

    fn negate(&self) -> Self {
        if self.0 == 0 {
            Self(0)
        } else {
            Self(GOLDILOCKS_PRIME - self.0)
        }
    }

    fn double(&self) -> Self {
        let doubled = self.0 as u128 * 2;
        Self(Self::reduce_u128(doubled))
    }

    fn square(&self) -> Self {
        let squared = self.0 as u128 * self.0 as u128;
        Self(Self::reduce_u128(squared))
    }

    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        // Fermat's little theorem: a^(p-1) = 1 (mod p)
        // So: a^(-1) = a^(p-2) (mod p)
        Some(self.pow_u64(GOLDILOCKS_PRIME - 2))
    }

    fn pow_u64(&self, mut exp: u64) -> Self {
        let mut base = *self;
        let mut result = Self::one();

        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base.square();
            exp >>= 1;
        }

        result
    }

    fn to_bytes(&self) -> [u8; 32] {
        // Goldilocks fits in 8 bytes, pad to 32 for interface compatibility
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&self.0.to_be_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        // Read from last 8 bytes (value must fit in u64)
        let mut value_bytes = [0u8; 8];
        value_bytes.copy_from_slice(&bytes[24..32]);
        let value = u64::from_be_bytes(value_bytes);

        // Check upper bytes are all zero
        if bytes[..24].iter().any(|&b| b != 0) {
            return None;
        }

        // Validate value is in field
        if value >= GOLDILOCKS_PRIME {
            return None;
        }

        Some(Self(value))
    }

    fn from_u64(value: u64) -> Self {
        Self(value % GOLDILOCKS_PRIME)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic operators
// ─────────────────────────────────────────────────────────────────────────────

impl Add for GoldilocksFieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let sum = self.0 as u128 + rhs.0 as u128;
        Self(Self::reduce_u128(sum))
    }
}

impl Sub for GoldilocksFieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            Self(self.0 - rhs.0)
        } else {
            Self(GOLDILOCKS_PRIME - rhs.0 + self.0)
        }
    }
}

impl Mul for GoldilocksFieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let product = self.0 as u128 * rhs.0 as u128;
        Self(Self::reduce_u128(product))
    }
}

impl Neg for GoldilocksFieldElement {
    type Output = Self;
    fn neg(self) -> Self {
        self.negate()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversion: Goldilocks → BN254
// ─────────────────────────────────────────────────────────────────────────────

impl GoldilocksFieldElement {
    /// Convert to BN254 field element.
    ///
    /// Since Goldilocks prime < BN254 prime, this is always safe.
    pub fn to_bn254(&self) -> crate::field::bn254::BN254FieldElement {
        crate::field::bn254::BN254FieldElement::from_u64(self.0)
    }

    /// Try to convert from BN254 field element.
    ///
    /// Returns None if the BN254 value exceeds the Goldilocks prime.
    pub fn from_bn254(
        elem: &crate::field::bn254::BN254FieldElement,
    ) -> Option<Self> {
        let bytes = elem.to_bytes();

        // Upper 24 bytes must be zero
        if bytes[..24].iter().any(|&b| b != 0) {
            return None;
        }

        let mut val_bytes = [0u8; 8];
        val_bytes.copy_from_slice(&bytes[24..]);
        let value = u64::from_be_bytes(val_bytes);

        if value >= GOLDILOCKS_PRIME {
            return None;
        }

        Some(Self(value))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Debug for GoldilocksFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GL({})", self.0)
    }
}

impl fmt::Display for GoldilocksFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default
// ─────────────────────────────────────────────────────────────────────────────

impl Default for GoldilocksFieldElement {
    fn default() -> Self {
        Self::zero()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::traits::run_field_axiom_tests;
    use proptest::prelude::*;

    #[test]
    fn test_goldilocks_field_axioms() {
        run_field_axiom_tests::<GoldilocksFieldElement>();
    }

    #[test]
    fn test_prime_value() {
        assert_eq!(GOLDILOCKS_PRIME, (1u64 << 64) - (1u64 << 32) + 1);
    }

    #[test]
    fn test_wrap_around() {
        let p_minus_one = GoldilocksFieldElement::new(GOLDILOCKS_PRIME - 1);
        let one = GoldilocksFieldElement::one();
        assert_eq!(p_minus_one + one, GoldilocksFieldElement::zero());
    }

    #[test]
    fn test_subtraction_wrap() {
        let zero = GoldilocksFieldElement::zero();
        let one = GoldilocksFieldElement::one();
        let result = zero - one;
        assert_eq!(result, GoldilocksFieldElement::new(GOLDILOCKS_PRIME - 1));
    }

    #[test]
    fn test_inverse() {
        let x = GoldilocksFieldElement::from_u64(7);
        let inv = x.inverse().unwrap();
        assert_eq!(x * inv, GoldilocksFieldElement::one());
    }

    #[test]
    fn test_zero_no_inverse() {
        assert!(GoldilocksFieldElement::zero().inverse().is_none());
    }

    #[test]
    fn test_bytes_roundtrip() {
        let x = GoldilocksFieldElement::from_u64(123456789);
        let bytes = x.to_bytes();
        let recovered = GoldilocksFieldElement::from_bytes(&bytes).unwrap();
        assert_eq!(x, recovered);
    }

    #[test]
    fn test_to_bn254_and_back() {
        let gl = GoldilocksFieldElement::from_u64(42);
        let bn = gl.to_bn254();
        let back = GoldilocksFieldElement::from_bn254(&bn).unwrap();
        assert_eq!(gl, back);
    }

    #[test]
    fn test_bn254_too_large_rejected() {
        // BN254 prime >> Goldilocks prime
        // Create a BN254 element with value > Goldilocks prime
        let large_bn = crate::field::bn254::BN254FieldElement::from_u64(GOLDILOCKS_PRIME + 1);
        let result = GoldilocksFieldElement::from_bn254(&large_bn);
        assert!(result.is_none());
    }

    proptest! {
        #[test]
        fn prop_gl_add_commutative(a: u32, b: u32) {
            let fa = GoldilocksFieldElement::from_u64(a as u64);
            let fb = GoldilocksFieldElement::from_u64(b as u64);
            prop_assert_eq!(fa + fb, fb + fa);
        }

        #[test]
        fn prop_gl_mul_commutative(a: u32, b: u32) {
            let fa = GoldilocksFieldElement::from_u64(a as u64);
            let fb = GoldilocksFieldElement::from_u64(b as u64);
            prop_assert_eq!(fa * fb, fb * fa);
        }

        #[test]
        fn prop_gl_distributive(a: u32, b: u32, c: u32) {
            let fa = GoldilocksFieldElement::from_u64(a as u64);
            let fb = GoldilocksFieldElement::from_u64(b as u64);
            let fc = GoldilocksFieldElement::from_u64(c as u64);
            prop_assert_eq!(fa * (fb + fc), fa * fb + fa * fc);
        }

        #[test]
        fn prop_gl_inverse(a: u32) {
            if a == 0 { return Ok(()); }
            let fa = GoldilocksFieldElement::from_u64(a as u64);
            let inv = fa.inverse().unwrap();
            prop_assert_eq!(fa * inv, GoldilocksFieldElement::one());
        }

        #[test]
        fn prop_gl_bytes_roundtrip(a: u32) {
            let fa = GoldilocksFieldElement::from_u64(a as u64);
            let bytes = fa.to_bytes();
            let recovered = GoldilocksFieldElement::from_bytes(&bytes).unwrap();
            prop_assert_eq!(fa, recovered);
        }
    }
}