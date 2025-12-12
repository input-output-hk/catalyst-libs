//! Implementation of the Unit Vector ZK argument presented by
//! Zhang, Oliynykov and Balogum in
//! [`A Treasury System for Cryptocurrencies: Enabling Better Collaborative Intelligence`](https://www.ndss-symposium.org/wp-content/uploads/2019/02/ndss2019_02A-2_Zhang_paper.pdf).
//!
//! This implementation follows this [specification](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/crypto/#d-non-interactive-zk-vote-proof)

// cspell: words Zhang, Oliynykov, Balogum

mod challenges;
mod decoding;
mod polynomial;
mod randomness_announcements;
mod utils;

use std::ops::Mul;

use challenges::{calculate_first_challenge_hash, calculate_second_challenge_hash};
use polynomial::{Polynomial, calculate_polynomial_val, generate_polynomial};
use randomness_announcements::{Announcement, BlindingRandomness, ResponseRandomness};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use utils::get_bit;

use crate::crypto::{
    elgamal::{Ciphertext, encrypt},
    group::{GroupElement, Scalar},
    rng::rand_core::CryptoRngCore,
};

/// Unit vector proof struct
///
/// The CBOR CDDL schema:
/// ```cddl
/// row-proof = [0, zkproof-elgamal-ristretto255-unit-vector-with-single-selection ]
///
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection = [ +zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item, zkproof-ed25519-scalar ]
///
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item = ( zkproof-elgamal-announcement, ~elgamal-ristretto255-encrypted-choice, zkproof-ed25519-r-response )
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct UnitVectorProof(
    Vec<Announcement>,
    Vec<Ciphertext>,
    Vec<ResponseRandomness>,
    Scalar,
);

/// Generates a unit vector proof.
///
/// `unit_vector` must be a collection of `Scalar` where only one element is equal to
/// `Scalar::one()` and the others are equal to `Scalar::zero()`.
/// Length of the `unit_vector`, `encryption_randomness`, `ciphertexts` must be equal with
/// each other.
///
/// Pls make sure that you are providing a correct arguments, otherwise
/// the proof will be invalid.
pub fn generate_unit_vector_proof<R: CryptoRngCore>(
    unit_vector: &[Scalar],
    mut ciphertexts: Vec<Ciphertext>,
    mut encryption_randomness: Vec<Scalar>,
    public_key: &GroupElement,
    commitment_key: &GroupElement,
    rng: &mut R,
) -> UnitVectorProof {
    let i = unit_vector
        .iter()
        .position(|s| s != &Scalar::zero())
        .unwrap_or(0);

    let m = unit_vector.len();
    let n = m.next_power_of_two();
    // calculates log_2(N)
    let log_n = n.trailing_zeros();

    encryption_randomness.resize(n, Scalar::zero());
    ciphertexts.resize(n, Ciphertext::zero());

    let blinding_randomness: Vec<_> = (0..log_n)
        .map(|_| BlindingRandomness::random(rng))
        .collect();

    let announcements: Vec<_> = blinding_randomness
        .par_iter()
        .enumerate()
        .map(|(l, r)| {
            let i_bit = get_bit(i, l);
            Announcement::new(i_bit, r, commitment_key)
        })
        .collect();

    let ch_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &announcements);
    let ch_1 = Scalar::from_hash(ch_1_hash.clone());

    let polynomials: Vec<_> = (0..n)
        .map(|j| generate_polynomial(i, j, &blinding_randomness))
        .collect();

    let (d_l, r_l) = generate_dl_and_rl(log_n, &ch_1, public_key, &polynomials, rng);

    let ch_2_hash = calculate_second_challenge_hash(ch_1_hash, &d_l);
    let ch_2 = Scalar::from_hash(ch_2_hash);

    let response_randomness: Vec<_> = blinding_randomness
        .iter()
        .enumerate()
        .map(|(l, r)| {
            let i_bit = get_bit(i, l);
            ResponseRandomness::new(i_bit, r, &ch_2)
        })
        .collect();

    let response = generate_response(log_n, &ch_1, &ch_2, &encryption_randomness, &r_l);

    UnitVectorProof(announcements, d_l, response_randomness, response)
}

