//! R1CS constraints.

//! R1CS constraints.

use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _; // Import trait for methods
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;


/// A single linear combination: Σ coeff_i · wire_i
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinearCombination {
    /// Map from wire index to coefficient
    terms: BTreeMap<usize, FieldElement>,
}

impl LinearCombination {
    /// Create an empty linear combination.
    pub fn zero() -> Self {
        Self {
            terms: BTreeMap::new(),
        }
    }

    /// Add a term: coeff * wire[index]
    pub fn add_term(&mut self, index: usize, coeff: FieldElement) {
        if !coeff.is_zero() {
            *self.terms.entry(index).or_insert_with(FieldElement::zero) =
                self.terms
                    .get(&index)
                    .map(|&c| c + coeff)
                    .unwrap_or(coeff);
        }
    }

    /// Evaluate the linear combination given witness values.
    pub fn evaluate(&self, witness: &[FieldElement]) -> FieldElement {
        self.terms
            .iter()
            .map(|(&idx, &coeff)| {
                witness
                    .get(idx)
                    .map(|&w| coeff * w)
                    .unwrap_or_else(FieldElement::zero)
            })
            .fold(FieldElement::zero(), |acc, val| acc + val)
    }

    /// Get all terms.
    pub fn terms(&self) -> &BTreeMap<usize, FieldElement> {
        &self.terms
    }

    /// Number of non-zero terms.
    pub fn num_terms(&self) -> usize {
        self.terms.len()
    }
}

/// A single R1CS constraint: A · B = C
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub a: LinearCombination,
    pub b: LinearCombination,
    pub c: LinearCombination,
}

impl Constraint {
    /// Create a new constraint.
    pub fn new(a: LinearCombination, b: LinearCombination, c: LinearCombination) -> Self {
        Self { a, b, c }
    }

    /// Check if constraint is satisfied by witness.
    pub fn is_satisfied(&self, witness: &[FieldElement]) -> bool {
        let a_val = self.a.evaluate(witness);
        let b_val = self.b.evaluate(witness);
        let c_val = self.c.evaluate(witness);

        a_val * b_val == c_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_combination() {
        let mut lc = LinearCombination::zero();
        lc.add_term(0, FieldElement::from_u64(2));
        lc.add_term(1, FieldElement::from_u64(3));

        let witness = vec![FieldElement::from_u64(5), FieldElement::from_u64(7)];
        // 2*5 + 3*7 = 10 + 21 = 31
        assert_eq!(lc.evaluate(&witness), FieldElement::from_u64(31));
    }

    #[test]
    fn test_simple_constraint() {
        // Constraint: (2*w0) * (3*w1) = (6*w2)
        let mut a = LinearCombination::zero();
        a.add_term(0, FieldElement::from_u64(2));

        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::from_u64(3));

        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::from_u64(6));

        let constraint = Constraint::new(a, b, c);

        let witness = vec![
            FieldElement::from_u64(5),
            FieldElement::from_u64(7),
            FieldElement::from_u64(35),
        ];

        assert!(constraint.is_satisfied(&witness));
    }
}