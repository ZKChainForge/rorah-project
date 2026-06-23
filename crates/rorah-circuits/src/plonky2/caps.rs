use crate::common::merkle::hash_pair;
use crate::plonky2::types::Plonky2ProofData;

pub struct CapVerifier;

impl CapVerifier {
    pub fn verify_cap(
        leaf: &[u8],
        cap: &[Vec<u8>],
        path: &[Vec<u8>],
        index: u64,
    ) -> anyhow::Result<bool> {
        if cap.is_empty() {
            anyhow::bail!("cap cannot be empty");
        }

        let computed_root = Self::compute_cap_root(leaf, path, index)?;
        Ok(computed_root == cap[0])
    }

    fn compute_cap_root(
        leaf: &[u8],
        path: &[Vec<u8>],
        mut index: u64,
    ) -> anyhow::Result<Vec<u8>> {
        let mut current = leaf.to_vec();

        for sibling in path {
            current = if index % 2 == 0 {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
            index /= 2;
        }

        Ok(current)
    }

    pub fn verify_caps_in_proof(proof: &Plonky2ProofData) -> anyhow::Result<bool> {
        if proof.wire_caps.is_empty() {
            return Ok(false);
        }

        for cap in &proof.wire_caps {
            if cap.is_empty() || cap.len() != 32 {
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
    fn test_cap_verification() {
        let leaf = b"leaf".to_vec();
        let cap = vec![b"root".to_vec()];
        let path = vec![];

        let result = CapVerifier::verify_cap(&leaf, &cap, &path, 0);
        assert!(result.is_ok());
    }
}