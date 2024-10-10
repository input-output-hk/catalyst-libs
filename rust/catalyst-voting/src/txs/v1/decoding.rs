//! V1 transaction objects decoding implementation.

use std::io::Read;

use anyhow::{anyhow, bail, ensure};

use super::{EncryptedVote, PublicKey, Tx, Vote, VoterProof};
use crate::utils::{read_array, read_be_u32, read_be_u64, read_be_u8};

impl Tx {
    /// Convert this `Tx` to its underlying sequence of bytes.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_bytes(&self) -> Vec<u8> {
        // Initialize already with the padding tag `0` and fragment tag `11`.
        let mut tx_body = vec![0, 11];

        tx_body.extend_from_slice(&self.vote_plan_id);
        tx_body.push(self.proposal_index);

        match &self.vote {
            Vote::Public(vote) => {
                // Public vote tag
                tx_body.push(1);
                tx_body.push(*vote);
            },
            Vote::Private(vote, proof) => {
                // Private vote tag
                tx_body.push(2);
                tx_body.push(vote.size() as u8);
                tx_body.extend_from_slice(&vote.to_bytes());

                tx_body.push(proof.size() as u8);
                tx_body.extend_from_slice(&proof.to_bytes());
            },
        }

        // Zeros block date
        tx_body.extend_from_slice(&[0u8; 8]);
        // Number of inputs
        tx_body.push(1);
        // Number of outputs
        tx_body.push(0);
        // Input tag
        tx_body.push(0xFF);
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
            padding_tag == 0,
            "Invalid padding tag field value, must be equals to `0`, provided: {padding_tag}.",
        );

        let fragment_tag = read_be_u8(reader)?;
        ensure!(
            fragment_tag == 11,
            "Invalid fragment tag field value, must be equals to `11`, provided: {fragment_tag}.",
        );

        let vote_plan_id = read_array(reader)?;

        let proposal_index = read_be_u8(reader)?;

        let vote_tag = read_be_u8(reader)?;
        let vote = match vote_tag {
            1 => {
                let vote = read_be_u8(reader)?;
                Vote::Public(vote)
            },
            2 => {
                let size = read_be_u8(reader)?;
                let vote = EncryptedVote::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid encrypted vote, error: {e}."))?;

                let size = read_be_u8(reader)?;
                let proof = VoterProof::from_bytes(reader, size.into())
                    .map_err(|e| anyhow!("Invalid voter proof, error: {e}."))?;

                Vote::Private(vote, proof)
            },
            tag => bail!("Invalid vote tag value, must be equals to `0` or `1`, provided: {tag}"),
        };

        // skip block date (epoch and slot)
        read_be_u64(reader)?;

        let inputs_amount = read_be_u8(reader)?;
        ensure!(
            inputs_amount == 1,
            "Invalid number of inputs, expected: `1`, provided: {inputs_amount}",
        );

        let outputs_amount = read_be_u8(reader)?;
        ensure!(
            outputs_amount == 0,
            "Invalid number of outputs, expected: `0`, provided: {outputs_amount}",
        );

        let input_tag = read_be_u8(reader)?;
        ensure!(
            input_tag == 0xFF,
            "Invalid input tag, expected: `255`, provided: {input_tag}",
        );

        // skip value
        read_be_u64(reader)?;

        let public_key_bytes = read_array(reader)?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| anyhow!("Invalid public key, error: {e}."))?;

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{any, any_with, Arbitrary, BoxedStrategy, Strategy};
    use test_strategy::proptest;

    use super::*;
    use crate::SecretKey;

    impl Arbitrary for Tx {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<([u8; 32], u8, Vote, SecretKey)>()
                .prop_map(|(vote_plan_id, proposal_index, vote, s)| {
                    Tx {
                        vote_plan_id,
                        proposal_index,
                        vote,
                        public_key: s.public_key(),
                    }
                })
                .boxed()
        }
    }

    impl Arbitrary for Vote {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any::<bool>()
                .prop_flat_map(|b| {
                    if b {
                        any::<u8>().prop_map(Vote::Public).boxed()
                    } else {
                        any::<(u8, u8)>()
                            .prop_flat_map(|(s1, s2)| {
                                any_with::<(EncryptedVote, VoterProof)>((s1.into(), s2.into()))
                                    .prop_map(|(v, p)| Vote::Private(v, p))
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
