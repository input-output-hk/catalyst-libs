//! Voter objects decoding implementation.

use std::io::Read;

use anyhow::anyhow;

use super::{proof::VoterProof, EncryptedVote};
use crate::{
    crypto::{elgamal::Ciphertext, zk_unit_vector::UnitVectorProof},
    utils::read_array,
};

impl EncryptedVote {
    /// Get an underlying vector length.
    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Decode `EncryptedVote` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode ciphertext.
    pub fn from_bytes<R: Read>(reader: &mut R, size: usize) -> anyhow::Result<Self> {
        let ciphertexts = (0..size)
            .map(|i| {
                let bytes = read_array(reader)?;
                Ciphertext::from_bytes(&bytes)
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
    /// Get an underlying vector length.
    ///
    /// **Note** each vector field has the same length.
    #[must_use]
    pub fn size(&self) -> usize {
        self.0.size()
    }

    /// Decode `VoterProof` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode announcement value.
    ///   - Cannot decode ciphertext value.
    ///   - Cannot decode response randomness value.
    ///   - Cannot decode scalar value.
    pub fn from_bytes<R: Read>(reader: &mut R, len: usize) -> anyhow::Result<Self> {
        UnitVectorProof::from_bytes(reader, len).map(Self)
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use std::io::Cursor;

    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn encrypted_vote_to_bytes_from_bytes_test(
        #[strategy(0..5usize)] _size: usize, #[any(#_size)] vote1: EncryptedVote,
    ) {
        let bytes = vote1.to_bytes();
        assert_eq!(bytes.len(), vote1.bytes_size());
        let vote2 = EncryptedVote::from_bytes(&mut Cursor::new(bytes), vote1.size()).unwrap();
        assert_eq!(vote1, vote2);
    }
}
