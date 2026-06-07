//! Hexadecimal encoding/decoding utilities.

use hex::{FromHex, ToHex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HexError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(String),

    #[error("Hex decode error: {0}")]
    DecodeError(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, HexError>;

/// Encode bytes to hexadecimal string with 0x prefix.
pub fn encode_hex(bytes: &[u8]) -> String {
    format!("0x{}", bytes.encode_hex::<String>())
}

/// Encode bytes to hexadecimal string without prefix.
pub fn encode_hex_no_prefix(bytes: &[u8]) -> String {
    bytes.encode_hex::<String>()
}

/// Decode hexadecimal string (with or without 0x prefix).
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    Vec::from_hex(hex_str).map_err(HexError::from)
}

/// Validate hex string format.
pub fn is_valid_hex(hex_str: &str) -> bool {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    hex_str.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78];
        let hex = encode_hex(&bytes);
        assert_eq!(hex, "0x12345678");

        let decoded = decode_hex(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_no_prefix() {
        let bytes = vec![0xab, 0xcd];
        let hex = encode_hex_no_prefix(&bytes);
        assert_eq!(hex, "abcd");

        let decoded = decode_hex("abcd").unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_validation() {
        assert!(is_valid_hex("0x123abc"));
        assert!(is_valid_hex("123abc"));
        assert!(!is_valid_hex("0xGGG"));
        assert!(!is_valid_hex("not_hex"));
    }
}