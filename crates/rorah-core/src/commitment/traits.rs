//! CommitmentScheme trait definition.
//!
//! Abstracts over different commitment constructions so Nova can be
//! parameterized with any binding commitment scheme.

use crate::error::Result;
use crate::field::traits::FieldElement;

/// A commitment to a message.
pub trait Commitment: Clone + PartialEq + Eq + std::fmt::Debug {
    /// Serialize commitment to bytes.
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize commitment from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// An opening (proof that a commitment corresponds to a message).
#[derive(Clone, Debug)]
pub struct Opening<F: FieldElement> {
    /// The committed message.
    pub message: Vec<F>,
    /// The blinding factor used.
    pub blinding: F,
}

/// A commitment scheme.
///
/// Provides commit, open, and verify operations.
pub trait CommitmentScheme {
    /// The field element type.
    type Field: FieldElement;
    /// The commitment type.
    type Commitment: Commitment;

    /// Commit to a message vector with blinding factor.
    ///
    /// # Security
    /// Blinding factor must be chosen uniformly at random for hiding.
    fn commit(
        &self,
        message: &[Self::Field],
        blinding: Self::Field,
    ) -> Result<Self::Commitment>;

    /// Commit without blinding (public commitment, not hiding).
    fn commit_unblinded(&self, message: &[Self::Field]) -> Result<Self::Commitment> {
        self.commit(message, Self::Field::zero())
    }

    /// Verify that a commitment opens to a specific message.
    fn verify(
        &self,
        commitment: &Self::Commitment,
        opening: &Opening<Self::Field>,
    ) -> Result<()>;

    /// Maximum message length supported.
    fn max_message_len(&self) -> usize;
}