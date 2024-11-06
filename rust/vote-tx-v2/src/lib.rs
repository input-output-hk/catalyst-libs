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
    choice: ciborium::Value,
    /// `proof` field
    proof: ciborium::Value,
    /// `prop-id` field
    prop_id: ciborium::Value,
}

impl Vote {
    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote` to CBOR
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let cbor_array = ciborium::Value::Array(vec![
            ciborium::Value::Tag(ENCODED_CBOR_TAG, self.choice.clone().into()),
            ciborium::Value::Tag(ENCODED_CBOR_TAG, self.proof.clone().into()),
            ciborium::Value::Tag(ENCODED_CBOR_TAG, self.prop_id.clone().into()),
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
        let val = ciborium::Value::from_slice(bytes)
            .map_err(|_| anyhow!("Invalid `Vote` CBOR encoded bytes, not a CBOR encoded."))?;
        let array = val
            .into_array()
            .map_err(|_| anyhow!("Invalid `Vote` CBOR encoded bytes, must be an array."))?;

        ensure!(
            array.len() == 3,
            "Invalid `Vote` CBOR encoded bytes, must be an array of length 3.",
        );

        let mut iter = array.into_iter();

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, choice) = iter.next().ok_or(anyhow!(
            "Invalid `Vote` CBOR encoded bytes, missing `choice` field."
        ))?
        else {
            bail!("Invalid `Vote` CBOR encoded bytes, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, proof) = iter.next().ok_or(anyhow!(
            "Invalid `Vote` CBOR encoded bytes, missing `proof` field."
        ))?
        else {
            bail!("Invalid `Vote` CBOR encoded bytes, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };

        let ciborium::Value::Tag(ENCODED_CBOR_TAG, prop_id) = iter.next().ok_or(anyhow!(
            "Invalid `Vote` CBOR encoded bytes, missing `prod-id` field."
        ))?
        else {
            bail!("Invalid `Vote` CBOR encoded bytes, must be a major type 6 with tag {ENCODED_CBOR_TAG}.");
        };

        Ok(Self {
            choice: *choice,
            proof: *proof,
            prop_id: *prop_id,
        })
    }
}
