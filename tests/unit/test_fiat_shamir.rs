//! Unit tests for Fiat-Shamir transcript.

use rorah_core::transcript::Transcript;
use rorah_core::FieldElement;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn new_transcript() -> Transcript {
    Transcript::new(b"rorah_test_protocol")
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic challenge generation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_squeeze_produces_field_element() {
    let mut t = new_transcript();
    t.absorb_field(b"input", &FieldElement::from_u64(1));

    let challenge = t.squeeze(b"challenge");
    assert!(challenge.is_ok());
}

#[test]
fn test_challenge_is_deterministic() {
    let field_value = FieldElement::from_u64(777);

    let mut t1 = new_transcript();
    t1.absorb_field(b"x", &field_value);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field(b"x", &field_value);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_eq!(c1, c2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain separation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_different_protocols_different_challenges() {
    let field_value = FieldElement::from_u64(100);

    let mut t1 = Transcript::new(b"protocol_one");
    t1.absorb_field(b"x", &field_value);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = Transcript::new(b"protocol_two");
    t2.absorb_field(b"x", &field_value);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2, "Different protocols should produce different challenges");
}

#[test]
fn test_different_labels_different_challenges() {
    let field_value = FieldElement::from_u64(50);

    let mut t = new_transcript();
    t.absorb_field(b"x", &field_value);

    let c1 = t.squeeze(b"label_a").unwrap();
    let c2 = t.squeeze(b"label_b").unwrap();

    assert_ne!(c1, c2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Input sensitivity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_different_inputs_different_challenges() {
    let mut t1 = new_transcript();
    t1.absorb_field(b"x", &FieldElement::from_u64(10));
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field(b"x", &FieldElement::from_u64(11));
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2);
}

#[test]
fn test_input_order_matters() {
    let a = FieldElement::from_u64(1);
    let b = FieldElement::from_u64(2);

    let mut t1 = new_transcript();
    t1.absorb_field(b"a", &a);
    t1.absorb_field(b"b", &b);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field(b"a", &b); // Reversed
    t2.absorb_field(b"b", &a);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2, "Input order should affect the challenge");
}

#[test]
fn test_label_matters_for_absorb() {
    let value = FieldElement::from_u64(42);

    let mut t1 = new_transcript();
    t1.absorb_field(b"label_x", &value);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field(b"label_y", &value);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2, "Labels should affect the challenge");
}

// ─────────────────────────────────────────────────────────────────────────────
// Vector absorption
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_absorb_field_vec() {
    let vec = vec![
        FieldElement::from_u64(1),
        FieldElement::from_u64(2),
        FieldElement::from_u64(3),
    ];

    let mut t1 = new_transcript();
    t1.absorb_field_vec(b"vec", &vec);
    let c1 = t1.squeeze(b"challenge").unwrap();

    // Same vector, same challenge
    let mut t2 = new_transcript();
    t2.absorb_field_vec(b"vec", &vec);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_eq!(c1, c2);
}

#[test]
fn test_vec_element_change_changes_challenge() {
    let vec1 = vec![
        FieldElement::from_u64(1),
        FieldElement::from_u64(2),
        FieldElement::from_u64(3),
    ];
    let vec2 = vec![
        FieldElement::from_u64(1),
        FieldElement::from_u64(2),
        FieldElement::from_u64(99), // Changed last element
    ];

    let mut t1 = new_transcript();
    t1.absorb_field_vec(b"vec", &vec1);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field_vec(b"vec", &vec2);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2);
}

#[test]
fn test_vec_length_matters() {
    let short = vec![FieldElement::from_u64(1), FieldElement::from_u64(2)];
    let long = vec![
        FieldElement::from_u64(1),
        FieldElement::from_u64(2),
        FieldElement::from_u64(0), // Extra zero element
    ];

    let mut t1 = new_transcript();
    t1.absorb_field_vec(b"vec", &short);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field_vec(b"vec", &long);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_ne!(c1, c2, "Vector length should affect challenge");
}

// ─────────────────────────────────────────────────────────────────────────────
// Multiple challenges
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_multiple_challenges_all_distinct() {
    let mut t = new_transcript();
    t.absorb_field(b"seed", &FieldElement::from_u64(42));

    let challenges = t.squeeze_challenges(b"multi", 10).unwrap();

    assert_eq!(challenges.len(), 10);

    for i in 0..10 {
        for j in (i + 1)..10 {
            assert_ne!(
                challenges[i],
                challenges[j],
                "Challenges {} and {} should be distinct",
                i,
                j
            );
        }
    }
}

#[test]
fn test_multiple_challenges_deterministic() {
    let seed = FieldElement::from_u64(12345);

    let mut t1 = new_transcript();
    t1.absorb_field(b"seed", &seed);
    let challenges1 = t1.squeeze_challenges(b"multi", 5).unwrap();

    let mut t2 = new_transcript();
    t2.absorb_field(b"seed", &seed);
    let challenges2 = t2.squeeze_challenges(b"multi", 5).unwrap();

    assert_eq!(challenges1, challenges2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw byte absorption
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_absorb_raw_bytes() {
    let data = b"raw bytes input";

    let mut t1 = new_transcript();
    t1.absorb(b"data", data);
    let c1 = t1.squeeze(b"challenge").unwrap();

    let mut t2 = new_transcript();
    t2.absorb(b"data", data);
    let c2 = t2.squeeze(b"challenge").unwrap();

    assert_eq!(c1, c2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Reset
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_reset_restores_initial_state() {
    let field_value = FieldElement::from_u64(55);

    // Fresh transcript
    let mut t_fresh = new_transcript();
    t_fresh.absorb_field(b"x", &field_value);
    let c_fresh = t_fresh.squeeze(b"challenge").unwrap();

    // Transcript with extra data, then reset
    let mut t_reset = new_transcript();
    t_reset.absorb_field(b"extra", &FieldElement::from_u64(9999));
    t_reset.reset();
    t_reset.absorb_field(b"x", &field_value);
    let c_reset = t_reset.squeeze(b"challenge").unwrap();

    assert_eq!(c_fresh, c_reset, "After reset, transcript should behave like fresh");
}