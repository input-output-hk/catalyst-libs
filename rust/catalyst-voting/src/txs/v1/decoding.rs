//! V1 transaction objects decoding implementation.

use std::io::Read;

use anyhow::{anyhow, bail, ensure};

use super::{EncryptedVote, PublicKey, Tx, Vote, VoterProof};

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
    pub fn from_bytes(mut bytes: &[u8]) -> anyhow::Result<Self> {
        let mut u8_buf = [0u8; 1];
        let mut u32_buf = [0u8; 4];
        let mut u64_buf = [0u8; 8];
        let mut u256_buf = [0u8; 32];

        // Read tx size
        bytes.read_exact(&mut u32_buf)?;

        bytes.read_exact(&mut u8_buf)?;
        ensure!(
            u8_buf[0] == 0,
            "Invalid padding tag field value, must be equals to `0`, provided: {0}.",
            u8_buf[0]
        );

        bytes.read_exact(&mut u8_buf)?;
        ensure!(
            u8_buf[0] == 11,
            "Invalid fragment tag field value, must be equals to `11`, provided: {0}.",
            u8_buf[0]
        );

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
                    .map_err(|e| anyhow!("Invalid encrypted vote, error: {e}."))?;
                bytes = &bytes[vote.bytes_size()..];

                bytes.read_exact(&mut u8_buf)?;
                let proof = VoterProof::from_bytes(bytes, u8_buf[0].into())
                    .map_err(|e| anyhow!("Invalid voter proof, error: {e}."))?;
                bytes = &bytes[vote.bytes_size()..];

                Vote::Private(vote, proof)
            },
            tag => bail!("Invalid vote tag value, must be equals to `0` or `1`, provided: {tag}"),
        };

        // skip block date (epoch and slot)
        bytes.read_exact(&mut u64_buf)?;

        bytes.read_exact(&mut u8_buf)?;
        ensure!(
            u8_buf[0] == 1,
            "Invalid number of inputs, expected: `1`, provided: {0}",
            u8_buf[0]
        );

        bytes.read_exact(&mut u8_buf)?;
        ensure!(
            u8_buf[0] == 0,
            "Invalid number of outputs, expected: `0`, provided: {0}",
            u8_buf[0]
        );

        bytes.read_exact(&mut u8_buf)?;
        ensure!(
            u8_buf[0] == 0xFF,
            "Invalid input tag, expected: `255`, provided: {0}",
            u8_buf[0]
        );

        // skip value
        bytes.read_exact(&mut u64_buf)?;

        bytes.read_exact(&mut u256_buf)?;
        let public_key = PublicKey::from_bytes(&u256_buf)
            .map_err(|e| anyhow!("Invalid public key, error: {e}."))?;

        Ok(Self {
            vote_plan_id,
            proposal_index,
            vote,
            public_key,
        })
    }
}
