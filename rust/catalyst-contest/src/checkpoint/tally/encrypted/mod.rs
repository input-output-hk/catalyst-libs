//! Encrypted Tally related code.
mod proposal_result;

use std::collections::HashMap;

use catalyst_signed_doc::DocumentRef;
use cbork_utils::{decode_context::DecodeCtx, map::Map};
use minicbor::{
    Decode, Decoder, Encode, Encoder, decode::Error as DecodeError, encode::Error as EncodeError,
};
pub use proposal_result::EncryptedTallyProposalResult;

/// Placeholder map of `document_ref => encrypted-tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::zero_sized_map_values)]
pub struct EncryptedTally(HashMap<DocumentRef, EncryptedTallyProposalResult>);

impl Encode<()> for EncryptedTally {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.map(self.0.len() as u64)?;

        // Sort entries by their CBOR-encoded key for RFC 8949 canonical ordering
        // (length-first, then lexicographic for equal-length keys)
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_cached_key(|(doc_ref, _)| {
            let mut buf = Vec::new();
            drop(doc_ref.encode(&mut Encoder::new(&mut buf), &mut ()));
            buf
        });

        for (doc_ref, proposal_result) in entries {
            doc_ref.encode(e, ctx)?;
            proposal_result.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for EncryptedTally {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = Map::decode(d, &mut DecodeCtx::Deterministic)?;

        let mut tally = HashMap::new();
        for entry in entries.as_slice() {
            let mut key_decoder = Decoder::new(&entry.key_bytes);
            let doc_ref = DocumentRef::decode(&mut key_decoder, ctx)?;

            let mut value_decoder = Decoder::new(&entry.value);
            let proposal_result = EncryptedTallyProposalResult::decode(&mut value_decoder, ctx)?;

            if tally.insert(doc_ref, proposal_result).is_some() {
                return Err(DecodeError::message("Duplicate document reference key"));
            }
        }

        Ok(Self(tally))
    }
}
