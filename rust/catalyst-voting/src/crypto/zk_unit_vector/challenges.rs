//! ZK unit vector challenges calculation functionality

use crate::crypto::{
    elgamal::Ciphertext,
    group::GroupElement,
    hash::{digest::Digest, Blake2b512Hasher},
    zk_unit_vector::randomness_announcements::Announcement,
};

/// Calculates the first challenge hash.
pub(crate) fn calculate_first_challenge_hash(
    commitment_key: &GroupElement, public_key: &GroupElement, ciphertexts: &[Ciphertext],
    announcements: &[Announcement],
) -> Blake2b512Hasher {
    let mut hash = Blake2b512Hasher::new()
        .chain_update(commitment_key.to_bytes())
        .chain_update(public_key.to_bytes());
    for c in ciphertexts {
        hash.update(c.first().to_bytes());
        hash.update(c.second().to_bytes());
    }
    for announcement in announcements {
        hash.update(announcement.i.to_bytes());
        hash.update(announcement.b.to_bytes());
        hash.update(announcement.a.to_bytes());
    }
    hash
}

/// Calculates the second challenge hash.
pub(crate) fn calculate_second_challenge_hash(
    mut com_1_hash: Blake2b512Hasher, ciphertexts: &[Ciphertext],
) -> Blake2b512Hasher {
    for c in ciphertexts {
        com_1_hash.update(c.first().to_bytes());
        com_1_hash.update(c.second().to_bytes());
    }
    com_1_hash
}
