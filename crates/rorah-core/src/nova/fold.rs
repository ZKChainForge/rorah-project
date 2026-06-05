//! Core Nova folding logic.

use crate::commitment::params::CommitmentParams;
use crate::commitment::pedersen::PedersenCommitment;
use crate::commitment::traits::Commitment; // this line
use crate::error::{Result, RorahError};
use crate::field::bn254::BN254FieldElement as FieldElement;
use crate::field::traits::FieldElement as _;
use crate::nova::{NovaAccumulator, NovaProof};
use crate::r1cs::{R1CSInstance, RelaxedR1CSInstance, Witness};
use crate::transcript::Transcript;



/// Fold a new R1CS instance into an existing accumulator.
pub fn fold_instances(
    accumulator: NovaAccumulator,
    instance: R1CSInstance,
    witness: Witness,
    params: &CommitmentParams,
) -> Result<(NovaAccumulator, NovaProof)> {
    instance.is_satisfied(&witness)?;
    accumulator.is_valid()?;

    let new_relaxed = RelaxedR1CSInstance::from_r1cs(instance);

    if accumulator.is_empty() {
        let dummy_point = params.commit_unblinded(&[])?;
        let proof = NovaProof::new(PedersenCommitment(dummy_point));
        let new_acc = NovaAccumulator::new(new_relaxed, witness)?;
        return Ok((new_acc, proof));
    }

    let cross_term = compute_cross_term(
        &accumulator.instance,
        &accumulator.witness,
        &new_relaxed,
        &witness,
    )?;

    let t_point = params.commit_unblinded(&cross_term)?;
    let t_commitment = PedersenCommitment(t_point);

    let r = derive_challenge(&accumulator, &new_relaxed, &t_commitment)?;

    let folded_instance = fold_relaxed(
        &accumulator.instance,
        &new_relaxed,
        &cross_term,
        r,
    )?;

    let folded_witness = fold_witnesses(&accumulator.witness, &witness, r)?;

    let new_accumulator = NovaAccumulator::new(folded_instance, folded_witness)?;

    Ok((new_accumulator, NovaProof::new(t_commitment)))
}

fn compute_cross_term(
    inst1: &RelaxedR1CSInstance,
    wit1: &Witness,
    inst2: &RelaxedR1CSInstance,
    wit2: &Witness,
) -> Result<Vec<FieldElement>> {
    if inst1.num_constraints != inst2.num_constraints {
        return Err(RorahError::DimensionMismatch {
            details: format!(
                "constraint counts differ: {} vs {}",
                inst1.num_constraints, inst2.num_constraints
            ),
        });
    }

    let z1 = wit1.variables();
    let z2 = wit2.variables();

    let az1 = inst1.a_matrix.multiply_vector(z1)?;
    let bz1 = inst1.b_matrix.multiply_vector(z1)?;
    let cz1 = inst1.c_matrix.multiply_vector(z1)?;

    let az2 = inst2.a_matrix.multiply_vector(z2)?;
    let bz2 = inst2.b_matrix.multiply_vector(z2)?;
    let cz2 = inst2.c_matrix.multiply_vector(z2)?;

    let mut t = Vec::with_capacity(inst1.num_constraints);

    for i in 0..inst1.num_constraints {
        let val = az1[i] * bz2[i]
            + az2[i] * bz1[i]
            - inst1.u * cz2[i]
            - inst2.u * cz1[i];
        t.push(val);
    }

    Ok(t)
}

fn derive_challenge(
    acc: &NovaAccumulator,
    new_inst: &RelaxedR1CSInstance,
    t_commitment: &PedersenCommitment,
) -> Result<FieldElement> {
    let mut transcript = Transcript::new(b"RORAH_NOVA_FOLD_V1");

    transcript.absorb_field_slice(b"acc_pub", acc.public_inputs());
    transcript.absorb_field(b"acc_u", &acc.instance.u);
    transcript.absorb_field_slice(b"new_pub", &new_inst.public_inputs);
    transcript.absorb_field(b"new_u", &new_inst.u);
    transcript.absorb(b"T_commit", &t_commitment.to_bytes());

    transcript.squeeze(b"r")
}

