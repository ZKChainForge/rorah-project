//! Byte manipulation utilities.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BytesError {
    #[error("Invalid length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, BytesError>;

/// Convert u64 to big-endian bytes.
pub fn u64_to_be_bytes(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// Convert big-endian bytes to u64.
pub fn u64_from_be_bytes(bytes: &[u8]) -> Result<u64> {
    if bytes.len() != 8 {
        return Err(BytesError::InvalidLength {
            expected: 8,
            actual: bytes.len(),
        });
    }

    let mut array = [0u8; 8];
    array.copy_from_slice(bytes);
    Ok(u64::from_be_bytes(array))
}

/// Convert usize to variable-length bytes (up to 8 bytes).
pub fn usize_to_bytes(value: usize) -> Vec<u8> {
    (value as u64).to_be_bytes().to_vec()
}

/// Pad bytes to target length with zeros on the left.
pub fn pad_left(bytes: &[u8], target_len: usize) -> Vec<u8> {
    if bytes.len() >= target_len {
        return bytes.to_vec();
    }

    let mut padded = vec![0u8; target_len - bytes.len()];
    padded.extend_from_slice(bytes);
    padded
}

/// Concatenate multiple byte slices.
pub fn concat_bytes(slices: &[&[u8]]) -> Vec<u8> {
    let total_len: usize = slices.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_len);

    for slice in slices {
        result.extend_from_slice(slice);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_conversion() {
        let value = 12345u64;
        let bytes = u64_to_be_bytes(value);
        let recovered = u64_from_be_bytes(&bytes).unwrap();
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_pad_left() {
        let bytes = vec![1, 2, 3];
        let padded = pad_left(&bytes, 5);
        assert_eq!(padded, vec![0, 0, 1, 2, 3]);
    }

    #[test]
    fn test_concat_bytes() {
        let a = &[1, 2][..];
        let b = &[3, 4, 5][..];
        let c = &[6][..];

        let result = concat_bytes(&[a, b, c]);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }
}