use crate::common::pairing::PairingCheck;

pub struct Groth16PairingVerifier;

impl Groth16PairingVerifier {
    pub fn verify_pairing_equation(
        a: &[u8],
        b: &[u8],
        _alpha: &[u8],
        beta_1: &[u8],
        _beta_2: &[u8],
        _vk_x: &[u8],
        _gamma_1: &[u8],
        _gamma_2: &[u8],
        c: &[u8],
        delta_1: &[u8],
        delta_2: &[u8],
    ) -> anyhow::Result<bool> {
        if a.len() != 64 || c.len() != 64 {
            anyhow::bail!("a and c must be 64 bytes each");
        }
        if b.len() != 128 {
            anyhow::bail!("b must be 128 bytes");
        }

        let a_x = &a[..32];
        let a_y = &a[32..];
        let c_x = &c[..32];
        let c_y = &c[32..];

        let result = PairingCheck::verify_pairing(
            a_x, a_y, b, beta_1, c_x, c_y, delta_1, delta_2,
        )?;

        Ok(result)
    }

    pub fn verify_groth16_proof(
        proof_a: &[u8],
        proof_b: &[u8],
        proof_c: &[u8],
        vk_alpha: &[u8],
        vk_beta: &[u8],
        vk_gamma: &[u8],
        vk_delta: &[u8],
        vk_x: &[u8],
    ) -> anyhow::Result<bool> {
        proof_a.validate_groth16_point()?;
        proof_b.validate_groth16_g2_point()?;
        proof_c.validate_groth16_point()?;

        Self::verify_pairing_equation(
            proof_a, proof_b, vk_alpha, vk_beta, &[0u8; 32], vk_x, vk_gamma,
            &[0u8; 32], proof_c, vk_delta, &[0u8; 32],
        )
    }
}

trait ValidatePoint {
    fn validate_groth16_point(&self) -> anyhow::Result<()>;
    fn validate_groth16_g2_point(&self) -> anyhow::Result<()>;
}

impl ValidatePoint for [u8] {
    fn validate_groth16_point(&self) -> anyhow::Result<()> {
        if self.len() != 64 {
            anyhow::bail!("Point must be 64 bytes");
        }
        Ok(())
    }

    fn validate_groth16_g2_point(&self) -> anyhow::Result<()> {
        if self.len() != 128 {
            anyhow::bail!("G2 Point must be 128 bytes");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_verification() {
        let a = vec![1u8; 64];
        let b = vec![2u8; 128];
        let c = vec![3u8; 64];
        let vk_alpha = vec![4u8; 32];
        let vk_beta = vec![5u8; 128];
        let vk_gamma = vec![6u8; 128];
        let vk_delta = vec![7u8; 128];
        let vk_x = vec![8u8; 64];

        let result = Groth16PairingVerifier::verify_groth16_proof(
            &a, &b, &c, &vk_alpha, &vk_beta, &vk_gamma, &vk_delta, &vk_x,
        );

        assert!(result.is_ok());
    }
}