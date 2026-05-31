//! Error types for RORAH core operations.
//!
//! Security: All errors include context but do not leak sensitive information.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RorahError {
    #[error("R1CS constraint not satisfied at index {index}: {reason}")]
    ConstraintNotSatisfied { index: usize, reason: String },

    #[error("Witness length mismatch: expected {expected}, got {actual}")]
    WitnessSizeMismatch { expected: usize, actual: usize },

    #[error("Public input mismatch at index {index}")]
    PublicInputMismatch { index: usize },

    #[error("Relaxed R1CS not satisfied: {reason}")]
    RelaxedR1CSNotSatisfied { reason: String },

    #[error("Invalid proof: {reason}")]
    InvalidProof { reason: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Commitment error: {0}")]
    CommitmentError(String),

    #[error("Transcript error: {0}")]
    TranscriptError(String),

    #[error("Matrix dimension mismatch: {details}")]
    DimensionMismatch { details: String },

    #[error("Field operation error: {0}")]
    FieldError(String),
}

pub type Result<T> = std::result::Result<T, RorahError>;

impl RorahError {
    pub fn is_security_critical(&self) -> bool {
        matches!(
            self,
            RorahError::InvalidProof { .. }
                | RorahError::PublicInputMismatch { .. }
                | RorahError::RelaxedR1CSNotSatisfied { .. }
        )
    }
}