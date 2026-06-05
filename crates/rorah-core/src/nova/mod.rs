//! Nova folding scheme implementation.
//!
//! Implements incremental verifiable computation via folding.

pub mod accumulator;
pub mod fold;
pub mod proof;

pub use accumulator::NovaAccumulator;
pub use fold::fold_instances;
pub use proof::NovaProof;