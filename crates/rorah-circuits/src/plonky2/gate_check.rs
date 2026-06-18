use ark_ff::{Field, Zero};
use ark_bn254::Fr;

pub struct GateChecker;

impl GateChecker {
    pub fn verify_arithmetic_gate(
        left_input: Fr,
        right_input: Fr,
        output: Fr,
        alpha: Fr,
        beta: Fr,
    ) -> bool {
        let computed = left_input * right_input + alpha * left_input + beta * right_input;
        computed == output
    }

    pub fn verify_mul_gate(left: Fr, right: Fr, output: Fr) -> bool {
        left * right == output
    }

    pub fn verify_add_gate(left: Fr, right: Fr, output: Fr) -> bool {
        left + right == output
    }

    pub fn verify_custom_gate(
        wires: &[Fr],
        gate_constraints: &[GateConstraint],
    ) -> anyhow::Result<bool> {
        for constraint in gate_constraints {
            if !constraint.check(wires) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct GateConstraint {
    pub coefficients: Vec<Fr>,
    pub wire_indices: Vec<usize>,
}

impl GateConstraint {
    pub fn check(&self, wires: &[Fr]) -> bool {
        let mut sum = Fr::zero();

        for (coeff, &idx) in self.coefficients.iter().zip(self.wire_indices.iter()) {
            if idx < wires.len() {
                sum += *coeff * wires[idx];
            }
        }

        sum == Fr::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_gate() {
        let left = Fr::from(5u64);
        let right = Fr::from(3u64);
        let alpha = Fr::from(5u64);
        let beta = Fr::from(0u64);
        let output = left * right + alpha * left + beta * right;

        let result = GateChecker::verify_arithmetic_gate(left, right, output, alpha, beta);
        assert!(result);
    }
}