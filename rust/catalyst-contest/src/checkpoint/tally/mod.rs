//! Tally related code
pub(super) mod encrypted;

mod clear_choice;
mod proposal_result;
mod voting_power;

use std::collections::HashMap;

use catalyst_signed_doc::DocumentRef;
use cbork_utils::{decode_context::DecodeCtx, map::Map};
pub use clear_choice::ClearChoice;
use minicbor::{
    Decode, Decoder, Encode, Encoder, decode::Error as DecodeError, encode::Error as EncodeError,
};
pub use proposal_result::ProposalResult;
pub use voting_power::VotingPower;

/// Tally map of `document_ref => tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tally(HashMap<DocumentRef, ProposalResult>);

impl Encode<()> for Tally {
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

impl Decode<'_, ()> for Tally {
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
            let proposal_result = ProposalResult::decode(&mut value_decoder, ctx)?;

            if tally.insert(doc_ref, proposal_result).is_some() {
                return Err(DecodeError::message("Duplicate document reference key"));
            }
        }

        Ok(Self(tally))
    }
}

#[cfg(test)]
mod tests {
    use catalyst_signed_doc::tests_utils::create_dummy_doc_ref;

    use super::*;

    #[test]
    fn roundtrip() {
        let doc_ref1 = create_dummy_doc_ref();
        let doc_ref2 = create_dummy_doc_ref();

        let mut tally_map = std::collections::HashMap::new();
        tally_map.insert(doc_ref1, ProposalResult::default());
        tally_map.insert(doc_ref2, ProposalResult::default());

        let original = Tally(tally_map);

        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Tally::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_empty() {
        let original = Tally::default();

        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Tally::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
