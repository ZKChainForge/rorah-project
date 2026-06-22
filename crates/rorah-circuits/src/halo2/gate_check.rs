use ark_ff::Zero;
use ark_bn254::Fr;

pub struct Halo2GateChecker;

#[derive(Debug, Clone)]
pub struct CustomGate {
    pub name: String,
    pub selector_index: usize,
    pub constraints: Vec<Vec<usize>>,
}

impl Halo2GateChecker {
    pub fn verify_gate_constraints(
        gates: &[CustomGate],
        cell_values: &[Fr],
    ) -> bool {
        gates.iter().all(|gate| {
            gate.constraints.iter().all(|constraint| {
                let mut sum = Fr::zero();

                for &cell_idx in constraint {
                    if cell_idx < cell_values.len() {
                        sum += cell_values[cell_idx];
                    }
                }

                sum == Fr::zero()
            })
        })
    }

    pub fn verify_flexibility(gates: &[CustomGate], max_gates: usize) -> bool {
        gates.len() <= max_gates
    }

    pub fn validate_gate_structure(gate: &CustomGate) -> anyhow::Result<()> {
        if gate.name.is_empty() {
            anyhow::bail!("Gate name cannot be empty");
        }
        if gate.constraints.is_empty() {
            anyhow::bail!("Gate must have at least one constraint");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_verification() {
        let gate = CustomGate {
            name: "test_gate".to_string(),
            selector_index: 0,
            constraints: vec![vec![0, 1]],
        };

        let result = Halo2GateChecker::validate_gate_structure(&gate);
        assert!(result.is_ok());
    }
}