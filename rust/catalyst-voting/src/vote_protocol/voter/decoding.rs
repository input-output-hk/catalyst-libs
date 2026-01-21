//! Voter objects decoding implementation.

use std::io::Read;

use anyhow::anyhow;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use super::EncryptedVote;
use crate::{
    crypto::{elgamal::Ciphertext, zk_unit_vector::UnitVectorProof},
    utils::read_array,
    vote_protocol::voter::proof::VoterProof,
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

impl Encode<()> for EncryptedVote {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for EncryptedVote {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self(Vec::<Ciphertext>::decode(d, ctx)?))
    }
}

impl Encode<()> for VoterProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for VoterProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self(UnitVectorProof::decode(d, ctx)?))
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

    #[proptest]
    fn encrypted_vote_cbor_roundtrip(original: EncryptedVote) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = EncryptedVote::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[proptest]
    fn voter_proof_cbor_roundtrip(original: VoterProof) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = VoterProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
