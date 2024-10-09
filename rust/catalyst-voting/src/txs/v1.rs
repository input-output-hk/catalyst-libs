//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

#![allow(unused_variables, dead_code)]

use std::io::Read;

use crate::vote_protocol::voter::{DecodingError as EncryptedVoteDecodingError, EncryptedVote};

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
    Private(EncryptedVote),
}

/// `Tx` decoding error
#[derive(thiserror::Error, Debug)]
pub enum DecodingError {
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
    /// Cannot decode ciphertexts array size
    #[error("Cannot decode encrypted vote size field.")]
    CannotDecodeEncryptedVoteSize,
    /// Cannot decode encrypted vote
    #[error(transparent)]
    CannotDecodeEncryptedVote(#[from] EncryptedVoteDecodingError),
}

impl Tx {
    /// Decode `Tx` from bytes.
    ///
    /// # Errors
    ///   - `DecodingError`
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, DecodingError> {
        let mut u32_buf = [0u8; 4];
        let mut u8_buf = [0u8; 1];
        let mut u256_buf = [0u8; 32];
        // let mut u512_buf = [0u8; 64];

        bytes
            .read_exact(&mut u32_buf)
            .map_err(|_| DecodingError::CannotDecodeTxSize)?;
        let tx_size = u32::from_be_bytes(u32_buf);

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| DecodingError::CannotDecodePaddingTag)?;
        if u8_buf[0] != 0 {
            return Err(DecodingError::InvalidPaddingTag(u8_buf[0]));
        }

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| DecodingError::CannotDecodeFragmentTag)?;
        if u8_buf[0] != 11 {
            return Err(DecodingError::InvalidFragmentTag(u8_buf[0]));
        }

        bytes
            .read_exact(&mut u256_buf)
            .map_err(|_| DecodingError::CannotDecodeVotePlanId)?;
        let vote_plan_id = u256_buf;

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| DecodingError::CannotDecodeProposalIndex)?;
        let proposal_index = u8_buf[0];

        bytes
            .read_exact(&mut u8_buf)
            .map_err(|_| DecodingError::CannotDecodeVoteTag)?;
        let vote = match u8_buf[0] {
            1 => {
                bytes
                    .read_exact(&mut u8_buf)
                    .map_err(|_| DecodingError::CannotDecodePublicVote)?;
                Vote::Public(u8_buf[0])
            },
            2 => {
                bytes
                    .read_exact(&mut u8_buf)
                    .map_err(|_| DecodingError::CannotDecodeEncryptedVoteSize)?;
                let encrypted_vote = EncryptedVote::from_bytes(bytes, u8_buf[0].into())?;

                Vote::Private(encrypted_vote)
            },
            tag => return Err(DecodingError::InvalidVoteTag(tag)),
        };

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
        })
    }
}
