//! Rejection Reason for a referenced ballot.

use minicbor::{Decode, Encode};
use strum::EnumCount;

/// String value for the already voted.
const ALREADY_VOTED: &str = "already-voted";
/// String value for obsolete vote.
const OBSOLETE_VOTE: &str = "obsolete-vote";

/// Reason for rejecting a ballot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount)]
pub enum RejectionReason {
    /// The user has already voted
    AlreadyVoted,
    /// The vote is obsolete.
    ObsoleteVote,
}

impl Encode<()> for RejectionReason {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let reason_str = match self {
            Self::AlreadyVoted => ALREADY_VOTED,
            Self::ObsoleteVote => OBSOLETE_VOTE,
        };
        e.str(reason_str)?;
        Ok(())
    }
}

impl Decode<'_, ()> for RejectionReason {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let reason_str = d.str()?;
        match reason_str {
            ALREADY_VOTED => Ok(Self::AlreadyVoted),
            OBSOLETE_VOTE => Ok(Self::ObsoleteVote),
            _ => {
                Err(minicbor::decode::Error::message(format!(
                    "Invalid rejection reason: {reason_str}. Expected '{ALREADY_VOTED}' or '{OBSOLETE_VOTE}'",
                )))
            },
        }
    }
}
