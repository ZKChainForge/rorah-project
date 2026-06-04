//! Fiat-Shamir transcript implementation.
//!
//! Converts interactive proofs to non-interactive by deriving
//! all verifier challenges from a running hash of the transcript.
//!
//! # Security Properties
//! - Unforgeability: adversary cannot predict future challenges
//! - Binding: challenges depend on all prior messages
//! - Domain separation: different protocols give different challenges
//! - Length-extension resistance: SHA3 (sponge) construction used
//!
//! # Implementation
//! Uses SHA3-256 (Keccak-based) which:
//! - Has no length-extension vulnerabilities (unlike SHA2)
//! - Is standardized and well-audited
//! - Is cheap to compute off-circuit
//!
//! For on-circuit challenges, use PoseidonHash instead.

use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement;
use crate::field::traits::FieldElement;
use sha3::{Digest, Sha3_256};

/// Transcript for the Fiat-Shamir transform.
///
/// Maintains a running hash state that is updated with each
/// absorbed message. Challenges are derived by squeezing the state.
pub struct Transcript {
    hasher: Sha3_256,
    /// Protocol identifier (domain separator).
    protocol_id: Vec<u8>,
    /// Number of messages absorbed so far.
    num_absorbed: u64,
    /// Number of challenges squeezed so far.
    num_squeezed: u64,
}

