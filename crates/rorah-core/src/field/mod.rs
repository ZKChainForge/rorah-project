//! Field arithmetic module.
//!
//! Provides field element implementations for cryptographic operations.
//!
//! # Structure
//! - `traits.rs`    - FieldElement trait definition
//! - `bn254.rs`     - BN254 scalar field implementation
//! - `goldilocks.rs`- Goldilocks field implementation
//!
//! # Security
//! All field operations are implemented with constant-time guarantees
//! where possible to prevent timing side-channel attacks.

pub mod bn254;
pub mod goldilocks;
pub mod traits;

// Re-export primary types
pub use bn254::BN254FieldElement;
pub use goldilocks::GoldilocksFieldElement;
pub use traits::FieldElement;