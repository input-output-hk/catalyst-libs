//! A private vote tx proof struct.

#![allow(dead_code)]

use catalyst_voting::vote_protocol::voter::proof::VoterProof;
use minicbor::{Decode, Encode};

/// `Proof` array struct length
const PROOF_LEN: u64 = 2;

/// A private voting proof struct, CBOR `undefined`.
#[derive(Debug, Clone, PartialEq)]
pub struct Proof(pub VoterProof);

impl Decode<'_, ()> for Proof {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let mut proof_bytes = d.bytes()?;
        let proof_bytes_len = proof_bytes.len();
        let proof = VoterProof::from_bytes_with_size(&mut proof_bytes, proof_bytes_len)
            .map_err(minicbor::decode::Error::message)?;
        Ok(Self(proof))
    }
}

impl Encode<()> for Proof {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let proof_bytes = self.0.to_bytes();
        e.bytes(&proof_bytes)?;
        Ok(())
    }
}