/// Generates `D_l` and `R_l` elements
#[allow(clippy::indexing_slicing)]
fn generate_dl_and_rl<R: CryptoRngCore>(
    log_n: u32,
    ch_1: &Scalar,
    public_key: &GroupElement,
    polynomials: &[Polynomial],
    rng: &mut R,
) -> (Vec<Ciphertext>, Vec<Scalar>) {
    let r_l: Vec<_> = (0..log_n).map(|_| Scalar::random(rng)).collect();

    let d_l: Vec<_> = r_l
        .par_iter()
        .enumerate()
        .map(|(l, r_l)| {
            let (sum, _) = polynomials.iter().fold(
                (Scalar::zero(), Scalar::one()),
                // exp_ch_1 = `ch_1^(j)`
                |(mut sum, mut exp_ch_1), pol| {
                    sum = &sum + &pol[l].mul(&exp_ch_1);
                    exp_ch_1 = exp_ch_1.mul(ch_1);
                    (sum, exp_ch_1)
                },
            );
            encrypt(&sum, public_key, r_l)
        })
        .collect();

    (d_l, r_l)
}

/// Generate response element `R`
fn generate_response(
    log_n: u32,
    ch_1: &Scalar,
    ch_2: &Scalar,
    encryption_randomness: &[Scalar],
    r_l: &[Scalar],
) -> Scalar {
    // exp_ch_2 == `ch_2^(log_2(N))`
    let exp_ch_2 = (0..log_n).fold(Scalar::one(), |exp, _| exp.mul(ch_2));

    let (r1, _) = encryption_randomness.iter().fold(
        (Scalar::zero(), Scalar::one()),
        // exp_ch_1 = `ch_1^(j)`
        |(mut sum, mut exp_ch_1), r| {
            sum = &sum + &r.mul(&exp_ch_2).mul(&exp_ch_1);
            exp_ch_1 = exp_ch_1.mul(ch_1);
            (sum, exp_ch_1)
        },
    );
    let (r2, _) = r_l.iter().fold(
        (Scalar::zero(), Scalar::one()),
        // exp_ch_2 = `ch_2^(l)`
        |(mut sum, mut exp_ch_2), r_l| {
            sum = &sum + &r_l.mul(&exp_ch_2);
            exp_ch_2 = exp_ch_2.mul(ch_2);
            (sum, exp_ch_2)
        },
    );
    &r1 + &r2
}

/// Verify a unit vector proof.
#[must_use]
pub fn verify_unit_vector_proof(
    proof: &UnitVectorProof,
    mut ciphertexts: Vec<Ciphertext>,
    public_key: &GroupElement,
    commitment_key: &GroupElement,
) -> bool {
    let m = ciphertexts.len();
    let n = m.next_power_of_two();
    // calculates log_2(N)
    let log_n = n.trailing_zeros();

    ciphertexts.resize(n, Ciphertext::zero());

    let ch_1_hash =
        calculate_first_challenge_hash(commitment_key, public_key, &ciphertexts, &proof.0);
    let ch_1 = Scalar::from_hash(ch_1_hash.clone());

    let ch_2_hash = calculate_second_challenge_hash(ch_1_hash, &proof.1);
    let ch_2 = Scalar::from_hash(ch_2_hash);

    check_1(proof, &ch_2, commitment_key)
        && check_2(proof, log_n, &ch_1, &ch_2, &ciphertexts, public_key)
}

/// Check the first part of the proof
fn check_1(
    proof: &UnitVectorProof,
    ch_2: &Scalar,
    commitment_key: &GroupElement,
) -> bool {
    proof.0.iter().zip(proof.2.iter()).all(|(an, rand)| {
        let right = &an.i.mul(ch_2) + &an.b;
        let left = &GroupElement::GENERATOR.mul(&rand.z) + &commitment_key.mul(&rand.w);
        let eq_1 = right == left;

        let right = &an.i.mul(&(ch_2 - &rand.z)) + &an.a;
        let left = &GroupElement::GENERATOR.mul(&Scalar::zero()) + &commitment_key.mul(&rand.v);
        let eq_2 = right == left;

        eq_1 && eq_2
    })
}

/// Check the second part of the proof
fn check_2(
    proof: &UnitVectorProof,
    log_n: u32,
    ch_1: &Scalar,
    ch_2: &Scalar,
    ciphertexts: &[Ciphertext],
    public_key: &GroupElement,
) -> bool {
    let left = encrypt(&Scalar::zero(), public_key, &proof.3);

    let (right_2, _) = proof.1.iter().fold(
        (Ciphertext::zero(), Scalar::one()),
        // exp_ch_2 = `ch_2^(l)`
        |(mut sum, mut exp_ch_2), d_l| {
            sum = &sum + &d_l.mul(&exp_ch_2);
            exp_ch_2 = exp_ch_2.mul(ch_2);
            (sum, exp_ch_2)
        },
    );

    let p_j: Vec<_> = (0..ciphertexts.len())
        .map(|j| calculate_polynomial_val(j, ch_2, &proof.2))
        .map(|p_ch_2| encrypt(&p_ch_2.negate(), public_key, &Scalar::zero()))
        .collect();

    // exp_ch_2 == `ch_2^(log_2(N))`
    let exp_ch_2 = (0..log_n).fold(Scalar::one(), |exp, _| exp.mul(ch_2));

    let (right_1, _) = p_j.iter().zip(ciphertexts.iter()).fold(
        (Ciphertext::zero(), Scalar::one()),
        // exp_ch_1 = `ch_1^(j)`
        |(mut sum, mut exp_ch_1), (p_j, c_j)| {
            sum = &sum + &(&c_j.mul(&exp_ch_2) + p_j).mul(&exp_ch_1);
            exp_ch_1 = exp_ch_1.mul(ch_1);
            (sum, exp_ch_1)
        },
    );

    &right_1 + &right_2 == left
}

