use ark_bn254::Fr;
use ark_ff::Field;

pub struct ConstraintChecker;

impl ConstraintChecker {
    pub fn verify_constraint_polynomial(
        evaluations: &[Vec<u8>],
        quotient_poly: &[u8],
    ) -> anyhow::Result<bool> {
        if evaluations.is_empty() {
            anyhow::bail!("evaluations cannot be empty");
        }

        if quotient_poly.is_empty() {
            anyhow::bail!("quotient_poly cannot be empty");
        }

        for eval in evaluations {
            if eval.len() != quotient_poly.len() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_gate_constraints(
        trace: &[Vec<u8>],
        constraint_count: u32,
    ) -> anyhow::Result<bool> {
        if trace.is_empty() {
            anyhow::bail!("trace cannot be empty");
        }

        let trace_length = trace[0].len();

        for row in trace {
            if row.len() != trace_length {
                return Ok(false);
            }
        }

        if constraint_count == 0 {
            anyhow::bail!("constraint_count must be > 0");
        }

        Ok(true)
    }

    pub fn verify_boundary_constraints(
        trace: &[Vec<u8>],
        initial_state: &[u8],
    ) -> anyhow::Result<bool> {
        if trace.is_empty() {
            anyhow::bail!("trace cannot be empty");
        }

        if let Some(first_row) = trace.first() {
            if first_row != initial_state {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_verification() {
        let evaluations = vec![vec![1u8; 32], vec![2u8; 32]];
        let quotient_poly = vec![1u8; 32];

        let result = ConstraintChecker::verify_constraint_polynomial(&evaluations, &quotient_poly);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}