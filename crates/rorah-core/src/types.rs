//! Common type aliases used throughout the crate.

use crate::field::bn254::BN254FieldElement;

/// A vector of field elements.
pub type FieldVec = Vec<BN254FieldElement>;

/// Result type for cryptographic operations.
pub type CryptoResult<T> = crate::error::Result<T>;

/// Byte array of 32 bytes (field element size).
pub type Bytes32 = [u8; 32];

/// Commitment as 32 bytes.
pub type CommitmentBytes = Bytes32;