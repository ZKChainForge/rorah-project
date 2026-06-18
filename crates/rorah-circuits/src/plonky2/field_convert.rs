use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};

pub struct FieldConverter;

const GOLDILOCKS_MOD: u64 = 0xFFFF_FFFF_0000_0001u64;

impl FieldConverter {
    pub fn goldilocks_to_bn254(value: u64) -> Fr {
        let reduced = if value >= GOLDILOCKS_MOD {
            value - GOLDILOCKS_MOD
        } else {
            value
        };
        Fr::from(reduced)
    }

    pub fn bn254_to_goldilocks(value: &Fr) -> anyhow::Result<u64> {
        let bytes = value.into_bigint().to_bytes_le();

        for &b in bytes.iter().skip(8) {
            if b != 0 {
                anyhow::bail!("Value does not fit in 64 bits");
            }
        }

        let mut arr = [0u8; 8];
        let copy_len = bytes.len().min(8);
        arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
        Ok(u64::from_le_bytes(arr))
    }

    pub fn batch_convert(values: &[u64]) -> Vec<Fr> {
        values.iter().map(|&v| Self::goldilocks_to_bn254(v)).collect()
    }

    pub fn conversion_constraints(num_conversions: usize) -> usize {
        num_conversions * 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goldilocks_to_bn254() {
        let val = 12345u64;
        let fr = FieldConverter::goldilocks_to_bn254(val);
        assert_eq!(fr, Fr::from(val));
    }

    #[test]
    fn test_bn254_to_goldilocks_small_value() {
        let original = 999u64;
        let fr = FieldConverter::goldilocks_to_bn254(original);
        let back = FieldConverter::bn254_to_goldilocks(&fr).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn test_batch_convert() {
        let values = vec![1u64, 2, 3, 4, 5];
        let converted = FieldConverter::batch_convert(&values);
        assert_eq!(converted.len(), 5);
    }
}