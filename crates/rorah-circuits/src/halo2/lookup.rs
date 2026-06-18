use ark_bn254::Fr;
use ark_ff::{Field, One};

pub struct LookupVerifier;

#[derive(Debug, Clone)]
pub struct LookupArgument {
    pub input_columns: Vec<Vec<Fr>>,
    pub table_columns: Vec<Vec<Fr>>,
    pub product_commitment: Vec<u8>,
}

impl LookupVerifier {
    pub fn verify_lookup(lookup: &LookupArgument) -> anyhow::Result<bool> {
        if lookup.input_columns.is_empty() {
            anyhow::bail!("input_columns cannot be empty");
        }
        if lookup.table_columns.is_empty() {
            anyhow::bail!("table_columns cannot be empty");
        }

        let input_size = lookup.input_columns[0].len();

        for input in &lookup.input_columns {
            if input.len() != input_size {
                return Ok(false);
            }
        }

        let table_size = lookup.table_columns[0].len();
        for table in &lookup.table_columns {
            if table.len() != table_size {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_inclusion(
        element: Fr,
        input: &[Fr],
        table: &[Fr],
    ) -> bool {
        input.contains(&element) && table.contains(&element)
    }

    pub fn compute_lookup_product(
        input: &[Fr],
        _table: &[Fr],
        beta: Fr,
        gamma: Fr,
    ) -> Fr {
        let mut product = Fr::one();

        for val in input {
            product *= (*val + beta * gamma);
        }

        product
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_verification() {
        let lookup = LookupArgument {
            input_columns: vec![vec![Fr::from(1u64), Fr::from(2u64)]],
            table_columns: vec![vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)]],
            product_commitment: vec![0u8; 32],
        };

        let result = LookupVerifier::verify_lookup(&lookup);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}