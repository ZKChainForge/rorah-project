//! Rank-1 Constraint System module.

pub mod constraint;
pub mod instance;
pub mod relaxed;
pub mod witness;

// Re-export all public types
pub use constraint::{Constraint, LinearCombination};
pub use instance::{R1CSInstance, SparseMatrix};
pub use relaxed::RelaxedR1CSInstance;
pub use witness::Witness;