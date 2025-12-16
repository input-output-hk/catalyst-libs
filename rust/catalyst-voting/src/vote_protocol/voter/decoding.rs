//! Voter objects decoding implementation.

use std::io::Read;

use anyhow::anyhow;

use super::EncryptedVote;
use crate::{crypto::elgamal::Ciphertext, utils::read_array};

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
    pub fn from_bytes<R: Read>(
        reader: &mut R,
        size: usize,
    ) -> anyhow::Result<Self> {
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

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use std::io::Cursor;

    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn encrypted_vote_to_bytes_from_bytes_test(
        #[strategy(0..5usize)] _size: usize,
        #[any(#_size)] vote1: EncryptedVote,
    ) {
        let bytes = vote1.to_bytes();
        assert_eq!(bytes.len(), vote1.bytes_size());
        let vote2 = EncryptedVote::from_bytes(&mut Cursor::new(bytes), vote1.size()).unwrap();
        assert_eq!(vote1, vote2);
    }
}
