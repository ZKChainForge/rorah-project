use ark_bn254::Fr;
use ark_ff::{Zero, One};

pub struct PermutationChecker;

#[derive(Debug, Clone)]
pub struct PermutationArgument {
    pub z: Vec<Fr>,
    pub z_inv: Vec<Fr>,
    pub beta: Fr,
    pub gamma: Fr,
}

impl PermutationChecker {
    pub fn verify_permutation(
        perm_arg: &PermutationArgument,
        left_wire: &[Fr],
        right_wire: &[Fr],
    ) -> bool {
        if left_wire.len() != right_wire.len() {
            return false;
        }

        if perm_arg.z.is_empty() {
            return false;
        }

        perm_arg.z.iter().all(|z| *z != Fr::zero())
    }

    pub fn compute_grand_product(
        beta: Fr,
        gamma: Fr,
        values: &[Fr],
    ) -> Fr {
        values.iter().fold(Fr::one(), |acc, v| {
            acc * (*v + beta * gamma)
        })
    }

    pub fn verify_copy_constraint(
        wire_values: &[Fr],
        permutation: &[usize],
        beta: Fr,
        gamma: Fr,
    ) -> bool {
        if wire_values.len() != permutation.len() {
            return false;
        }

        let numerator = wire_values
            .iter()
            .fold(Fr::one(), |acc, w| acc * (*w + beta * gamma));

        let denominator = (0..wire_values.len())
            .fold(Fr::one(), |acc, i| {
                let perm_idx = permutation[i];
                if perm_idx < wire_values.len() {
                    acc * (wire_values[perm_idx] + beta * gamma)
                } else {
                    acc
                }
            });

        numerator == denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_verification() {
        let perm_arg = PermutationArgument {
            z: vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)],
            z_inv: vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)],
            beta: Fr::from(5u64),
            gamma: Fr::from(7u64),
        };

        let left_wire = vec![Fr::from(1u64), Fr::from(2u64)];
        let right_wire = vec![Fr::from(3u64), Fr::from(4u64)];

        let result = PermutationChecker::verify_permutation(&perm_arg, &left_wire, &right_wire);
        assert!(result);
    }
}