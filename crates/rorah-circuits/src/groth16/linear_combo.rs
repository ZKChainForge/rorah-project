use crate::common::elliptic_curve::{ECPoint, G1Operations};
use ark_bn254::Fr;
use ark_ff::Field;

pub struct LinearComboComputer;

impl LinearComboComputer {
    pub fn compute_vk_x(
        public_inputs: &[Vec<u8>],
        gamma_abc: &[ECPoint],
    ) -> anyhow::Result<ECPoint> {
        if public_inputs.len() + 1 != gamma_abc.len() {
            anyhow::bail!("public_inputs + 1 must equal gamma_abc length");
        }

        let mut points = vec![gamma_abc[0].clone()];
        let mut scalars = vec![vec![1u8; 32]];

        for (i, input) in public_inputs.iter().enumerate() {
            points.push(gamma_abc[i + 1].clone());
            scalars.push(input.clone());
        }

        G1Operations::multi_scalar_mul(&points, &scalars)
    }

    pub fn verify_linear_combination(
        vk_x: &ECPoint,
        public_inputs: &[Vec<u8>],
        gamma_abc: &[ECPoint],
    ) -> anyhow::Result<bool> {
        let computed = Self::compute_vk_x(public_inputs, gamma_abc)?;

        Ok(computed.x == vk_x.x && computed.y == vk_x.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_combo_computation() {
        let point1 = ECPoint::new(vec![1u8; 32], vec![2u8; 32]);
        let point2 = ECPoint::new(vec![3u8; 32], vec![4u8; 32]);

        let gamma_abc = vec![point1, point2];
        let public_inputs = vec![vec![5u8; 32]];

        let result = LinearComboComputer::compute_vk_x(&public_inputs, &gamma_abc);
        assert!(result.is_ok());
    }
}