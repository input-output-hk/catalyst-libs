//! Polynomial generation implementation for the ZK unit vector algorithm

use std::ops::{Deref, Mul};

use super::{bin_rep, randomness_announcements::BlindingRandomness};
use crate::crypto::group::Scalar;

/// Polynomial representation in the following form:
/// `p_0 + p_1 * x + p_2 * x^2 + ... + p_n * x^n`
pub struct Polynomial(Vec<Scalar>);

impl Deref for Polynomial {
    type Target = Vec<Scalar>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Polynomial {
    /// Multiplication of a current polynomial on a degree-1 polynomial `a * x + b`.
    #[allow(clippy::indexing_slicing)]
    fn pol_mul(&mut self, a: &Scalar, b: &Scalar) {
        self.0.push(&self.0[self.0.len() - 1] * a);
        for i in (1..self.0.len() - 1).rev() {
            self.0[i] = &(&self.0[i] * b) + &(&self.0[i - 1] * a);
        }
        self.0[0] = &self.0[0] * b;
    }

    /// Multiplication of a current polynomial on scalar.
    fn scalar_mul(&mut self, a: &Scalar) {
        self.0.iter_mut().for_each(|v| *v = v.mul(a));
    }
}

/// Generate the polynomial according to the step 7 of this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#prover)
pub(crate) fn generate_polynomial(
    i_bits: &[bool], randomness: &[BlindingRandomness], j: usize, log_n: u32,
) -> Polynomial {
    let j_bits = bin_rep(j, log_n);

    let mut pol = j_bits
        .iter()
        .zip(i_bits.iter())
        .zip(randomness.iter().map(|r| &r.betta))
        .fold(
            // `0 * x + 1`
            Polynomial(vec![Scalar::one()]),
            |mut acc, ((j_bit, i_bit), betta)| {
                match (*j_bit, *i_bit) {
                    // `1 * x + beta`
                    (true, true) => acc.pol_mul(&Scalar::one(), betta),
                    // `0 * x + beta`
                    (true, false) => acc.scalar_mul(betta),
                    // `0 * x - beta`
                    (false, true) => acc.scalar_mul(&betta.negate()),
                    // `1 * x - beta`
                    (false, false) => acc.pol_mul(&Scalar::one(), &betta.negate()),
                }
                acc
            },
        );

    pol.0.resize((log_n + 1) as usize, Scalar::zero());
    pol
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[test]
    fn polynomial_test() {
        // `3 * x + 2`
        let mut p = Polynomial(vec![Scalar::from(2), Scalar::from(3)]);

        // multiply on `5 * x + 2`
        p.pol_mul(&Scalar::from(5), &Scalar::from(2));
        // `(3 * x + 2) * (5 * x + 2) == 15 * x^2 + 16 * x + 4`
        assert_eq!(p.0, vec![
            Scalar::from(4),
            Scalar::from(16),
            Scalar::from(15),
        ]);

        // multiply on `-7 * x - 3`
        p.pol_mul(&Scalar::from(7).negate(), &Scalar::from(3).negate());
        // `(15 * x^2 + 16 * x + 4) * (7 * x - 3) == -105 * x^2 - 157 * x - 76 * x - 12`
        assert_eq!(p.0, vec![
            Scalar::from(12).negate(),
            Scalar::from(76).negate(),
            Scalar::from(157).negate(),
            Scalar::from(105).negate(),
        ]);

        p.scalar_mul(&Scalar::from(2).negate());
        // `(-105 * x^2 - 157 * x - 76 * x - 12) * -2 == 210 * x^2 + 314 * x + 152 * x + 24`
        assert_eq!(p.0, vec![
            Scalar::from(24),
            Scalar::from(152),
            Scalar::from(314),
            Scalar::from(210),
        ]);
    }

    const N: usize = 4;
    const LOG_N: u32 = 2;

    #[proptest]
    fn generate_polynomial_test(betta: Scalar, randomnesses: [BlindingRandomness; LOG_N as usize]) {
        for i in 0..N {
            let i_bits = bin_rep(i, LOG_N);
            for j in 0..N {
                let p = generate_polynomial(&i_bits, &randomnesses, j, LOG_N);

                assert_eq!(p.0.len(), (LOG_N + 1) as usize);
                if i == j {
                    assert_eq!(p.0.last().unwrap(), &Scalar::one());
                } else {
                    assert_eq!(p.0.last().unwrap(), &Scalar::zero());
                }
            }
        }
    }
}
