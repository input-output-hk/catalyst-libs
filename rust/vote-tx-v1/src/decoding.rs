//! V1 transaction objects decoding implementation.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/abnf/jorm.abnf>

use std::io::Read;

use anyhow::{anyhow, bail, ensure};
use catalyst_voting::crypto::ed25519::{PublicKey, Signature};

use crate::{
    utils::{read_array, read_be_u32, read_be_u64, read_be_u8},
    EncryptedVote, Tx, VotePayload, VoterProof,
};

/// Jörmungandr tx fragment tag.
const FRAGMENT_TAG: u8 = 11;
/// Jörmungandr tx input tag.
const INPUT_TAG: u8 = 0xFF;
/// Jörmungandr tx number of inputs.
const NUMBER_OF_INPUTS: u8 = 1;
/// Jörmungandr tx number of outputs.
const NUMBER_OF_OUTPUTS: u8 = 0;
/// Jörmungandr tx padding tag.
const PADDING_TAG: u8 = 0;
/// Jörmungandr tx private vote tag.
const PRIVATE_VOTE_TAG: u8 = 2;
/// Jörmungandr tx public vote tag.
const PUBLIC_VOTE_TAG: u8 = 1;
/// Jörmungandr tx witness tag.
const WITNESS_TAG: u8 = 2;

impl Tx {
    /// Write the bytes of the `Tx` body to provided `buf`.
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn tx_body_decode(
        vote_plan_id: &[u8; 32],
        proposal_index: u8,
        vote: &VotePayload,
        public_key: &PublicKey,
        buf: &mut Vec<u8>,
    ) {
        buf.extend_from_slice(vote_plan_id);
        buf.push(proposal_index);

        match vote {
            VotePayload::Public(vote) => {
                // Public vote tag
                buf.push(PUBLIC_VOTE_TAG);
                buf.push(*vote);
            },
            VotePayload::Private(vote, proof) => {
                // Private vote tag
                buf.push(PRIVATE_VOTE_TAG);
                buf.push(vote.size() as u8);
                buf.extend_from_slice(&vote.to_bytes());

                buf.push(proof.size() as u8);
                buf.extend_from_slice(&proof.to_bytes());
            },
        }

        // Zeros block date
        buf.extend_from_slice(&[0u8; 8]);
        // Number of inputs
        buf.push(NUMBER_OF_INPUTS);
        // Number of outputs
        buf.push(NUMBER_OF_OUTPUTS);
        // Input tag
        buf.push(INPUT_TAG);
        // Zero value
        buf.extend_from_slice(&[0u8; 8]);

        buf.extend_from_slice(&public_key.to_bytes());
    }

    /// Convert this `Tx` to its underlying sequence of bytes.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_bytes(&self) -> Vec<u8> {
        // Initialize already with the padding tag `0` and fragment tag `11`.
        let mut buf = vec![PADDING_TAG, FRAGMENT_TAG];

        Self::tx_body_decode(
            &self.vote_plan_id,
            self.proposal_index,
            &self.vote,
            &self.public_key,
            &mut buf,
        );

