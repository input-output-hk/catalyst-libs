//! Encrypted Tally related code.
mod proposal_result;

use std::collections::HashMap;

use catalyst_signed_doc::DocumentRef;
use minicbor::{Decode, Encode};
pub use proposal_result::EncryptedTallyProposalResult;

/// Placeholder map of `document_ref => encrypted-tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::zero_sized_map_values)]
pub struct EncryptedTally(HashMap<DocumentRef, EncryptedTallyProposalResult>);

impl Encode<()> for EncryptedTally {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        for (doc_ref, proposal_result) in &self.0 {
            doc_ref.encode(e, ctx)?;
            proposal_result.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for EncryptedTally {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let Some(map_len) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "encrypted-tally must be a defined-size map",
            ));
        };

        let mut tally = HashMap::new();
        for _ in 0..map_len {
            let doc_ref = DocumentRef::decode(d, ctx)?;
            let proposal_result = EncryptedTallyProposalResult::decode(d, ctx)?;
            tally.insert(doc_ref, proposal_result);
        }

        Ok(Self(tally))
    }
}
