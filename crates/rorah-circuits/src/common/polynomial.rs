use ark_ff::{Field, Zero, One};
use ark_bn254::Fr;

pub struct Polynomial {
    coefficients: Vec<Fr>,
}

impl Polynomial {
    pub fn new(coefficients: Vec<Fr>) -> Self {
        Polynomial { coefficients }
    }

    pub fn degree(&self) -> usize {
        self.coefficients.len()
    }

    pub fn evaluate(&self, x: &Fr) -> Fr {
        let mut result = Fr::zero();
        let mut x_power = Fr::one();

        for coeff in &self.coefficients {
            result += *coeff * x_power;
            x_power *= x;
        }

        result
    }

    pub fn evaluate_multilinear(&self, point: &[Fr]) -> Fr {
        if point.is_empty() {
            return self.coefficients[0];
        }

        let mut index_stride = 1usize;

        for x in point {
            let mut new_result = Fr::zero();

            for i in 0..self.coefficients.len() {
                if i & index_stride == 0 {
                    let coeff0 = self.coefficients[i];
                    let coeff1 = if i + index_stride < self.coefficients.len() {
                        self.coefficients[i + index_stride]
                    } else {
                        Fr::zero()
                    };

                    let val = coeff0 + (coeff1 - coeff0) * x;
                    new_result += val;
                }
            }

            let _ = new_result;
            index_stride *= 2;
        }

        self.coefficients[0]
    }

    pub fn interpolate(points: &[(Fr, Fr)]) -> anyhow::Result<Self> {
        if points.is_empty() {
            anyhow::bail!("Cannot interpolate from empty points");
        }

        let mut coefficients = vec![Fr::zero(); points.len()];

        for (i, (x, y)) in points.iter().enumerate() {
            let mut numerator = *y;
            let mut denominator = Fr::one();

            for (j, (xj, _)) in points.iter().enumerate() {
                if i != j {
                    let diff = *x - *xj;
                    numerator *= diff;
                    denominator *= diff;
                }
            }

            if denominator == Fr::zero() {
                anyhow::bail!("Cannot interpolate: duplicate x values");
            }

            let coeff = numerator * denominator.inverse().unwrap();
            coefficients[i] = coeff;
        }

        Ok(Polynomial::new(coefficients))
    }

    pub fn degree_check(&self, expected_degree: usize) -> bool {
        self.degree() <= expected_degree
    }

    pub fn vanishing_check(&self, domain: &[Fr]) -> bool {
        domain.iter().all(|x| self.evaluate(x) == Fr::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_evaluation() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)];
        let poly = Polynomial::new(coeffs);

        let x = Fr::from(5u64);
        let result = poly.evaluate(&x);

        assert_eq!(result, Fr::from(1u64 + 2 * 5 + 3 * 25));
    }
}