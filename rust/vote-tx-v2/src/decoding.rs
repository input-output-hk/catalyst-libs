//! CBOR encoding and decoding implementation.

use anyhow::{anyhow, bail, ensure};

use crate::Vote;

/// `encoded-cbor` tag number
const ENCODED_CBOR_TAG: u64 = 24;

impl Vote {
    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote` to CBOR
    pub fn write_to_bytes(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        let cbor_array = ciborium::Value::Array(vec![
            ciborium::Value::Tag(
                ENCODED_CBOR_TAG,
                ciborium::Value::Array(
                    self.choices
                        .clone()
                        .into_iter()
                        .map(ciborium::Value::Bytes)
                        .collect(),
                )
                .into(),
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
        ciborium::ser::into_writer(&cbor_array, buf)
            .map_err(|_| anyhow!("Cannot encode `Vote` to CBOR"))
    }

    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote` to CBOR
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write_to_bytes(&mut bytes)?;
        Ok(bytes)
    }

    /// Decodes `Vote` from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Invalid `Vote` CBOR encoded bytes.
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        /// Invalid `Vote` CBOR encoded bytes error msg.
        const INVALID_CBOR_BYTES_ERR: &str = "Invalid `Vote` CBOR encoded bytes";

        let val: ciborium::Value = ciborium::de::from_reader(bytes)
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, not a CBOR encoded."))?;

        let array = val
            .into_array()
            .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, must be array type."))?;

        ensure!(
            array.len() == 3,
            "{INVALID_CBOR_BYTES_ERR}, must be an array of length 3.",
        );

        let mut iter = array.into_iter();

        let choices = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, choices) = iter.next().ok_or(anyhow!(
                "{INVALID_CBOR_BYTES_ERR}, missing `choices` field."
            ))?
            else {
                bail!(
                    "{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}."
                );
            };
            let choices = choices
                .into_array()
                .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `choices` must be array type."))?;

            ensure!(
                !choices.is_empty(),
                "{INVALID_CBOR_BYTES_ERR}, `choices` must have at least one element.",
            );

            choices
                .into_iter()
                .enumerate()
                .map(|(i, choice)| {
                    choice
                        .into_bytes()
                        .map_err(|_| anyhow!("`choice` {i} must be bytes type."))
                })
                .collect::<anyhow::Result<_>>()?
        };

        let proof = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, proof) = iter
                .next()
                .ok_or(anyhow!("{INVALID_CBOR_BYTES_ERR}, missing `proof` field."))?
            else {
                bail!(
                    "{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}."
                );
            };
            proof
                .into_bytes()
                .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `proof` must be bytes type."))?
        };

        let prop_id = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, prop_id) = iter.next().ok_or(anyhow!(
                "{INVALID_CBOR_BYTES_ERR}, missing `prod-id` field."
            ))?
            else {
                bail!(
                    "{INVALID_CBOR_BYTES_ERR}, must be a major type 6 with tag {ENCODED_CBOR_TAG}."
                );
            };
            prop_id
                .into_bytes()
                .map_err(|_| anyhow!("{INVALID_CBOR_BYTES_ERR}, `prop-id` must be bytes type."))?
        };

        Ok(Self {
            choices,
            proof,
            prop_id,
        })
    }
}