#[cfg(test)]
mod arbitrary_impl {
    use proptest::{
        prelude::{Arbitrary, BoxedStrategy, Strategy, any_with},
        sample::size_range,
    };

    use super::{Announcement, Ciphertext, ResponseRandomness, Scalar, UnitVectorProof};

    impl Arbitrary for UnitVectorProof {
        type Parameters = usize;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(size: Self::Parameters) -> Self::Strategy {
            any_with::<(
                Vec<((Announcement, Ciphertext), ResponseRandomness)>,
                Scalar,
            )>(((size_range(size + 1), (((), ()), ())), ()))
            .prop_map(|(val, scalar)| {
                let (vec, rr): (Vec<_>, Vec<_>) = val.into_iter().unzip();
                let (an, cipher) = vec.into_iter().unzip();
                Self(an, cipher, rr, scalar)
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use proptest::sample::size_range;
    use rand_core::OsRng;
    use test_strategy::proptest;

    use super::{super::elgamal::generate_public_key, *};

    fn is_unit_vector(vector: &[Scalar]) -> bool {
        let ones = vector.iter().filter(|s| s == &&Scalar::one()).count();
        let zeros = vector.iter().filter(|s| s == &&Scalar::zero()).count();
        ones == 1 && zeros == vector.len() - 1
    }

    #[proptest(cases = 10)]
    fn zk_unit_vector_test(
        secret_key: Scalar,
        commitment_key: GroupElement,
        #[strategy(1..10_usize)] unit_vector_size: usize,
        #[strategy(0..#unit_vector_size)] unit_vector_index: usize,
    ) {
        let mut rng = OsRng;

        let public_key = generate_public_key(&secret_key);

        let unit_vector: Vec<_> = (0..unit_vector_size)
            .map(|i| {
                if i == unit_vector_index {
                    Scalar::one()
                } else {
                    Scalar::zero()
                }
            })
            .collect();

        assert!(is_unit_vector(&unit_vector));

        let encryption_randomness: Vec<_> = unit_vector
            .iter()
            .map(|_| Scalar::random(&mut rng))
            .collect();

        let ciphertexts: Vec<_> = encryption_randomness
            .iter()
            .zip(unit_vector.iter())
            .map(|(r, v)| encrypt(v, &public_key, r))
            .collect();

        let proof = generate_unit_vector_proof(
            &unit_vector,
            ciphertexts.clone(),
            encryption_randomness,
            &public_key,
            &commitment_key,
            &mut rng,
        );

        assert!(verify_unit_vector_proof(
            &proof,
            ciphertexts,
            &public_key,
            &commitment_key
        ));
    }

    #[proptest(cases = 10)]
    fn not_a_unit_vector_test(
        secret_key: Scalar,
        commitment_key: GroupElement,
        #[any(size_range(1..10_usize).lift())] random_vector: Vec<Scalar>,
    ) {
        let mut rng = OsRng;

        // make sure the `random_vector` is not a unit vector
        // if it is early return
        if is_unit_vector(&random_vector) {
            return Ok(());
        }

        let public_key = generate_public_key(&secret_key);

        let encryption_randomness: Vec<_> = random_vector
            .iter()
            .map(|_| Scalar::random(&mut rng))
            .collect();

        let ciphertexts: Vec<_> = encryption_randomness
            .iter()
            .zip(random_vector.iter())
            .map(|(r, v)| encrypt(v, &public_key, r))
            .collect();

        let proof = generate_unit_vector_proof(
            &random_vector,
            ciphertexts.clone(),
            encryption_randomness,
            &public_key,
            &commitment_key,
            &mut rng,
        );

        assert!(!verify_unit_vector_proof(
            &proof,
            ciphertexts,
            &public_key,
            &commitment_key
        ));
    }
}
