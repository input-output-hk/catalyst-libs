//! A contest ballot payload.

use std::collections::BTreeMap;

use cbork_utils::{decode_context::DecodeCtx, map::Map};
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::{Choices, EncryptedChoices};

/// A contest ballot payload.
///
/// The CDDL schema:
/// ```cddl
/// contest-ballot-payload = {
///     + uint => choices
///     ? "column-proof" : column-proof
///     ? "matrix-proof" : matrix-proof
///     ? "voter-choice" : voter-choice
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentBallotPayload {
    /// A map of voters choices.
    pub choices: BTreeMap<u64, Choices>,
    /// A universal encrypted column proof.
    ///
    /// This is a placeholder for now and should always be `None`.
    pub column_proof: Option<()>,
    /// A universal encrypted matrix proof.
    ///
    /// This is a placeholder for now and should always be `None`.
    pub matrix_proof: Option<()>,
    /// An encrypted voter choice payload.
    pub voter_choices: Option<EncryptedChoices>,
}

impl Decode<'_, ()> for ContentBallotPayload {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;

        let map = Map::decode(d, &mut DecodeCtx::Deterministic)?;

        let mut choices = BTreeMap::new();
        let column_proof = None;
        let matrix_proof = None;
        let mut voter_choices = None;

        for entry in map {
            let mut key_decoder = Decoder::new(&entry.key_bytes);
            let mut value_decoder = Decoder::new(&entry.value);

            match key_decoder.datatype()? {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                    let key = key_decoder.u64()?;
                    let val = Choices::decode(&mut value_decoder, ctx)?;
                    choices.insert(key, val);
                },
                Type::String => {
                    match key_decoder.str()? {
                        "column-proof" => {
                            return Err(minicbor::decode::Error::message(
                                "column-proof is a placeholder and shouldn't be used",
                            ));
                        },
                        "matrix-proof" => {
                            return Err(minicbor::decode::Error::message(
                                "matrix-proof is a placeholder and shouldn't be used",
                            ));
                        },
                        "voter-choices" => {
                            voter_choices =
                                Some(EncryptedChoices::decode(&mut value_decoder, ctx)?);
                        },
                        key => {
                            return Err(minicbor::decode::Error::message(format!(
                                "Unexpected content ballot payload key value: {key:?}"
                            )));
                        },
                    }
                },
                t => {
                    return Err(minicbor::decode::Error::message(format!(
                        "Unexpected content ballot payload key type: {t:?}"
                    )));
                },
            }
        }

        Ok(Self {
            choices,
            column_proof,
            matrix_proof,
            voter_choices,
        })
    }
}

impl Encode<()> for ContentBallotPayload {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let len = u64::try_from(self.choices.len())
            .map_err(minicbor::encode::Error::message)?
            .checked_add(u64::from(self.column_proof.is_some()))
            .and_then(|v| v.checked_add(u64::from(self.matrix_proof.is_some())))
            .and_then(|v| v.checked_add(u64::from(self.voter_choices.is_some())))
            .ok_or_else(|| {
                minicbor::encode::Error::message("contest ballot payload map length overflow")
            })?;
        e.map(len)?;

        for (&key, val) in &self.choices {
            e.u64(key)?.encode(val)?;
        }
        if let Some(column_proof) = self.column_proof.as_ref() {
            e.str("column-proof")?.encode(column_proof)?;
        }
        if let Some(matrix_proof) = self.matrix_proof.as_ref() {
            e.str("matrix-proof")?.encode(matrix_proof)?;
        }
        if let Some(voter_choices) = self.voter_choices.as_ref() {
            e.str("voter-choices")?.encode(voter_choices)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use catalyst_voting::crypto::elgamal::Ciphertext;

    use super::*;
    use crate::contest_ballot::encrypted_block::EncryptedBlock;

    #[test]
    fn roundtrip() {
        let original = ContentBallotPayload {
            choices: [
                (1, Choices::Clear(vec![1, 2, 3, 4, 5])),
                (2, Choices::Encrypted {
                    choices: vec![Ciphertext::zero()],
                    row_proof: None,
                }),
            ]
            .into(),
            column_proof: None,
            matrix_proof: None,
            voter_choices: Some(EncryptedChoices(vec![
                EncryptedBlock([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
                EncryptedBlock([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            ])),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ContentBallotPayload::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
