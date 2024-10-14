//! V1 transaction objects decoding implementation.

use std::io::Read;

use anyhow::{anyhow, bail, ensure};

use super::{EncryptedVote, Tx, VotePayload, VoterProof};
use crate::{
    crypto::ed25519::{PublicKey, Signature},
    utils::{read_array, read_be_u32, read_be_u64, read_be_u8},
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

impl Tx {
    /// Write the bytes to sign for the `Tx` to provided `buf`.
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn bytes_to_sign(
        vote_plan_id: &[u8; 32], proposal_index: u8, vote: &VotePayload, public_key: &PublicKey,
        buf: &mut Vec<u8>,
    ) {
        buf.extend_from_slice(vote_plan_id);
        buf.push(proposal_index);

        match vote {
            VotePayload::Public(vote) => {
                // Public vote tag
                buf.push(1);
                buf.push(*vote);
            },
            VotePayload::Private(vote, proof) => {
                // Private vote tag
                buf.push(2);
                buf.push(vote.size() as u8);
                buf.extend_from_slice(&vote.to_bytes());

                buf.push(proof.size() as u8);
                buf.extend_from_slice(&proof.to_bytes());
            },
        }

        // Zeros block date
        buf.extend_from_slice(&[0u8; 8]);
        // Number of inputs
        buf.push(1);
        // Number of outputs
        buf.push(0);
        // Input tag
        buf.push(0xFF);
        // Zero value
        buf.extend_from_slice(&[0u8; 8]);

        buf.extend_from_slice(&public_key.to_bytes());
    }

    /// Convert this `Tx` to its underlying sequence of bytes.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_bytes(&self) -> Vec<u8> {
        // Initialize already with the padding tag `0` and fragment tag `11`.
        let mut tx_body = vec![PADDING_TAG, FRAGMENT_TAG];

        tx_body.extend_from_slice(&self.vote_plan_id);
        tx_body.push(self.proposal_index);

        match &self.vote {
            Vote::Public(vote) => {
                // Public vote tag
                tx_body.push(PUBLIC_VOTE_TAG);
                tx_body.push(*vote);
            },
            Vote::Private(vote, proof) => {
                // Private vote tag
                tx_body.push(PRIVATE_VOTE_TAG);
                tx_body.push(vote.size() as u8);
                tx_body.extend_from_slice(&vote.to_bytes());

                tx_body.push(proof.size() as u8);
                tx_body.extend_from_slice(&proof.to_bytes());
            },
        }

        // Zeros block date
        tx_body.extend_from_slice(&[0u8; 8]);
        // Number of inputs
        tx_body.push(NUMBER_OF_INPUTS);
        // Number of outputs
        tx_body.push(NUMBER_OF_OUTPUTS);
        // Input tag
        tx_body.push(INPUT_TAG);
        // Zero value
        tx_body.extend_from_slice(&[0u8; 8]);

        tx_body.extend_from_slice(&self.public_key.to_bytes());

        // Add the size of decoded bytes to the beginning.
        let mut res = (tx_body.len() as u32).to_be_bytes().to_vec();
        res.append(&mut tx_body);
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
        read_be_u32(reader)?;

        let padding_tag = read_be_u8(reader)?;
        ensure!(
            padding_tag == PADDING_TAG,
            "Invalid padding tag field value, must be equals to {PADDING_TAG}, provided: {padding_tag}.",
        );

        let fragment_tag = read_be_u8(reader)?;
        ensure!(
            fragment_tag == FRAGMENT_TAG,
            "Invalid fragment tag field value, must be equals to {FRAGMENT_TAG}, provided: {fragment_tag}.",
        );

        let vote_plan_id = read_array(reader)?;

        let proposal_index = read_be_u8(reader)?;

        let vote_tag = read_be_u8(reader)?;
        let vote = match vote_tag {
            PUBLIC_VOTE_TAG => {
                let vote = read_be_u8(reader)?;
                Vote::Public(vote)
            },
            PRIVATE_VOTE_TAG => {
                let size = read_be_u8(reader)?;
                let vote = EncryptedVote::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid encrypted vote, error: {e}."))?;

                let size = read_be_u8(reader)?;
                let proof = VoterProof::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid voter proof, error: {e}."))?;

                Vote::Private(vote, proof)
            },
            tag => {
                bail!(
                    "Invalid vote tag value, must be equals to {PUBLIC_VOTE_TAG} or {PRIVATE_VOTE_TAG}, provided: {tag}"
                )
            },
        };

        // skip block date (epoch and slot)
        read_be_u64(reader)?;

        let inputs_amount = read_be_u8(reader)?;
        ensure!(
            inputs_amount == NUMBER_OF_INPUTS,
            "Invalid number of inputs, expected: {NUMBER_OF_INPUTS}, provided: {inputs_amount}",
        );

        let outputs_amount = read_be_u8(reader)?;
        ensure!(
            outputs_amount == NUMBER_OF_OUTPUTS,
            "Invalid number of outputs, expected: {NUMBER_OF_OUTPUTS}, provided: {outputs_amount}",
        );

        let input_tag = read_be_u8(reader)?;
        ensure!(
            input_tag == INPUT_TAG,
            "Invalid input tag, expected: {INPUT_TAG}, provided: {input_tag}",
        );

        // skip value
        read_be_u64(reader)?;

        let public_key_bytes = read_array(reader)?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| anyhow!("Invalid public key, error: {e}."))?;

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
    use proptest::prelude::{any, any_with, Arbitrary, BoxedStrategy, Strategy};
    use test_strategy::proptest;

    use super::*;
    use crate::crypto::ed25519::PrivateKey;

    impl Arbitrary for Tx {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<(
                [u8; 32],
                u8,
                VotePayload,
                PrivateKey,
                [u8; Signature::BYTES_SIZE],
            )>()
            .prop_map(
                |(vote_plan_id, proposal_index, vote, sk, signature_bytes)| {
                    Tx {
                        vote_plan_id,
                        proposal_index,
                        vote,
                        public_key: sk.public_key(),
                        signature: Signature::from_bytes(&signature_bytes),
                    }
                },
            )
            .boxed()
        }
    }

    impl Arbitrary for VotePayload {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<bool>()
                .prop_flat_map(|b| {
                    if b {
                        any::<u8>().prop_map(VotePayload::Public).boxed()
                    } else {
                        any::<(u8, u8)>()
                            .prop_flat_map(|(s1, s2)| {
                                any_with::<(EncryptedVote, VoterProof)>((s1.into(), s2.into()))
                                    .prop_map(|(v, p)| VotePayload::Private(v, p))
                            })
                            .boxed()
                    }
                })
                .boxed()
        }
    }

    #[proptest]
    #[allow(clippy::indexing_slicing)]
    fn tx_to_bytes_from_bytes_test(t1: Tx) {
        let bytes = t1.to_bytes();

        // verify correctness serializing tx size field
        let size = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        assert_eq!(size as usize, bytes.len() - 4);

        let t2 = Tx::from_bytes(&mut bytes.as_slice()).unwrap();
        assert_eq!(t1, t2);
    }
}
