//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

#![allow(unused_variables, dead_code)]

use std::io::Read;

/// A v1 (Jörmungandr) transaction struct
pub struct Tx {
    /// Vote plan id
    vote_plan_id: [u8; 32],
    /// Proposal index
    proposal_index: u8,
    /// Vote
    vote: Vote,
}

/// Vote struct
pub enum Vote {
    /// Public voting choice
    Public(u8),
    /// Private (encrypted) voting choice
    Private,
}

/// `V1Tx` decoding error
#[derive(thiserror::Error, Debug)]
pub enum TxDecodingError {
    /// Cannot decode tx size
    #[error("Cannot decode `u32` tx size field.")]
    CannotDecodeTxSize,
    /// Cannot decode padding tag
    #[error("Cannot decode `u8` padding tag field.")]
    CannotDecodePaddingTag,
    /// Invalid padding tag
    #[error("Invalid padding tag field value, must be equals to `0`, provided: {0}.")]
    InvalidPaddingTag(u8),
    /// Cannot decode fragment tag
    #[error("Cannot decode `u8` fragment tag field.")]
    CannotDecodeFragmentTag,
    /// Invalid fragment tag
    #[error("Invalid fragment tag field value, must be equals to `11`, provided: {0}.")]
    InvalidFragmentTag(u8),
    /// Cannot decode vote plan id
    #[error("Cannot decode vote plan id field.")]
    CannotDecodeVotePlanId,
    /// Cannot decode proposal index
    #[error("Cannot decode proposal index field.")]
    CannotDecodeProposalIndex,
    /// Cannot decode vote tag
    #[error("Cannot decode vote tag field.")]
    CannotDecodeVoteTag,
    /// Cannot decode vote tag
    #[error("Invalid vote tag value, must be equals to `0` or `1`, provided: {0}")]
    InvalidVoteTag(u8),
    /// Cannot decode public vote
    #[error("Cannot decode public vote field.")]
    CannotDecodePublicVote,
}

impl Tx {
    /// Decode `V1Tx` from bytes.
    ///
    /// # Errors
    ///   - `TxDecodingError`
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, TxDecodingError> {
        let mut u32_buf = [0u8; 4];
        let mut u8_buf = [0u8; 1];
        let mut u256_buf = [0u8; 32];

        bytes
            .read_exact(&mut u32_buf)
            .map_err(|_| TxDecodingError::CannotDecodeTxSize)?;
        let tx_size = u32::from_be_bytes(u32_buf);

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| TxDecodingError::CannotDecodePaddingTag)?;
        if u8_buf[0] != 0 {
            return Err(TxDecodingError::InvalidPaddingTag(u8_buf[0]));
        }

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| TxDecodingError::CannotDecodeFragmentTag)?;
        if u8_buf[0] != 11 {
            return Err(TxDecodingError::InvalidFragmentTag(u8_buf[0]));
        }

        bytes
            .read_exact(&mut u256_buf)
            .map_err(|_| TxDecodingError::CannotDecodeVotePlanId)?;
        let vote_plan_id = u256_buf;

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| TxDecodingError::CannotDecodeProposalIndex)?;
        let proposal_index = u8_buf[0];

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| TxDecodingError::CannotDecodeVoteTag)?;
        let vote = match u8_buf[0] {
            1 => {
                bytes
                    .read_exact(&mut u8_buf)
                    .map_err(|_| TxDecodingError::CannotDecodePublicVote)?;
                Vote::Public(u8_buf[0])
            },
            2 => Vote::Private,
            tag => return Err(TxDecodingError::InvalidVoteTag(tag)),
        };

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
        })
    }
}
