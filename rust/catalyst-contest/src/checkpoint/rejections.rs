//! A map for rejected ballots by reason.

use std::collections::HashMap;

use catalyst_signed_doc::{DocumentRef, DocumentRefs};
use cbork_utils::{decode_context::DecodeCtx, map::Map};
use minicbor::{
    Decode, Decoder, Encode, Encoder, decode::Error as DecodeError, encode::Error as EncodeError,
};
use strum::EnumCount;

use crate::checkpoint::RejectionReason;

/// A Map for Rejected Contest Ballots by Reason.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Rejections(pub(crate) HashMap<RejectionReason, DocumentRefs>);

impl Encode<()> for Rejections {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.map(self.0.len() as u64)?;

        // Sort entries by their CBOR-encoded key for RFC 8949 canonical ordering
        // (length-first, then lexicographic for equal-length keys)
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_cached_key(|(reason, _)| {
            let mut buf = Vec::new();
            drop(reason.encode(&mut Encoder::new(&mut buf), &mut ()));
            buf
        });

        for (reason, doc_refs) in entries {
            reason.encode(e, ctx)?;
            doc_refs.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for Rejections {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = Map::decode(d, &mut DecodeCtx::Deterministic)?;
        let map_len = entries.len() as u64;

        // Limit map size to the number of RejectionReason variants
        if map_len > RejectionReason::COUNT as u64 {
            return Err(DecodeError::message(
                "rejections map can only have the existing reasons for rejection",
            ));
        }

        let mut rejections = HashMap::new();
        for entry in entries.as_slice() {
            let mut key_decoder = Decoder::new(&entry.key_bytes);
            let reason = RejectionReason::decode(&mut key_decoder, ctx)?;

            let mut value_decoder = Decoder::new(&entry.value);
            let Some(arr_len) = value_decoder.array()? else {
                return Err(DecodeError::message(
                    "rejection value must be a defined-size array",
                ));
            };

            let mut doc_refs = Vec::new();
            for _ in 0..arr_len {
                let doc_ref = DocumentRef::decode(&mut value_decoder, ctx)?;
                doc_refs.push(doc_ref);
            }

            if rejections.insert(reason, doc_refs.into()).is_some() {
                return Err(DecodeError::message("Duplicate rejection reason key"));
            }
        }

        Ok(Self(rejections))
    }
}

#[cfg(test)]
mod tests {
    use catalyst_signed_doc::tests_utils::create_dummy_doc_ref;

    use super::*;

    #[test]
    fn roundtrip() {
        // Create test DocumentRef instances
        let doc_ref1 = create_dummy_doc_ref();
        let doc_ref2 = create_dummy_doc_ref();

        let mut rejections_map = HashMap::new();
        rejections_map.insert(RejectionReason::AlreadyVoted, vec![doc_ref1.clone()].into());
        rejections_map.insert(
            RejectionReason::ObsoleteVote,
            vec![doc_ref2.clone(), doc_ref1.clone()].into(),
        );

        let original = Rejections(rejections_map);

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Rejections::decode(&mut minicbor::Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
