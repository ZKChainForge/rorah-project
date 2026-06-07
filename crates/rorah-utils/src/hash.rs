//! Cryptographic hashing utilities.

use sha3::{Digest, Keccak256, Sha3_256};

/// Compute Keccak256 hash (Ethereum-compatible).
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA3-256 hash.
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute hash of multiple data slices.
pub fn keccak256_concat(slices: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    for slice in slices {
        hasher.update(slice);
    }
    hasher.finalize().into()
}

/// Hash a string (UTF-8 encoded).
pub fn keccak256_str(s: &str) -> [u8; 32] {
    keccak256(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash1 = keccak256(data);
        let hash2 = keccak256(data);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Different input
        let hash3 = keccak256(b"hello world!");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_concat_hash() {
        let a = b"hello";
        let b = b" ";
        let c = b"world";

        let hash_concat = keccak256_concat(&[a, b, c]);
        let hash_direct = keccak256(b"hello world");

        assert_eq!(hash_concat, hash_direct);
    }
}