//! A Jörmungandr transaction object structured following this [spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/voting_transaction/transaction/#v1-jormungandr)

#![allow(unused_variables, dead_code)]

use std::io::Read;

use crate::vote_protocol::voter::{proof::VoterProof, EncryptedVote};

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
    Private(EncryptedVote, VoterProof),
}

/// `Tx` decoding error
#[derive(thiserror::Error, Debug)]
pub enum DecodingError {
    /// `std::io::Error`
    #[error(transparent)]
    IoRead(#[from] std::io::Error),
    /// Invalid padding tag
    #[error("Invalid padding tag field value, must be equals to `0`, provided: {0}.")]
    InvalidPaddingTag(u8),
    /// Invalid fragment tag
    #[error("Invalid fragment tag field value, must be equals to `11`, provided: {0}.")]
    InvalidFragmentTag(u8),
    /// Cannot decode vote tag
    #[error("Invalid vote tag value, must be equals to `0` or `1`, provided: {0}")]
    InvalidVoteTag(u8),
    /// Cannot decode encrypted vote
    #[error("Cannot decode ecnrypted vote field.")]
    CannotDecodeEncryptedVote,
    /// Cannot decode voter proof field
    #[error("Cannot decode voter proof field.")]
    CannotDecodeVoterProof,
    /// Invalid number of inputs
    #[error("Invalid number of inputs, expected: `1`, provided: {0}")]
    InvalidNumberOfInputs(u8),
    /// Invalid number of outputs
    #[error("Invalid number of outputs, expected: `0`, provided: {0}")]
    InvalidNumberOfOutputs(u8),
    /// Invalid input tag
    #[error("Invalid input tag, expected: `255`, provided: {0}")]
    InvalidInputTag(u8),
}

impl Tx {
    /// Decode `Tx` from bytes.
    ///
    /// # Errors
    ///   - `DecodingError`
    #[allow(clippy::indexing_slicing)]
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, DecodingError> {
        let mut u8_buf = [0u8; 1];
        let mut u32_buf = [0u8; 4];
        let mut u64_buf = [0u8; 8];
        let mut u256_buf = [0u8; 32];
        // let mut u512_buf = [0u8; 64];

        bytes.read_exact(&mut u32_buf)?;
        let tx_size = u32::from_be_bytes(u32_buf);

        bytes.read_exact(&mut u8_buf)?;
        if u8_buf[0] != 0 {
            return Err(DecodingError::InvalidPaddingTag(u8_buf[0]));
        }

        bytes.read_exact(&mut u8_buf)?;
        if u8_buf[0] != 11 {
            return Err(DecodingError::InvalidFragmentTag(u8_buf[0]));
        }

        bytes.read_exact(&mut u256_buf)?;
        let vote_plan_id = u256_buf;

        bytes.read_exact(&mut u8_buf)?;
        let proposal_index = u8_buf[0];

        bytes.read_exact(&mut u8_buf)?;
        let vote = match u8_buf[0] {
            1 => {
                bytes.read_exact(&mut u8_buf)?;
                Vote::Public(u8_buf[0])
            },
            2 => {
                bytes.read_exact(&mut u8_buf)?;
                let vote = EncryptedVote::from_bytes(bytes, u8_buf[0].into())
                    .ok_or(DecodingError::CannotDecodeEncryptedVote)?;
                bytes = &bytes[vote.bytes_size()..];

                bytes.read_exact(&mut u8_buf)?;
                let proof = VoterProof::from_bytes(bytes, u8_buf[0].into())
                    .ok_or(DecodingError::CannotDecodeVoterProof)?;
                bytes = &bytes[vote.bytes_size()..];

                Vote::Private(vote, proof)
            },
            tag => return Err(DecodingError::InvalidVoteTag(tag)),
        };

        // skip block date (epoch and slot)
        bytes.read_exact(&mut u64_buf)?;

        bytes.read_exact(&mut u8_buf)?;
        if u8_buf[0] != 1 {
            return Err(DecodingError::InvalidNumberOfInputs(u8_buf[0]));
        }
        bytes.read_exact(&mut u8_buf)?;
        if u8_buf[0] != 0 {
            return Err(DecodingError::InvalidNumberOfOutputs(u8_buf[0]));
        }

        bytes.read_exact(&mut u8_buf)?;
        if u8_buf[0] != 0xFF {
            return Err(DecodingError::InvalidInputTag(u8_buf[0]));
        }

        // skip value
        bytes.read_exact(&mut u64_buf)?;

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
        })
    }
}