        // Witness tag
        buf.push(WITNESS_TAG);
        // Zero nonce
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&self.signature.to_bytes());

        // Add the size of decoded bytes to the beginning.
        let mut res = (buf.len() as u32).to_be_bytes().to_vec();
        res.append(&mut buf);
        res
    }

    /// Attempt to construct a `Tx` from a byte representation.
    ///
    /// # Errors
    ///   - Invalid padding tag field value.
    ///   - Invalid fragment tag field value.
    ///   - Invalid encrypted vote.
    ///   - Invalid voter proof.
    ///   - Invalid vote tag value.
    ///   - Invalid public key.
    #[allow(clippy::indexing_slicing)]
    pub fn from_bytes<R: Read>(reader: &mut R) -> anyhow::Result<Self> {
        // Skip tx size field
        read_be_u32(reader).map_err(|_| anyhow!("Missing tx size field."))?;

        let padding_tag = read_be_u8(reader).map_err(|_| anyhow!("Missing padding tag field."))?;
        ensure!(
            padding_tag == PADDING_TAG,
            "Invalid padding tag field value, must be equals to {PADDING_TAG}, \
            provided: {padding_tag}.",
        );

        let fragment_tag =
            read_be_u8(reader).map_err(|_| anyhow!("Missing fragment tag field."))?;
        ensure!(
            fragment_tag == FRAGMENT_TAG,
            "Invalid fragment tag field value, must be equals to {FRAGMENT_TAG}, \
            provided: {fragment_tag}.",
        );

        let vote_plan_id =
            read_array(reader).map_err(|_| anyhow!("Missing vote plan id field."))?;

        let proposal_index =
            read_be_u8(reader).map_err(|_| anyhow!("Missing proposal index field."))?;

        let vote_tag = read_be_u8(reader).map_err(|_| anyhow!("Missing vote tag field."))?;
        let vote = match vote_tag {
            PUBLIC_VOTE_TAG => {
                let vote =
                    read_be_u8(reader).map_err(|_| anyhow!("Missing public vote choice field."))?;
                VotePayload::Public(vote)
            },
            PRIVATE_VOTE_TAG => {
                let size = read_be_u8(reader).map_err(|_| anyhow!("Missing vote size field."))?;
                let vote = EncryptedVote::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid encrypted vote, error: {e}."))?;

                let size = read_be_u8(reader).map_err(|_| anyhow!("Missing proof size field."))?;
                let proof = VoterProof::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid voter proof, error: {e}."))?;

                VotePayload::Private(vote, proof)
            },
            tag => {
                bail!(
                    "Invalid vote tag value, \
                    must be equals to {PUBLIC_VOTE_TAG} or {PRIVATE_VOTE_TAG}, provided: {tag}"
                )
            },
        };

        // skip block date (epoch and slot)
        read_be_u64(reader).map_err(|_| anyhow!("Missing block date field."))?;

        let inputs_amount =
            read_be_u8(reader).map_err(|_| anyhow!("Missing inputs amount field."))?;
        ensure!(
            inputs_amount == NUMBER_OF_INPUTS,
            "Invalid number of inputs, expected: {NUMBER_OF_INPUTS}, \
            provided: {inputs_amount}",
        );

        let outputs_amount =
            read_be_u8(reader).map_err(|_| anyhow!("Missing outputs amount field."))?;
        ensure!(
            outputs_amount == NUMBER_OF_OUTPUTS,
            "Invalid number of outputs, expected: {NUMBER_OF_OUTPUTS}, \
            provided: {outputs_amount}",
        );

        let input_tag = read_be_u8(reader).map_err(|_| anyhow!("Missing input tag field."))?;
        ensure!(
            input_tag == INPUT_TAG,
            "Invalid input tag, expected: {INPUT_TAG}, \
            provided: {input_tag}",
        );

        // skip value
        read_be_u64(reader).map_err(|_| anyhow!("Missing value field."))?;

        let public_key_bytes =
            read_array(reader).map_err(|_| anyhow!("Missing public_key field."))?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| anyhow!("Invalid public key, error: {e}."))?;

        let witness_tag = read_be_u8(reader).map_err(|_| anyhow!("Missing witness tag field."))?;
        ensure!(
            witness_tag == WITNESS_TAG,
            "Invalid witness tag, expected: {WITNESS_TAG}, \
            provided: {witness_tag}",
        );

        // Skip nonce field
        read_be_u32(reader).map_err(|_| anyhow!("Missing nonce field."))?;

        let signature_bytes =
            read_array(reader).map_err(|_| anyhow!("Missing signature field."))?;
        let signature = Signature::from_bytes(&signature_bytes);

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::explicit_deref_methods)]

    use catalyst_voting::{
        crypto::{ed25519::PrivateKey, rng::rand_core::OsRng},
        vote_protocol::committee::ElectionSecretKey,
    };
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn tx_public_to_bytes_from_bytes_test(
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        #[strategy(1u8..5)] voting_options: u8,
        #[strategy(0..#voting_options)] choice: u8,
    ) {
        let mut rng = OsRng;
        let users_private_key = PrivateKey::random(&mut rng);

        let t1 = Tx::new_public(
            vote_plan_id,
            proposal_index,
            voting_options,
            choice,
            &users_private_key,
        )
        .unwrap();

        let bytes = t1.to_bytes();

        let mut reader = bytes.as_slice();

        let size = read_be_u32(&mut reader).unwrap();
        assert_eq!(size as usize, bytes.len().checked_sub(4).unwrap());

        let padding_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(padding_tag, PADDING_TAG);

        let fragment_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(fragment_tag, FRAGMENT_TAG);

        let vote_plan_id = read_array(&mut reader).unwrap();
        assert_eq!(vote_plan_id, t1.vote_plan_id);

        let proposal_index = read_be_u8(&mut reader).unwrap();
        assert_eq!(proposal_index, t1.proposal_index);

        let vote_tag = read_be_u8(&mut reader).unwrap();
        assert!(vote_tag == PUBLIC_VOTE_TAG);

        let vote = read_be_u8(&mut reader).unwrap();
        assert_eq!(VotePayload::Public(vote), t1.vote);

        let block_date = read_be_u64(&mut reader).unwrap();
        assert_eq!(block_date, 0);

        let inputs_amount = read_be_u8(&mut reader).unwrap();
        assert_eq!(inputs_amount, NUMBER_OF_INPUTS);

        let outputs_amount = read_be_u8(&mut reader).unwrap();
        assert_eq!(outputs_amount, NUMBER_OF_OUTPUTS);

        let input_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(input_tag, INPUT_TAG);

        let value = read_be_u64(&mut reader).unwrap();
        assert_eq!(value, 0);

        let public_key = read_array(&mut reader).unwrap();
        assert_eq!(PublicKey::from_bytes(&public_key).unwrap(), t1.public_key);

        let witness_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(witness_tag, WITNESS_TAG);

        let nonce = read_be_u32(&mut reader).unwrap();
        assert_eq!(nonce, 0);

        let signature = read_array(&mut reader).unwrap();
        assert_eq!(Signature::from_bytes(&signature), t1.signature);

        let t2 = Tx::from_bytes(&mut bytes.as_slice()).unwrap();
        assert_eq!(t1, t2);
    }

    #[proptest]
    fn tx_private_to_bytes_from_bytes_test(
        vote_plan_id: [u8; 32],
        proposal_index: u8,
        #[strategy(1u8..5)] voting_options: u8,
        #[strategy(0..#voting_options)] choice: u8,
    ) {
        let mut rng = OsRng;
        let users_private_key = PrivateKey::random(&mut rng);
        let election_secret_key = ElectionSecretKey::random(&mut rng);
        let election_public_key = election_secret_key.public_key();

        let t1 = Tx::new_private(
            vote_plan_id,
            proposal_index,
            voting_options,
            choice,
            &election_public_key,
            &users_private_key,
            &mut rng,
        )
        .unwrap();

        let bytes = t1.to_bytes();

        let mut reader = bytes.as_slice();

        let size = read_be_u32(&mut reader).unwrap();
        assert_eq!(size as usize, bytes.len().checked_sub(4).unwrap());

        let padding_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(padding_tag, PADDING_TAG);

        let fragment_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(fragment_tag, FRAGMENT_TAG);

        let vote_plan_id = read_array(&mut reader).unwrap();
        assert_eq!(vote_plan_id, t1.vote_plan_id);

        let proposal_index = read_be_u8(&mut reader).unwrap();
        assert_eq!(proposal_index, t1.proposal_index);

        let vote_tag = read_be_u8(&mut reader).unwrap();
        assert!(vote_tag == PRIVATE_VOTE_TAG);
        let size = read_be_u8(&mut reader).unwrap();
        let vote = EncryptedVote::from_bytes(&mut reader, size.into()).unwrap();
        let size = read_be_u8(&mut reader).unwrap();
        let proof = VoterProof::from_bytes(&mut reader, size.into()).unwrap();
        assert_eq!(VotePayload::Private(vote, proof), t1.vote);

        let block_date = read_be_u64(&mut reader).unwrap();
        assert_eq!(block_date, 0);

        let inputs_amount = read_be_u8(&mut reader).unwrap();
        assert_eq!(inputs_amount, NUMBER_OF_INPUTS);

        let outputs_amount = read_be_u8(&mut reader).unwrap();
        assert_eq!(outputs_amount, NUMBER_OF_OUTPUTS);

        let input_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(input_tag, INPUT_TAG);

        let value = read_be_u64(&mut reader).unwrap();
        assert_eq!(value, 0);

        let public_key = read_array(&mut reader).unwrap();
        assert_eq!(PublicKey::from_bytes(&public_key).unwrap(), t1.public_key);

        let witness_tag = read_be_u8(&mut reader).unwrap();
        assert_eq!(witness_tag, WITNESS_TAG);

        let nonce = read_be_u32(&mut reader).unwrap();
        assert_eq!(nonce, 0);

        let signature = read_array(&mut reader).unwrap();
        assert_eq!(Signature::from_bytes(&signature), t1.signature);

        let t2 = Tx::from_bytes(&mut bytes.as_slice()).unwrap();
        assert_eq!(t1, t2);
    }
}
