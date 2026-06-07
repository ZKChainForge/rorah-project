//! RORAH shared utilities.
//!
//! Provides byte helpers, hashing, hex encoding, serialization,
//! and timing utilities used across all RORAH crates.

pub mod bytes;
pub mod hash;
pub mod hex_utils;
pub mod serialization;
pub mod timer;

// ─────────────────────────────────────────────────────────────────────────────
// bytes module re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use bytes::BytesError;
pub use bytes::{
    concat_bytes,
    pad_left,
    u64_from_be_bytes,
    u64_to_be_bytes,
    usize_to_bytes,
};

// ─────────────────────────────────────────────────────────────────────────────
// hash module re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use hash::{
    keccak256,
    keccak256_concat,
    keccak256_str,
    sha3_256,
};

// ─────────────────────────────────────────────────────────────────────────────
// hex_utils module re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use hex_utils::HexError;
pub use hex_utils::{
    decode_hex,
    encode_hex,
    encode_hex_no_prefix,
    is_valid_hex,
};

// ─────────────────────────────────────────────────────────────────────────────
// serialization module re-exports
// NOTE: Function names match what's actually in serialization.rs
// ─────────────────────────────────────────────────────────────────────────────

pub use serialization::SerializeError;
pub use serialization::{
    binary_decode,      // was: from_binary
    binary_encode,      // was: to_binary
    json_from_bytes,    // MATCHES serialization.rs
    json_from_string,   // MATCHES serialization.rs
    json_to_bytes,      // MATCHES serialization.rs
    json_to_string,     // MATCHES serialization.rs
};

// ─────────────────────────────────────────────────────────────────────────────
// timer module re-exports
// ─────────────────────────────────────────────────────────────────────────────

pub use timer::{time_it, Timer};