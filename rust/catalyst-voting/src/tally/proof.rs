//! Tally proof generation and verification procedures.
//! It allows to transparently verify the correctness of decryption tally procedure.

use rand_core::CryptoRngCore;

use super::EncryptedTally;
use crate::SecretKey;

/// Generates a tally proof.
/// More detailed described [here](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/crypto/#tally-proof)
#[allow(clippy::module_name_repetitions)]
pub fn generate_tally_proof<R: CryptoRngCore>(
    _tally_result: &EncryptedTally, _secret_key: &SecretKey, _rng: &mut R,
) {
}
