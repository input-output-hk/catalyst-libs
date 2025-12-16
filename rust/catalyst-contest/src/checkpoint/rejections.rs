//! A map for rejected ballots by reason.

use std::collections::HashMap;

use catalyst_signed_doc::{DocumentRef, DocumentRefs};
use minicbor::{Decode, Encode};
use strum::EnumCount;

use crate::checkpoint::RejectionReason;

/// A Map for Rejected Contest Ballots by Reason.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Rejections(pub(crate) HashMap<RejectionReason, DocumentRefs>);

impl Encode<()> for Rejections {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        for (reason, doc_refs) in &self.0 {
            reason.encode(e, ctx)?;
            doc_refs.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for Rejections {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let Some(map_len) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "rejections must be a defined-size map",
            ));
        };

        // Limit map size to the number of RejectionReason variants
        if map_len > RejectionReason::COUNT as u64 {
            return Err(minicbor::decode::Error::message(
                "rejections map can only have the existing reasons for rejection",
            ));
        }

        let mut rejections = HashMap::new();
        for _ in 0..map_len {
            let reason = RejectionReason::decode(d, ctx)?;

            // According to CDDL: rejection-reason => [ + document_ref ]
            // However, the struct uses String as placeholder
            // For now, decode as array and serialize to String as placeholder
            let Some(arr_len) = d.array()? else {
                return Err(minicbor::decode::Error::message(
                    "rejection value must be a defined-size array",
                ));
            };

            let mut doc_refs = Vec::new();
            for _ in 0..arr_len {
                let doc_ref = DocumentRef::decode(d, ctx)?;
                doc_refs.push(doc_ref);
            }

            rejections.insert(reason, doc_refs.into());
        }

        Ok(Self(rejections))
    }
}
