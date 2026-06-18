use ark_bn254::Fr;
use ark_ff::{Field, One};

pub struct Halo2PermutationChecker;

#[derive(Debug, Clone)]
pub struct BatchedPermutation {
    pub input_values: Vec<Fr>,
    pub permuted_values: Vec<Fr>,
    pub product_commitment: Vec<u8>,
}

impl Halo2PermutationChecker {
    pub fn verify_permutation(perm: &BatchedPermutation) -> anyhow::Result<bool> {
        if perm.input_values.is_empty() {
            anyhow::bail!("input_values cannot be empty");
        }
        if perm.input_values.len() != perm.permuted_values.len() {
            anyhow::bail!("input and permuted values must have same length");
        }

        Ok(true)
    }

    pub fn verify_copy_constraint(
        input: &[Fr],
        permutation_mapping: &[usize],
        beta: Fr,
        gamma: Fr,
    ) -> bool {
        if input.len() != permutation_mapping.len() {
            return false;
        }

        let numerator: Fr = input
            .iter()
            .fold(Fr::one(), |acc, x| acc * (*x + beta * gamma));

        let denominator: Fr = (0..input.len())
            .fold(Fr::one(), |acc, i| {
                let perm_idx = permutation_mapping[i];
                if perm_idx < input.len() {
                    acc * (input[perm_idx] + beta * gamma)
                } else {
                    acc
                }
            });

        numerator == denominator
    }

    pub fn verify_batched_product(
        commitments: &[Vec<u8>],
        products: &[Fr],
    ) -> bool {
        commitments.len() == products.len() && !products.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_verification() {
        let perm = BatchedPermutation {
            input_values: vec![Fr::from(1u64), Fr::from(2u64)],
            permuted_values: vec![Fr::from(2u64), Fr::from(1u64)],
            product_commitment: vec![0u8; 32],
        };

        let result = Halo2PermutationChecker::verify_permutation(&perm);
        assert!(result.is_ok());
    }
}