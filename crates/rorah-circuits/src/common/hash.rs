use sha2::{Digest, Sha256};

pub trait HashFunction {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn name(&self) -> &'static str;
}

pub struct Sha256Hash;

impl HashFunction for Sha256Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    fn name(&self) -> &'static str {
        "SHA256"
    }
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn sha256_2(data1: &[u8], data2: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data1);
    hasher.update(data2);
    hasher.finalize().to_vec()
}

pub struct PoseidonHash {
    rate: usize,
}

impl PoseidonHash {
    pub fn new(rate: usize) -> Self {
        PoseidonHash { rate }
    }

    pub fn hash(&self, inputs: &[u64]) -> Vec<u64> {
        let mut state = vec![0u64; 12];
        for (i, &input) in inputs.iter().enumerate().take(self.rate) {
            state[i] ^= input;
        }
        self.permutation(&mut state);
        state[0..4].to_vec()
    }

    fn permutation(&self, state: &mut [u64]) {
        for _ in 0..8 {
            self.apply_sbox(state);
            self.apply_mds(state);
        }
    }

    fn apply_sbox(&self, state: &mut [u64]) {
        for element in state.iter_mut() {
            let x = *element;
            *element = x.wrapping_pow(5);
        }
    }

    fn apply_mds(&self, _state: &mut [u64]) {}
}

pub struct KeccakHash;

impl KeccakHash {
    pub fn hash(data: &[u8]) -> Vec<u8> {
        use sha3::{Digest as Sha3Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"test data";
        let hash1 = sha256(data);
        let hash2 = sha256(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_sha256_2() {
        let data1 = b"part1";
        let data2 = b"part2";
        let hash = sha256_2(data1, data2);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_keccak() {
        let data = b"test";
        let hash = KeccakHash::hash(data);
        assert_eq!(hash.len(), 32);
    }
}