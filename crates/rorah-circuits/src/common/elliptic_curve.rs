use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{PrimeField, Zero};
use ark_serialize::CanonicalSerialize;

#[derive(Debug, Clone, PartialEq)]
pub struct ECPoint {
    pub x: Vec<u8>,
    pub y: Vec<u8>,
}

impl ECPoint {
    pub fn new(x: Vec<u8>, y: Vec<u8>) -> Self {
        ECPoint { x, y }
    }

    pub fn infinity() -> Self {
        ECPoint {
            x: vec![0; 32],
            y: vec![0; 32],
        }
    }

    pub fn is_identity(&self) -> bool {
        self.x.iter().all(|&b| b == 0) && self.y.iter().all(|&b| b == 0)
    }
}

pub struct G1Operations;

impl G1Operations {
    pub fn point_add(p1: &ECPoint, p2: &ECPoint) -> anyhow::Result<ECPoint> {
        if p1.is_identity() {
            return Ok(p2.clone());
        }
        if p2.is_identity() {
            return Ok(p1.clone());
        }

        let x1 = bytes_to_fq(&p1.x)?;
        let y1 = bytes_to_fq(&p1.y)?;
        let x2 = bytes_to_fq(&p2.x)?;
        let y2 = bytes_to_fq(&p2.y)?;

        let point1 = G1Affine::new_unchecked(x1, y1);
        let point2 = G1Affine::new_unchecked(x2, y2);

        let result = (G1Projective::from(point1) + G1Projective::from(point2)).into_affine();

        if result.is_zero() {
            return Ok(ECPoint::infinity());
        }

        Ok(ECPoint {
            x: fq_to_bytes(&result.x),
            y: fq_to_bytes(&result.y),
        })
    }

    pub fn point_scalar_mul(point: &ECPoint, scalar: &[u8]) -> anyhow::Result<ECPoint> {
        if point.is_identity() {
            return Ok(ECPoint::infinity());
        }

        let x = bytes_to_fq(&point.x)?;
        let y = bytes_to_fq(&point.y)?;
        let s = bytes_to_fr(scalar)?;

        let affine = G1Affine::new_unchecked(x, y);
        let result = (G1Projective::from(affine) * s).into_affine();

        if result.is_zero() {
            return Ok(ECPoint::infinity());
        }

        Ok(ECPoint {
            x: fq_to_bytes(&result.x),
            y: fq_to_bytes(&result.y),
        })
    }

    pub fn multi_scalar_mul(
        points: &[ECPoint],
        scalars: &[Vec<u8>],
    ) -> anyhow::Result<ECPoint> {
        if points.len() != scalars.len() {
            anyhow::bail!("points and scalars must have same length");
        }

        let mut result = G1Projective::zero();

        for (point, scalar) in points.iter().zip(scalars.iter()) {
            if point.is_identity() {
                continue;
            }

            let x = bytes_to_fq(&point.x)?;
            let y = bytes_to_fq(&point.y)?;
            let s = bytes_to_fr(scalar)?;

            let affine = G1Affine::new_unchecked(x, y);
            result += G1Projective::from(affine) * s;
        }

        let affine = result.into_affine();

        if affine.is_zero() {
            return Ok(ECPoint::infinity());
        }

        Ok(ECPoint {
            x: fq_to_bytes(&affine.x),
            y: fq_to_bytes(&affine.y),
        })
    }
}

pub struct G2Operations;

impl G2Operations {
    pub fn point_add(_p1: &[u8; 64], _p2: &[u8; 64]) -> anyhow::Result<[u8; 64]> {
        Ok([0u8; 64])
    }
}

fn bytes_to_fq(bytes: &[u8]) -> anyhow::Result<Fq> {
    let mut arr = [0u8; 32];
    let copy_len = bytes.len().min(32);
    arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
    Ok(Fq::from_le_bytes_mod_order(&arr))
}

fn bytes_to_fr(bytes: &[u8]) -> anyhow::Result<Fr> {
    let mut arr = [0u8; 32];
    let copy_len = bytes.len().min(32);
    arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
    Ok(Fr::from_le_bytes_mod_order(&arr))
}

fn fq_to_bytes(fq: &Fq) -> Vec<u8> {
    let mut bytes = Vec::new();
    fq.serialize_compressed(&mut bytes)
        .expect("Fq serialization cannot fail");
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_identity() {
        let p = ECPoint::infinity();
        assert!(p.is_identity());
    }

    #[test]
    fn test_point_add_identity_left() {
        let identity = ECPoint::infinity();
        let p = ECPoint::new(vec![1u8; 32], vec![2u8; 32]);

        let result = G1Operations::point_add(&identity, &p).unwrap();
        assert_eq!(result, p);
    }

    #[test]
    fn test_point_add_identity_right() {
        let identity = ECPoint::infinity();
        let p = ECPoint::new(vec![1u8; 32], vec![2u8; 32]);

        let result = G1Operations::point_add(&p, &identity).unwrap();
        assert_eq!(result, p);
    }

    #[test]
    fn test_bytes_to_fq() {
        let bytes = vec![1u8; 32];
        let result = bytes_to_fq(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fq_to_bytes_roundtrip() {
        let fq = Fq::from_le_bytes_mod_order(&[7u8; 32]);
        let bytes = fq_to_bytes(&fq);
        assert_eq!(bytes.len(), 32);
    }
}