fn fold_relaxed(
    inst1: &RelaxedR1CSInstance,
    inst2: &RelaxedR1CSInstance,
    cross_term: &[FieldElement],
    r: FieldElement,
) -> Result<RelaxedR1CSInstance> {
    let r_sq = r.square();

    let folded_u = inst1.u + r * inst2.u;

    let mut folded_e = Vec::with_capacity(inst1.num_constraints);
    for i in 0..inst1.num_constraints {
        let e_i = inst1.error_vector[i]
            + r * cross_term[i]
            + r_sq * inst2.error_vector[i];
        folded_e.push(e_i);
    }

    let max_pub = inst1.public_inputs.len().max(inst2.public_inputs.len());
    let mut folded_pub = Vec::with_capacity(max_pub);
    for i in 0..max_pub {
        let x1 = inst1
            .public_inputs
            .get(i)
            .copied()
            .unwrap_or_else(FieldElement::zero);
        let x2 = inst2
            .public_inputs
            .get(i)
            .copied()
            .unwrap_or_else(FieldElement::zero);
        folded_pub.push(x1 + r * x2);
    }

    Ok(RelaxedR1CSInstance {
        num_constraints: inst1.num_constraints,
        num_variables: inst1.num_variables,
        num_public_inputs: folded_pub.len(),
        a_matrix: inst1.a_matrix.clone(),
        b_matrix: inst1.b_matrix.clone(),
        c_matrix: inst1.c_matrix.clone(),
        public_inputs: folded_pub,
        u: folded_u,
        error_vector: folded_e,
    })
}

fn fold_witnesses(
    w1: &Witness,
    w2: &Witness,
    r: FieldElement,
) -> Result<Witness> {
    if w1.len() != w2.len() {
        return Err(RorahError::WitnessSizeMismatch {
            expected: w1.len(),
            actual: w2.len(),
        });
    }

    let z1 = w1.variables();
    let z2 = w2.variables();

    let folded: Vec<FieldElement> = z1
        .iter()
        .zip(z2.iter())
        .map(|(&a, &b)| a + r * b)
        .collect();

    let pub_len = {
        let p1 = w1.public_inputs().len();
        let p2 = w2.public_inputs().len();
        p1.max(p2) + 1
    };

    Witness::new(folded, pub_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r1cs::constraint::{Constraint, LinearCombination};

    fn square_instance_witness(x: u64) -> (R1CSInstance, Witness) {
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

        let witness = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(x),
                FieldElement::from_u64(x * x),
            ],
            2,
        )
        .unwrap();

        (instance, witness)
    }

    #[test]
    fn test_fold_first_into_empty() {
        let params = CommitmentParams::new(16);
        let acc = NovaAccumulator::empty(3).unwrap();

        let (inst, wit) = square_instance_witness(4);

        let (new_acc, _proof) = fold_instances(acc, inst, wit, &params).unwrap();

        assert!(!new_acc.is_empty());
        assert!(new_acc.is_valid().is_ok());
        assert_eq!(new_acc.num_constraints(), 1);
    }

    #[test]
    fn test_fold_two_sequential() {
        let params = CommitmentParams::new(16);
        let acc = NovaAccumulator::empty(3).unwrap();

        let (i1, w1) = square_instance_witness(3);
        let (acc1, _) = fold_instances(acc, i1, w1, &params).unwrap();
        assert!(acc1.is_valid().is_ok());

        let (i2, w2) = square_instance_witness(7);
        let (acc2, _) = fold_instances(acc1, i2, w2, &params).unwrap();
        assert!(acc2.is_valid().is_ok());
    }

    #[test]
    fn test_invalid_witness_rejected() {
        let params = CommitmentParams::new(16);
        let acc = NovaAccumulator::empty(3).unwrap();

        let (inst, _) = square_instance_witness(5);

        let bad = Witness::new(
            vec![
                FieldElement::one(),
                FieldElement::from_u64(5),
                FieldElement::from_u64(26),
            ],
            2,
        )
        .unwrap();

        assert!(fold_instances(acc, inst, bad, &params).is_err());
    }
}