//! Transcript module for Fiat-Shamir non-interactive proofs.
//!
//! # Structure
//! - `fiat_shamir.rs` - Main transcript with absorb/squeeze API
//! - `poseidon.rs`    - Poseidon hash (ZK-friendly, for future circuit use)
//!
//! # Security
//! - Domain separation prevents cross-protocol attacks
//! - Absorb-then-squeeze ordering prevents length-extension attacks
//! - Labels on all absorbed values prevent reordering attacks

pub mod fiat_shamir;
pub mod poseidon;

pub use fiat_shamir::Transcript;
pub use poseidon::PoseidonHash;