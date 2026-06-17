use ark_bn254::{Bn254, Fq, G1Affine, G2Affine};
use ark_ec::{AffineRepr, pairing::Pairing};
use ark_ff::{PrimeField, Zero};
use ark_serialize::CanonicalSerialize;

pub struct PairingCheck;

impl PairingCheck {
    pub fn verify_pairing(
        a1_x: &[u8],
        a1_y: &[u8],
        b1_x: &[u8],
        b1_y: &[u8],
        a2_x: &[u8],
        a2_y: &[u8],
        b2_x: &[u8],
        b2_y: &[u8],
    ) -> anyhow::Result<bool> {
        let p_a = bytes_to_g1(a1_x, a1_y)?;
        let p_b = bytes_to_g1(b1_x, b1_y)?;
        let p_c = bytes_to_g1(a2_x, a2_y)?;
        let _p_d = bytes_to_g1(b2_x, b2_y)?;

        let g2_gen = G2Affine::generator();

        let pairing1 = Bn254::pairing(&p_a, &g2_gen);
        let pairing2 = Bn254::pairing(&p_b, &g2_gen);

        Ok(pairing1 == pairing2)
    }

    pub fn groth16_pairing_check(
        a: &G1Affine,
        b: &G2Affine,
        alpha: &G1Affine,
        beta: &G2Affine,
        vk_x: &G1Affine,
        gamma: &G2Affine,
        c: &G1Affine,
        delta: &G2Affine,
    ) -> bool {
        let neg_a = -*a;

        let ml = Bn254::multi_pairing(
            [neg_a, *alpha, *vk_x, *c],
            [*b, *beta, *gamma, *delta],
        );

        ml.0 == ark_bn254::Fq12::zero()
    }
}

fn bytes_to_g1(x: &[u8], y: &[u8]) -> anyhow::Result<G1Affine> {
    if x.len() > 32 {
        anyhow::bail!("x coordinate bytes too long: {}", x.len());
    }
    if y.len() > 32 {
        anyhow::bail!("y coordinate bytes too long: {}", y.len());
    }

    let mut x_arr = [0u8; 32];
    let mut y_arr = [0u8; 32];

    x_arr[..x.len()].copy_from_slice(x);
    y_arr[..y.len()].copy_from_slice(y);

    let x_field = Fq::from_le_bytes_mod_order(&x_arr);
    let y_field = Fq::from_le_bytes_mod_order(&y_arr);

    Ok(G1Affine::new_unchecked(x_field, y_field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_g1() {
        let x = vec![1u8; 32];
        let y = vec![2u8; 32];
        let result = bytes_to_g1(&x, &y);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pairing_check_runs() {
        let result = PairingCheck::verify_pairing(
            &[1u8; 32],
            &[2u8; 32],
            &[3u8; 32],
            &[4u8; 32],
            &[5u8; 32],
            &[6u8; 32],
            &[7u8; 32],
            &[8u8; 32],
        );
        assert!(result.is_ok());
    }
}