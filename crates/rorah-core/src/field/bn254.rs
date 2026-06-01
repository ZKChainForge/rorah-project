//! BN254 scalar field implementation.
//!
//! Prime:
//! p = 21888242871839275222246405745257275088548364400416034343698204186575808495617

use super::traits::FieldElement as FieldElementTrait;
use ark_bn254::Fr;
use ark_ec::AdditiveGroup;
use ark_ff::{Field, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{One, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};
use subtle::{Choice, ConstantTimeEq};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BN254FieldElement(pub(crate) Fr);

// ─────────────────────────────────────────────────────────────────────────────
// FieldElement trait implementation
// ─────────────────────────────────────────────────────────────────────────────

impl FieldElementTrait for BN254FieldElement {
    fn zero() -> Self {
        Self(Fr::zero())
    }

    fn one() -> Self {
        Self(Fr::one())
    }

    fn is_zero(&self) -> bool {
        bool::from(self.ct_eq(&Self::zero()))
    }

    fn is_one(&self) -> bool {
        bool::from(self.ct_eq(&Self::one()))
    }

    fn negate(&self) -> Self {
        Self(-self.0)
    }

    fn double(&self) -> Self {
        Self(self.0.double())
    }

    fn square(&self) -> Self {
        Self(self.0.square())
    }

    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        Field::inverse(&self.0).map(Self)
    }

    fn pow_u64(&self, exp: u64) -> Self {
        Self(self.0.pow([exp]))
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.0
            .serialize_compressed(&mut bytes[..])
            .expect("serialization to fixed buffer cannot fail");
        bytes
    }

    fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let elem = Fr::deserialize_compressed(&bytes[..]).ok()?;
        // Validate element is in field
        if elem.into_bigint() >= Fr::MODULUS {
            return None;
        }
        Some(Self(elem))
    }

    fn from_u64(value: u64) -> Self {
        Self(Fr::from(value))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Arithmetic
// ─────────────────────────────────────────────────────────────────────────────

impl Add for BN254FieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for BN254FieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul for BN254FieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Neg for BN254FieldElement {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Add<&BN254FieldElement> for BN254FieldElement {
    type Output = Self;
    fn add(self, rhs: &Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<&BN254FieldElement> for BN254FieldElement {
    type Output = Self;
    fn sub(self, rhs: &Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<&BN254FieldElement> for BN254FieldElement {
    type Output = Self;
    fn mul(self, rhs: &Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Constant-time equality
// ─────────────────────────────────────────────────────────────────────────────

impl ConstantTimeEq for BN254FieldElement {
    fn ct_eq(&self, other: &Self) -> Choice {
        // Use field element comparison
        let diff = self.0 - other.0;
        let is_zero = diff == Fr::zero();
        if is_zero {
            Choice::from(1u8)
        } else {
            Choice::from(0u8)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display / Debug
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Debug for BN254FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_bytes();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "BN254(0x{}...{})", &hex[..8], &hex[56..])
    }
}

impl fmt::Display for BN254FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.to_bytes();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "0x{}", hex)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Serde
// ─────────────────────────────────────────────────────────────────────────────

impl Serialize for BN254FieldElement {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let hex: String = self
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        s.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for BN254FieldElement {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let hex_str = String::deserialize(d)?;
        let bytes = hex::decode(&hex_str)
            .map_err(|e| serde::de::Error::custom(format!("hex decode: {}", e)))?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Self::from_bytes(&arr)
            .ok_or_else(|| serde::de::Error::custom("invalid BN254 field element"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

impl BN254FieldElement {
    pub(crate) fn inner(&self) -> &Fr {
        &self.0
    }

    pub(crate) fn from_inner(inner: Fr) -> Self {
        Self(inner)
    }

    pub fn modulus_str() -> &'static str {
        "21888242871839275222246405745257275088548364400416034343698204186575808495617"
    }

    pub fn num_bits() -> u32 {
        Fr::MODULUS_BIT_SIZE
    }
}

impl Default for BN254FieldElement {
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
    fn test_field_axioms() {
        run_field_axiom_tests::<BN254FieldElement>();
    }

    #[test]
    fn test_zero() {
        let z = BN254FieldElement::zero();
        assert!(z.is_zero());
        assert!(!z.is_one());
    }

    #[test]
    fn test_one() {
        let o = BN254FieldElement::one();
        assert!(o.is_one());
        assert!(!o.is_zero());
    }

    #[test]
    fn test_add() {
        let a = BN254FieldElement::from_u64(5);
        let b = BN254FieldElement::from_u64(3);
        assert_eq!(a + b, BN254FieldElement::from_u64(8));
    }

    #[test]
    fn test_sub() {
        let a = BN254FieldElement::from_u64(10);
        let b = BN254FieldElement::from_u64(4);
        assert_eq!(a - b, BN254FieldElement::from_u64(6));
    }

    #[test]
    fn test_mul() {
        let a = BN254FieldElement::from_u64(6);
        let b = BN254FieldElement::from_u64(7);
        assert_eq!(a * b, BN254FieldElement::from_u64(42));
    }

    #[test]
    fn test_inverse() {
        let x = BN254FieldElement::from_u64(7);
        let inv = x.inverse().unwrap();
        assert_eq!(x * inv, BN254FieldElement::one());
    }

    #[test]
    fn test_zero_no_inverse() {
        assert!(BN254FieldElement::zero().inverse().is_none());
    }

    #[test]
    fn test_negate() {
        let x = BN254FieldElement::from_u64(99);
        assert_eq!(x + x.negate(), BN254FieldElement::zero());
    }

    #[test]
    fn test_square() {
        let x = BN254FieldElement::from_u64(9);
        assert_eq!(x.square(), BN254FieldElement::from_u64(81));
    }

    #[test]
    fn test_double() {
        let x = BN254FieldElement::from_u64(11);
        assert_eq!(x.double(), BN254FieldElement::from_u64(22));
    }

    #[test]
    fn test_pow() {
        let x = BN254FieldElement::from_u64(2);
        assert_eq!(x.pow_u64(8), BN254FieldElement::from_u64(256));
    }

    #[test]
    fn test_bytes_roundtrip() {
        let x = BN254FieldElement::from_u64(987654321);
        let bytes = x.to_bytes();
        let recovered = BN254FieldElement::from_bytes(&bytes).unwrap();
        assert_eq!(x, recovered);
    }

    #[test]
    fn test_serde_roundtrip() {
        let x = BN254FieldElement::from_u64(42);
        let json = serde_json::to_string(&x).unwrap();
        let recovered: BN254FieldElement = serde_json::from_str(&json).unwrap();
        assert_eq!(x, recovered);
    }

    proptest! {
        #[test]
        fn prop_add_commutative(a: u64, b: u64) {
            let fa = BN254FieldElement::from_u64(a);
            let fb = BN254FieldElement::from_u64(b);
            prop_assert_eq!(fa + fb, fb + fa);
        }

        #[test]
        fn prop_mul_commutative(a: u64, b: u64) {
            let fa = BN254FieldElement::from_u64(a);
            let fb = BN254FieldElement::from_u64(b);
            prop_assert_eq!(fa * fb, fb * fa);
        }

        #[test]
        fn prop_distributive(a: u64, b: u64, c: u64) {
            let fa = BN254FieldElement::from_u64(a);
            let fb = BN254FieldElement::from_u64(b);
            let fc = BN254FieldElement::from_u64(c);
            prop_assert_eq!(fa * (fb + fc), fa * fb + fa * fc);
        }

        #[test]
        fn prop_negate(a: u64) {
            let fa = BN254FieldElement::from_u64(a);
            prop_assert_eq!(fa + fa.negate(), BN254FieldElement::zero());
        }

        #[test]
        fn prop_bytes_roundtrip(a: u64) {
            let fa = BN254FieldElement::from_u64(a);
            let bytes = fa.to_bytes();
            let recovered = BN254FieldElement::from_bytes(&bytes).unwrap();
            prop_assert_eq!(fa, recovered);
        }
    }
}