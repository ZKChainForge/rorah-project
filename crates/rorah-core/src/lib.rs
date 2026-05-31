//! RORAH Core - Nova folding engine for rollup proof aggregation.
//!
//! This crate implements the core Nova folding scheme used in RORAH
//! for circuit-agnostic proof aggregation.

pub mod commitment;
pub mod error;
pub mod field;
pub mod nova;
pub mod r1cs;
pub mod transcript;
pub mod types;

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports for convenience
// ─────────────────────────────────────────────────────────────────────────────

pub use commitment::params::CommitmentParams;
pub use commitment::pedersen::{PedersenCommitment, PedersenScheme};
pub use commitment::traits::Commitment;
pub use error::{Result, RorahError};
pub use field::bn254::BN254FieldElement;
pub use nova::{fold_instances, NovaAccumulator, NovaProof};
pub use r1cs::{R1CSInstance, RelaxedR1CSInstance, Witness};
pub use transcript::Transcript;

/// RORAH protocol version.
pub const PROTOCOL_VERSION: &str = "0.1.0";