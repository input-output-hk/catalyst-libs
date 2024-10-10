//! Voter objects decoding implementation.

use std::io::Read;

use anyhow::anyhow;

use super::{proof::VoterProof, EncryptedVote};
use crate::crypto::{elgamal::Ciphertext, zk_unit_vector::UnitVectorProof};

impl EncryptedVote {
    /// Decode `EncryptedVote` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode ciphertext.
    pub fn from_bytes(mut bytes: &[u8], size: usize) -> anyhow::Result<Self> {
        let mut ciph_buf = [0u8; Ciphertext::BYTES_SIZE];

        let ciphertexts = (0..size)
            .map(|i| {
                bytes.read_exact(&mut ciph_buf)?;
                Ciphertext::from_bytes(&ciph_buf)
                    .map_err(|e| anyhow!("Cannot decode ciphertext at {i}, error: {e}"))
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(Self(ciphertexts))
    }

    /// Get a deserialized bytes size
    #[must_use]
    pub fn bytes_size(&self) -> usize {
        self.0.len() * Ciphertext::BYTES_SIZE
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.bytes_size());
        self.0
            .iter()
            .for_each(|c| res.extend_from_slice(&c.to_bytes()));
        res
    }
}

impl VoterProof {
    /// Decode `VoterProof` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode announcement value.
    ///   - Cannot decode ciphertext value.
    ///   - Cannot decode response randomness value.
    ///   - Cannot decode scalar value.
    pub fn from_bytes(bytes: &[u8], size: usize) -> anyhow::Result<Self> {
        UnitVectorProof::from_bytes(bytes, size).map(Self)
    }

    /// Get a deserialized bytes size
    #[must_use]
    pub fn bytes_size(&self) -> usize {
        self.0.bytes_size()
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use proptest::sample::size_range;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn encrypted_vote_to_bytes_from_bytes_test(
        #[any(size_range(0..u8::MAX as usize).lift())] ciphers: Vec<Ciphertext>,
    ) {
        let vote1 = EncryptedVote(ciphers);
        let bytes = vote1.to_bytes();
        assert_eq!(bytes.len(), vote1.bytes_size());
        let vote2 = EncryptedVote::from_bytes(&bytes, vote1.0.len()).unwrap();
        assert_eq!(vote1, vote2);
    }
}
