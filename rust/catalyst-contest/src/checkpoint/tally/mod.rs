//! Tally related code
pub(super) mod encrypted;

mod clear_choice;
mod proposal_result;
mod voting_power;

use std::collections::HashMap;

use catalyst_signed_doc::DocumentRef;
pub use clear_choice::ClearChoice;
use minicbor::{Decode, Encode};
pub use proposal_result::ProposalResult;
pub use voting_power::VotingPower;

/// Tally map of `document_ref => tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tally(HashMap<DocumentRef, ProposalResult>);

impl Encode<()> for Tally {
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

impl Decode<'_, ()> for Tally {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let Some(map_len) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "tally must be a defined-size map",
            ));
        };

        let mut tally = HashMap::new();
        for _ in 0..map_len {
            let doc_ref = DocumentRef::decode(d, ctx)?;
            let proposal_result = ProposalResult::decode(d, ctx)?;

            tally.insert(doc_ref, proposal_result);
        }

        Ok(Self(tally))
    }
}
