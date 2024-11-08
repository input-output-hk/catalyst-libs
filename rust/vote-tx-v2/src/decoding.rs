//! CBOR encoding and decoding implementation.

use std::fmt::Display;

use anyhow::{anyhow, bail, ensure};

use crate::Vote;

/// `encoded-cbor` tag number
const ENCODED_CBOR_TAG: u64 = 24;

/// CBOR decoding error.
fn cbor_decoding_err<T>(reason: impl Display) -> anyhow::Error {
    anyhow!("Cannot decode `{}`, {reason}.", std::any::type_name::<T>())
}

/// CBOR encoding error.
fn cbor_encoding_err<T>(reason: impl Display) -> anyhow::Error {
    anyhow!("Cannot encode `{}`, {reason}.", std::any::type_name::<T>())
}

impl Vote {
    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote`
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
            .map_err(|_| cbor_encoding_err::<Self>("interal `ciborioum` error"))
    }

    /// Encodes `Vote` to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode `Vote`
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write_to_bytes(&mut bytes)?;
        Ok(bytes)
    }

    /// Decodes `Vote` from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot decode `GeneralisedTxBody`
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let val: ciborium::Value = ciborium::de::from_reader(bytes)
            .map_err(|_| cbor_decoding_err::<Self>("not a CBOR encoded"))?;

        let array = val
            .into_array()
            .map_err(|_| cbor_decoding_err::<Self>("must be array type"))?;

        ensure!(
            array.len() == 3,
            cbor_decoding_err::<Self>("must be an array of length 3")
        );

        let mut iter = array.into_iter();

        let choices = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, choices) = iter
                .next()
                .ok_or(cbor_decoding_err::<Self>("missing `choices` field"))?
            else {
                bail!(cbor_decoding_err::<Self>(format!(
                    "must be a major type 6 with tag {ENCODED_CBOR_TAG}"
                )));
            };
            let choices = choices
                .into_array()
                .map_err(|_| cbor_decoding_err::<Self>("`choices` must be array type"))?;

            ensure!(
                !choices.is_empty(),
                cbor_decoding_err::<Self>("`choices` must have at least one element")
            );

            choices
                .into_iter()
                .enumerate()
                .map(|(i, choice)| {
                    choice.into_bytes().map_err(|_| {
                        cbor_decoding_err::<Self>(format!("`choice` {i} must be bytes type"))
                    })
                })
                .collect::<anyhow::Result<_>>()?
        };

        let proof = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, proof) = iter
                .next()
                .ok_or(cbor_decoding_err::<Self>("missing `proof` field"))?
            else {
                bail!(cbor_decoding_err::<Self>(format!(
                    "must be a major type 6 with tag {ENCODED_CBOR_TAG}"
                )));
            };
            proof
                .into_bytes()
                .map_err(|_| cbor_decoding_err::<Self>("`proof` must be bytes type"))?
        };

        let prop_id = {
            let ciborium::Value::Tag(ENCODED_CBOR_TAG, prop_id) = iter
                .next()
                .ok_or(cbor_decoding_err::<Self>("missing `prod-id` field"))?
            else {
                bail!(cbor_decoding_err::<Self>(format!(
                    "must be a major type 6 with tag {ENCODED_CBOR_TAG}"
                )));
            };
            prop_id
                .into_bytes()
                .map_err(|_| cbor_decoding_err::<Self>("`prop-id` must be bytes type"))?
        };

        Ok(Self {
            choices,
            proof,
            prop_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::Vote;

    #[proptest]
    fn vote_from_bytes_to_bytes_test(vote: Vote) {
        let bytes = vote.to_bytes().unwrap();
        let decoded = Vote::from_bytes(&bytes).unwrap();
        assert_eq!(vote, decoded);
    }
}
