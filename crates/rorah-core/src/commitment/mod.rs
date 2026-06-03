//! Commitment scheme module.
//!
//! Provides cryptographic commitment schemes used in Nova folding.
//!
//! # Structure
//! - `traits.rs`   - CommitmentScheme trait
//! - `params.rs`   - Commitment parameters (generators)
//! - `pedersen.rs` - Pedersen vector commitment implementation
//!
//! # Security
//! All commitment schemes are:
//! - Computationally binding (under discrete log)
//! - Perfectly hiding (when using random blinding)
//! - Additively homomorphic

pub mod params;
pub mod pedersen;
pub mod traits;

pub use params::CommitmentParams;
pub use pedersen::PedersenCommitment;
pub use traits::{CommitmentScheme, Opening};