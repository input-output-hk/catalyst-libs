//! A Catalyst vote transaction v1 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/)

use anyhow::{anyhow, bail, ensure};
use coset::CborSerializable;

/// `encoded-cbor` tag number
const ENCODED_CBOR_TAG: u64 = 24;

/// A generalised vote transaction struct.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct GeneralisedTx {
    /// `tx-body` field
    tx_body: GeneralisedTxBody,
}

/// A generalised vote transaction body struct.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct GeneralisedTxBody {
    /// `vote-type` field
    vote_type: Vec<u8>,
    /// `event` field
    event: Vec<(ciborium::Value, ciborium::Value)>,
    /// `votes` field
    votes: Vec<Vote>,
    /// `voters-data` field
    voters_data: Vec<u8>,
}

/// A vote struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Vote {
    /// `choice` field
    choice: Vec<u8>,
    /// `proof` field
    proof: Vec<u8>,
    /// `prop-id` field
    prop_id: Vec<u8>,
}

impl Vote {
    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote` to CBOR
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let cbor_array = ciborium::Value::Array(vec![
            ciborium::Value::Tag(
                ENCODED_CBOR_TAG,
                ciborium::Value::Bytes(self.choice.clone()).into(),
            ),
            ciborium::Value::Tag(
                ENCODED_CBOR_TAG,
                ciborium::Value::Bytes(self.proof.clone()).into(),
            ),
            ciborium::Value::Tag(
                ENCODED_CBOR_TAG,
                ciborium::Value::Bytes(self.prop_id.clone()).into(),
            ),
        ]);

        cbor_array
            .to_vec()
            .map_err(|_| anyhow!("Cannot encode `Vote` to CBOR"))
    }

    /// Decodes `Vote` from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Invalid `Vote` CBOR encoded bytes.
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        /// Invalid `Vote` CBOR encoded bytes error msg.
        const INVALID_CBOR_BYTES_ERR: &str = "Invalid `Vote` CBOR encoded bytes";

        let val = ciborium::Value::from_slice(bytes)
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, not a CBOR encoded."))?;
        let array = val
            .into_array()
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, must be array type."))?;

        ensure!(
            array.len() == 3,
            "{INVALID_CBOR_BYTES_ERR}, must be an array of length 3.",
        );

        let mut iter = array.into_iter();

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, choice) = iter
            .next()
            .ok_or(anyhow!("{INVALID_CBOR_BYTES_ERR}, missing `choice` field."))?
        else {
            bail!("{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };
        let choice = choice
            .into_bytes()
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `choice` must be bytes type."))?;

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, proof) = iter
            .next()
            .ok_or(anyhow!("{INVALID_CBOR_BYTES_ERR}, missing `proof` field."))?
        else {
            bail!("{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };
        let proof = proof
            .into_bytes()
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `proof` must be bytes type."))?;

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, prop_id) = iter.next().ok_or(anyhow!(
            "{INVALID_CBOR_BYTES_ERR}, missing `prod-id` field."
        ))?
        else {
            bail!("{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };
        let prop_id = prop_id
            .into_bytes()
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `prop-id` must be bytes type."))?;

        Ok(Self {
            choice,
            proof,
            prop_id,
        })
    }
}
