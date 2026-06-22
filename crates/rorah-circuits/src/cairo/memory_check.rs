use ark_bn254::Fr;
use ark_ff::One;

pub struct MemoryChecker;

#[derive(Debug, Clone)]
pub struct MemoryAccessRecord {
    pub address: u64,
    pub value: Vec<u8>,
    pub cycle: u64,
}

impl MemoryChecker {
    pub fn verify_memory_consistency(
        accesses: &[MemoryAccessRecord],
    ) -> bool {
        if accesses.is_empty() {
            return false;
        }

        accesses.windows(2).all(|window| {
            window[0].cycle < window[1].cycle
        })
    }

    pub fn verify_no_contradictions(
        reads: &[MemoryAccessRecord],
        writes: &[MemoryAccessRecord],
    ) -> bool {
        for read in reads {
            for write in writes {
                if read.address == write.address && read.cycle == write.cycle {
                    if read.value != write.value {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn compute_memory_product(
        accesses: &[MemoryAccessRecord],
        beta: Fr,
        gamma: Fr,
    ) -> Fr {
        accesses.iter().fold(Fr::one(), |acc, access| {
            let address_val = Fr::from(access.address);
            let cycle_val = Fr::from(access.cycle);
            acc * (address_val + beta * cycle_val + gamma)
        })
    }

    pub fn verify_read_once_property(
        accesses: &[MemoryAccessRecord],
    ) -> bool {
        let mut seen_addresses = std::collections::HashSet::new();

        for access in accesses {
            if !seen_addresses.insert(access.address) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_consistency() {
        let accesses = vec![
            MemoryAccessRecord {
                address: 0,
                value: vec![1u8],
                cycle: 0,
            },
            MemoryAccessRecord {
                address: 1,
                value: vec![2u8],
                cycle: 1,
            },
        ];

        let result = MemoryChecker::verify_memory_consistency(&accesses);
        assert!(result);
    }

    #[test]
    fn test_read_once_property() {
        let accesses = vec![
            MemoryAccessRecord {
                address: 0,
                value: vec![1u8],
                cycle: 0,
            },
            MemoryAccessRecord {
                address: 1,
                value: vec![2u8],
                cycle: 1,
            },
        ];

        assert!(MemoryChecker::verify_read_once_property(&accesses));
    }

    #[test]
    fn test_read_once_property_fails_on_duplicate() {
        let accesses = vec![
            MemoryAccessRecord {
                address: 0,
                value: vec![1u8],
                cycle: 0,
            },
            MemoryAccessRecord {
                address: 0,
                value: vec![2u8],
                cycle: 1,
            },
        ];

        assert!(!MemoryChecker::verify_read_once_property(&accesses));
    }
}