//! Cross-term polynomial computation for Nova folding.
//!
//! T = Az₁∘Bz₂ + Az₂∘Bz₁ - u₁·Cz₂ - u₂·Cz₁

use crate::error::{Result, RorahError};
use crate::field::FieldElement;
use crate::r1cs::{RelaxedR1CSInstance, Witness};

/// Compute the cross-term polynomial T.
///
/// This is the most computationally intensive part of Nova folding.
/// T must be committed to before the Fiat-Shamir challenge is derived.
///
/// Formula:
///   T = (Az₁) ∘ (Bz₂) + (Az₂) ∘ (Bz₁) - u₁·(Cz₂) - u₂·(Cz₁)
///
/// # Security
/// The cross-term binds both instances together such that the challenge
/// cannot be manipulated after committing to T.
pub fn compute_cross_term(
    instance1: &RelaxedR1CSInstance,
    witness1: &Witness,
    instance2: &RelaxedR1CSInstance,
    witness2: &Witness,
) -> Result<Vec<FieldElement>> {
    validate_dimensions(instance1, instance2, witness1, witness2)?;

    if instance1.num_constraints == 0 {
        return Ok(Vec::new());
    }

    let z1 = witness1.variables();
    let z2 = witness2.variables();

    // Compute matrix-vector products
    let az1 = instance1.a_matrix.multiply_vector(z1)?;
    let bz1 = instance1.b_matrix.multiply_vector(z1)?;
    let cz1 = instance1.c_matrix.multiply_vector(z1)?;

    let az2 = instance2.a_matrix.multiply_vector(z2)?;
    let bz2 = instance2.b_matrix.multiply_vector(z2)?;
    let cz2 = instance2.c_matrix.multiply_vector(z2)?;

    // T = Az₁∘Bz₂ + Az₂∘Bz₁ - u₁·Cz₂ - u₂·Cz₁
    let mut t = Vec::with_capacity(instance1.num_constraints);

    for i in 0..instance1.num_constraints {
        let term1 = az1[i] * bz2[i]; // Az₁ ∘ Bz₂
        let term2 = az2[i] * bz1[i]; // Az₂ ∘ Bz₁
        let term3 = instance1.u * cz2[i]; // u₁ · Cz₂
        let term4 = instance2.u * cz1[i]; // u₂ · Cz₁

        t.push(term1 + term2 - term3 - term4);
    }

    Ok(t)
}

/// Validate that two instances can be folded together.
fn validate_dimensions(
    instance1: &RelaxedR1CSInstance,
    instance2: &RelaxedR1CSInstance,
    witness1: &Witness,
    witness2: &Witness,
) -> Result<()> {
    if instance1.num_constraints != instance2.num_constraints {
        return Err(RorahError::DimensionMismatch {
            details: format!(
                "Constraint counts differ: {} vs {}",
                instance1.num_constraints, instance2.num_constraints
            ),
        });
    }

    if instance1.num_variables != instance2.num_variables {
        return Err(RorahError::DimensionMismatch {
            details: format!(
                "Variable counts differ: {} vs {}",
                instance1.num_variables, instance2.num_variables
            ),
        });
    }

    if witness1.len() != instance1.num_variables {
        return Err(RorahError::WitnessSizeMismatch {
            expected: instance1.num_variables,
            actual: witness1.len(),
        });
    }

    if witness2.len() != instance2.num_variables {
        return Err(RorahError::WitnessSizeMismatch {
            expected: instance2.num_variables,
            actual: witness2.len(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r1cs::{
        constraint::{Constraint, LinearCombination},
        R1CSInstance, RelaxedR1CSInstance,
    };
    use crate::FieldElement;

    fn make_square_relaxed(x: u64) -> (RelaxedR1CSInstance, Witness) {
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());
        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());
        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());

        let constraint = Constraint::new(a, b, c);
        let public_inputs = vec![FieldElement::from_u64(x)];
        let instance =
            R1CSInstance::from_constraints(vec![constraint], 3, public_inputs).unwrap();

        let relaxed = RelaxedR1CSInstance::from_r1cs(instance);

        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(x),
                FieldElement::from_u64(x * x),
            ],
            2,
        )
        .unwrap();

        (relaxed, witness)
    }

    #[test]
    fn test_cross_term_same_instances() {
        let (inst1, wit1) = make_square_relaxed(3);
        let (inst2, wit2) = make_square_relaxed(3);

        let t = compute_cross_term(&inst1, &wit1, &inst2, &wit2).unwrap();

        assert_eq!(t.len(), 1);
        // With u1=u2=1 and same instance:
        // T = Az1∘Bz2 + Az2∘Bz1 - Cz2 - Cz1
        // = 3*3 + 3*3 - 9 - 9 = 9 + 9 - 18 = 0
        assert_eq!(t[0], FieldElement::zero());
    }

    #[test]
    fn test_cross_term_different_instances() {
        let (inst1, wit1) = make_square_relaxed(3); // x=3, y=9
        let (inst2, wit2) = make_square_relaxed(5); // x=5, y=25

        let t = compute_cross_term(&inst1, &wit1, &inst2, &wit2).unwrap();

        assert_eq!(t.len(), 1);
        // T = Az1∘Bz2 + Az2∘Bz1 - u1·Cz2 - u2·Cz1
        // Az1 = [3], Bz1 = [3], Cz1 = [9]
        // Az2 = [5], Bz2 = [5], Cz2 = [25]
        // T = 3*5 + 5*3 - 1*25 - 1*9
        // T = 15 + 15 - 25 - 9 = -4
        let expected = FieldElement::from_u64(15)
            + FieldElement::from_u64(15)
            - FieldElement::from_u64(25)
            - FieldElement::from_u64(9);

        assert_eq!(t[0], expected);
    }

    #[test]
    fn test_cross_term_dimension_mismatch() {
        let (inst1, wit1) = make_square_relaxed(3);

        // Instance with different num_variables
        let mut a = LinearCombination::zero();
        a.add_term(1, FieldElement::one());
        let mut b = LinearCombination::zero();
        b.add_term(1, FieldElement::one());
        let mut c = LinearCombination::zero();
        c.add_term(2, FieldElement::one());
        let constraint = Constraint::new(a, b, c);

        let instance_diff = R1CSInstance::from_constraints(
            vec![constraint],
            5, // Different number of variables
            vec![FieldElement::from_u64(3)],
        )
        .unwrap();

        let relaxed_diff = RelaxedR1CSInstance::from_r1cs(instance_diff);
        let witness_diff = Witness::new(
            vec![FieldElement::one(); 5],
            2,
        )
        .unwrap();

        let result = compute_cross_term(&inst1, &wit1, &relaxed_diff, &witness_diff);
        assert!(result.is_err());
    }
}