use crate::common::elliptic_curve::{ECPoint, G1Operations};

pub struct MSM;

impl MSM {
    pub fn compute_msm(
        points: &[ECPoint],
        scalars: &[Vec<u8>],
    ) -> anyhow::Result<ECPoint> {
        if points.len() != scalars.len() {
            anyhow::bail!("points and scalars must have same length");
        }

        G1Operations::multi_scalar_mul(points, scalars)
    }

    pub fn pippenger_msm(
        points: &[ECPoint],
        scalars: &[Vec<u8>],
    ) -> anyhow::Result<ECPoint> {
        Self::compute_msm(points, scalars)
    }

    pub fn verify_msm_result(
        points: &[ECPoint],
        scalars: &[Vec<u8>],
        expected_result: &ECPoint,
    ) -> anyhow::Result<bool> {
        let computed = Self::compute_msm(points, scalars)?;

        Ok(computed.x == expected_result.x && computed.y == expected_result.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msm_computation() {
        let point = ECPoint::new(vec![1u8; 32], vec![2u8; 32]);
        let scalar = vec![3u8; 32];

        let result = MSM::compute_msm(&[point], &[scalar]);
        assert!(result.is_ok());
    }
}