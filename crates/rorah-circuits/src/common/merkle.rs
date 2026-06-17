use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf: Vec<u8>,
    pub path: Vec<Vec<u8>>,
    pub index: u64,
}

impl MerkleProof {
    pub fn new(leaf: Vec<u8>, path: Vec<Vec<u8>>, index: u64) -> Self {
        MerkleProof { leaf, path, index }
    }

    pub fn verify(&self, root: &[u8]) -> anyhow::Result<bool> {
        let mut current = self.leaf.clone();
        let mut index = self.index;

        for sibling in &self.path {
            current = if index % 2 == 0 {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
            index /= 2;
        }

        Ok(current == root)
    }

    pub fn compute_root(&self) -> Vec<u8> {
        let mut current = self.leaf.clone();
        let mut index = self.index;

        for sibling in &self.path {
            current = if index % 2 == 0 {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
            index /= 2;
        }

        current
    }

    pub fn leaf_index(&self) -> u64 {
        self.index
    }

    pub fn depth(&self) -> usize {
        self.path.len()
    }
}

pub fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().to_vec()
}

pub fn compute_root(leaf: &[u8], path: &[Vec<u8>], index: u64) -> Vec<u8> {
    let mut current = leaf.to_vec();
    let mut idx = index;

    for sibling in path {
        current = if idx % 2 == 0 {
            hash_pair(&current, sibling)
        } else {
            hash_pair(sibling, &current)
        };
        idx /= 2;
    }

    current
}

pub struct MerkleCap {
    pub roots: Vec<Vec<u8>>,
}

impl MerkleCap {
    pub fn new(roots: Vec<Vec<u8>>) -> Self {
        MerkleCap { roots }
    }

    pub fn verify_leaf(&self, leaf: &[u8], path: &[Vec<u8>], index: u64) -> anyhow::Result<bool> {
        let computed_root = compute_root(leaf, path, index);
        let cap_root = &self.roots[0];
        Ok(computed_root == *cap_root)
    }

    pub fn height(&self) -> usize {
        (self.roots.len() as f64).log2().ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_verification() {
        let leaf = b"leaf".to_vec();
        let sibling1 = b"sibling1".to_vec();
        let path = vec![sibling1.clone()];
        let index = 0u64;

        let proof = MerkleProof::new(leaf.clone(), path, index);
        let root = proof.compute_root();

        let proof2 = MerkleProof::new(leaf, vec![sibling1], index);
        assert!(proof2.verify(&root).unwrap());
    }
}