impl Transcript {
    /// Create a new transcript for a specific protocol.
    ///
    /// # Security
    /// The `protocol_id` is the domain separator. Different protocols
    /// must use different identifiers to prevent cross-protocol attacks.
    ///
    /// # Example
    /// ```
    /// let t = Transcript::new(b"RORAH_NOVA_FOLD_V1");
    /// ```
    pub fn new(protocol_id: &[u8]) -> Self {
        let mut hasher = Sha3_256::new();

        // Initialize with versioned domain separator
        hasher.update(b"RORAH_FS_TRANSCRIPT_V1:");
        hasher.update(&(protocol_id.len() as u64).to_le_bytes());
        hasher.update(protocol_id);

        Self {
            hasher,
            protocol_id: protocol_id.to_vec(),
            num_absorbed: 0,
            num_squeezed: 0,
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Absorb methods
    // ─────────────────────────────────────────────────────────────────

    /// Absorb raw bytes with a label.
    ///
    /// # Security
    /// The label is included in the hash to prevent:
    /// - Different message types colliding
    /// - Reordering attacks
    pub fn absorb(&mut self, label: &[u8], data: &[u8]) {
        // Include label
        self.hasher.update(b"absorb:");
        self.hasher.update(&(label.len() as u64).to_le_bytes());
        self.hasher.update(label);

        // Include data with length prefix (prevents length-extension)
        self.hasher.update(&(data.len() as u64).to_le_bytes());
        self.hasher.update(data);

        // Include message counter (prevents reordering)
        self.hasher.update(&self.num_absorbed.to_le_bytes());
        self.num_absorbed += 1;
    }

    /// Absorb a single BN254 field element.
    pub fn absorb_field(&mut self, label: &[u8], element: &BN254FieldElement) {
        let bytes = element.to_bytes();
        self.absorb(label, &bytes);
    }

    /// Absorb a slice of BN254 field elements.
    pub fn absorb_field_slice(&mut self, label: &[u8], elements: &[BN254FieldElement]) {
        // Absorb length first to prevent ambiguity
        let len_label = [label, b"_len"].concat();
        self.absorb(&len_label, &(elements.len() as u64).to_le_bytes());

        // Absorb each element with index
        for (i, elem) in elements.iter().enumerate() {
            let elem_label = format!("{}__{}", String::from_utf8_lossy(label), i);
            self.absorb_field(elem_label.as_bytes(), elem);
        }
    }

    /// Compatibility alias for absorb_field_slice.
    pub fn absorb_field_vec(&mut self, label: &[u8], elements: &[BN254FieldElement]) {
        self.absorb_field_slice(label, elements);
    }

    /// Absorb a u64 value.
    pub fn absorb_u64(&mut self, label: &[u8], value: u64) {
        self.absorb(label, &value.to_le_bytes());
    }

    /// Absorb a boolean value.
    pub fn absorb_bool(&mut self, label: &[u8], value: bool) {
        self.absorb(label, &[value as u8]);
    }

    // ─────────────────────────────────────────────────────────────────
    // Squeeze methods
    // ─────────────────────────────────────────────────────────────────

    /// Squeeze a BN254 field element challenge.
    ///
    /// The challenge depends on all previously absorbed messages.
    ///
    /// # Security
    /// Uses rejection sampling to ensure uniform distribution over F_p.
    pub fn squeeze(&mut self, label: &[u8]) -> Result<BN254FieldElement> {
        // Try up to 100 times (probability of failure per attempt is negligible)
        for attempt in 0u64..100 {
            let candidate = self.squeeze_candidate(label, attempt)?;

            // Rejection sampling: accept if candidate is valid field element
            if let Some(elem) = BN254FieldElement::from_bytes(&candidate) {
                self.num_squeezed += 1;
                return Ok(elem);
            }
        }

        Err(RorahError::TranscriptError(
            "Failed to generate valid field element after 100 attempts".to_string(),
        ))
    }

    /// Squeeze multiple challenges at once.
    ///
    /// Each challenge is distinct and depends on all prior challenges.
    pub fn squeeze_challenges(
        &mut self,
        label: &[u8],
        count: usize,
    ) -> Result<Vec<BN254FieldElement>> {
        let mut challenges = Vec::with_capacity(count);

        for i in 0..count {
            let indexed_label = format!("{}__{}", String::from_utf8_lossy(label), i);
            let challenge = self.squeeze(indexed_label.as_bytes())?;
            challenges.push(challenge);

            // Feed each challenge back into transcript (chaining)
            self.absorb_field(b"squeeze_feedback", &challenge);
        }

        Ok(challenges)
    }

    // ─────────────────────────────────────────────────────────────────
    // State management
    // ─────────────────────────────────────────────────────────────────

    /// Reset transcript to its initial state.
    ///
    /// Useful for testing or restarting a protocol.
    pub fn reset(&mut self) {
        let mut hasher = Sha3_256::new();

        hasher.update(b"RORAH_FS_TRANSCRIPT_V1:");
        hasher.update(&(self.protocol_id.len() as u64).to_le_bytes());
        hasher.update(&self.protocol_id);

        self.hasher = hasher;
        self.num_absorbed = 0;
        self.num_squeezed = 0;
    }

    /// Number of messages absorbed.
    pub fn num_absorbed(&self) -> u64 {
        self.num_absorbed
    }

    /// Number of challenges squeezed.
    pub fn num_squeezed(&self) -> u64 {
        self.num_squeezed
    }

    // ─────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────

    /// Produce a 32-byte candidate for a field element.
    fn squeeze_candidate(
        &self,
        label: &[u8],
        attempt: u64,
    ) -> Result<[u8; 32]> {
        // Fork the hasher (don't consume the state)
        let mut hasher = self.hasher.clone();

        // Add squeeze metadata
        hasher.update(b"squeeze:");
        hasher.update(&(label.len() as u64).to_le_bytes());
        hasher.update(label);
        hasher.update(&self.num_squeezed.to_le_bytes());
        hasher.update(&attempt.to_le_bytes());

        let hash = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash[..32]);
        Ok(bytes)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn new_t() -> Transcript {
        Transcript::new(b"test_protocol_v1")
    }

    // ─────────────────────────────────────────────────────────────────
    // Determinism
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_deterministic_challenges() {
        let value = BN254FieldElement::from_u64(42);

        let mut t1 = new_t();
        t1.absorb_field(b"x", &value);
        let c1 = t1.squeeze(b"challenge").unwrap();

        let mut t2 = new_t();
        t2.absorb_field(b"x", &value);
        let c2 = t2.squeeze(b"challenge").unwrap();

        assert_eq!(c1, c2);
    }

    // ─────────────────────────────────────────────────────────────────
    // Sensitivity
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_different_values_different_challenges() {
        let mut t1 = new_t();
        t1.absorb_field(b"x", &BN254FieldElement::from_u64(1));
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = new_t();
        t2.absorb_field(b"x", &BN254FieldElement::from_u64(2));
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_ne!(c1, c2);
    }

    #[test]
    fn test_different_labels_different_challenges() {
        let val = BN254FieldElement::from_u64(10);

        let mut t = new_t();
        t.absorb_field(b"input", &val);

        let c1 = t.squeeze(b"label_a").unwrap();
        let c2 = t.squeeze(b"label_b").unwrap();

        assert_ne!(c1, c2);
    }

    #[test]
    fn test_absorb_order_matters() {
        let a = BN254FieldElement::from_u64(5);
        let b = BN254FieldElement::from_u64(10);

        let mut t1 = new_t();
        t1.absorb_field(b"first", &a);
        t1.absorb_field(b"second", &b);
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = new_t();
        t2.absorb_field(b"first", &b);
        t2.absorb_field(b"second", &a);
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_ne!(c1, c2);
    }

    // ─────────────────────────────────────────────────────────────────
    // Domain separation
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_domain_separation() {
        let val = BN254FieldElement::from_u64(99);

        let mut t1 = Transcript::new(b"protocol_a");
        t1.absorb_field(b"x", &val);
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = Transcript::new(b"protocol_b");
        t2.absorb_field(b"x", &val);
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_ne!(c1, c2);
    }

    // ─────────────────────────────────────────────────────────────────
    // Multiple challenges
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_multiple_challenges_distinct() {
        let mut t = new_t();
        t.absorb_field(b"seed", &BN254FieldElement::from_u64(777));

        let challenges = t.squeeze_challenges(b"ch", 8).unwrap();
        assert_eq!(challenges.len(), 8);

        for i in 0..8 {
            for j in (i + 1)..8 {
                assert_ne!(
                    challenges[i],
                    challenges[j],
                    "Challenges {} and {} should differ",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_multiple_challenges_deterministic() {
        let seed = BN254FieldElement::from_u64(12345);

        let mut t1 = new_t();
        t1.absorb_field(b"seed", &seed);
        let cs1 = t1.squeeze_challenges(b"ch", 5).unwrap();

        let mut t2 = new_t();
        t2.absorb_field(b"seed", &seed);
        let cs2 = t2.squeeze_challenges(b"ch", 5).unwrap();

        assert_eq!(cs1, cs2);
    }

    // ─────────────────────────────────────────────────────────────────
    // Slice absorption
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_absorb_field_slice() {
        let vec = vec![
            BN254FieldElement::from_u64(1),
            BN254FieldElement::from_u64(2),
            BN254FieldElement::from_u64(3),
        ];

        let mut t1 = new_t();
        t1.absorb_field_slice(b"v", &vec);
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = new_t();
        t2.absorb_field_slice(b"v", &vec);
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_eq!(c1, c2);
    }

    #[test]
    fn test_slice_element_change_changes_challenge() {
        let v1 = vec![
            BN254FieldElement::from_u64(1),
            BN254FieldElement::from_u64(2),
        ];
        let v2 = vec![
            BN254FieldElement::from_u64(1),
            BN254FieldElement::from_u64(99),
        ];

        let mut t1 = new_t();
        t1.absorb_field_slice(b"v", &v1);
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = new_t();
        t2.absorb_field_slice(b"v", &v2);
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_ne!(c1, c2);
    }

    #[test]
    fn test_slice_length_matters() {
        let short = vec![BN254FieldElement::from_u64(1)];
        let long = vec![
            BN254FieldElement::from_u64(1),
            BN254FieldElement::from_u64(0),
        ];

        let mut t1 = new_t();
        t1.absorb_field_slice(b"v", &short);
        let c1 = t1.squeeze(b"ch").unwrap();

        let mut t2 = new_t();
        t2.absorb_field_slice(b"v", &long);
        let c2 = t2.squeeze(b"ch").unwrap();

        assert_ne!(c1, c2);
    }

    // ─────────────────────────────────────────────────────────────────
    // Reset
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_reset() {
        let val = BN254FieldElement::from_u64(42);

        let mut fresh = new_t();
        fresh.absorb_field(b"x", &val);
        let c_fresh = fresh.squeeze(b"ch").unwrap();

        let mut with_reset = new_t();
        with_reset.absorb_field(b"noise", &BN254FieldElement::from_u64(999));
        with_reset.reset();
        with_reset.absorb_field(b"x", &val);
        let c_reset = with_reset.squeeze(b"ch").unwrap();

        assert_eq!(c_fresh, c_reset);
    }

    // ─────────────────────────────────────────────────────────────────
    // Counters
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_counters() {
        let mut t = new_t();
        assert_eq!(t.num_absorbed(), 0);
        assert_eq!(t.num_squeezed(), 0);

        t.absorb_field(b"a", &BN254FieldElement::from_u64(1));
        assert_eq!(t.num_absorbed(), 1);

        t.absorb_field(b"b", &BN254FieldElement::from_u64(2));
        assert_eq!(t.num_absorbed(), 2);

        t.squeeze(b"ch").unwrap();
        assert_eq!(t.num_squeezed(), 1);

        t.squeeze(b"ch2").unwrap();
        assert_eq!(t.num_squeezed(), 2);
    }
}