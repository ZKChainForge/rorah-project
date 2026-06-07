//! Serialization utilities for proofs and data structures.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, SerializeError>;

// ─────────────────────────────────────────────────────────────────────────────
// JSON
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize a value to JSON bytes.
pub fn json_to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(SerializeError::from)
}

/// Deserialize a value from JSON bytes.
pub fn json_from_bytes<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(SerializeError::from)
}

/// Serialize a value to a pretty-printed JSON string.
pub fn json_to_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(SerializeError::from)
}

/// Deserialize a value from a JSON string.
pub fn json_from_string<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(SerializeError::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// Binary (bincode)
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize a value to binary (bincode).
pub fn binary_encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(SerializeError::from)
}

/// Deserialize a value from binary (bincode).
pub fn binary_decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).map_err(SerializeError::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Sample {
        id:   u64,
        name: String,
    }

    fn sample() -> Sample {
        Sample { id: 42, name: "rorah".to_string() }
    }

    #[test]
    fn test_json_bytes_roundtrip() {
        let original = sample();
        let bytes    = json_to_bytes(&original).unwrap();
        let recovered: Sample = json_from_bytes(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_json_string_roundtrip() {
        let original = sample();
        let s        = json_to_string(&original).unwrap();
        let recovered: Sample = json_from_string(&s).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_binary_roundtrip() {
        let original = sample();
        let bytes    = binary_encode(&original).unwrap();
        let recovered: Sample = binary_decode(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_json_is_human_readable() {
        let original = sample();
        let s        = json_to_string(&original).unwrap();
        assert!(s.contains("rorah"));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_bad_bytes_returns_error() {
        let bad: &[u8] = b"not valid json";
        let result: Result<Sample> = json_from_bytes(bad);
        assert!(result.is_err());
    }
}