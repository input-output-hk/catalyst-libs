//! A contest ballot payload.

mod choices;
mod encrypted_block;
mod encrypted_choices;

use catalyst_signed_doc::problem_report::ProblemReport;
use cbork_utils::{decode_context::DecodeCtx, map::Map};
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

pub(crate) use self::{choices::Choices, encrypted_choices::EncryptedChoices};

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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContestBallotPayload {
    /// A map of voters choices.
    pub choices: Vec<Choices>,
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

impl Decode<'_, ProblemReport> for ContestBallotPayload {
    fn decode(
        d: &mut Decoder<'_>,
        report: &mut ProblemReport,
    ) -> Result<Self, minicbor::decode::Error> {
        use minicbor::data::Type;

        let context = "Content ballot payload decoding";

        let map = Map::decode(d, &mut DecodeCtx::Deterministic)?;

        let mut choices = Vec::new();
        let column_proof = None;
        let matrix_proof = None;
        let mut voter_choices = None;

        for (i, entry) in map.into_iter().enumerate() {
            let mut key_decoder = Decoder::new(&entry.key_bytes);
            let mut value_decoder = Decoder::new(&entry.value);

            match key_decoder.datatype()? {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                    let key = key_decoder.u64()?;
                    if !u64::try_from(i).is_ok_and(|i| i == key) {
                        report.other(
                            &format!("choices keys must be continuous, expected {i}, found {key}"),
                            context,
                        );
                    }
                    match Choices::decode(&mut value_decoder, report) {
                        Ok(val) => {
                            choices.push(val);
                        },
                        Err(e) => {
                            report.other(
                                &format!("Unable to decode choices for {key} key: {e:?}"),
                                context,
                            );
                        },
                    }
                },
                Type::String => {
                    match key_decoder.str()? {
                        "column-proof" => {
                            report.other(
                                "column-proof is a placeholder and shouldn't be used",
                                context,
                            );
                        },
                        "matrix-proof" => {
                            report.other(
                                "matrix-proof is a placeholder and shouldn't be used",
                                context,
                            );
                        },
                        "voter-choices" => {
                            match EncryptedChoices::decode(&mut value_decoder, &mut ()) {
                                Ok(v) => voter_choices = Some(v),
                                Err(e) => {
                                    report.other(
                                        &format!("Unable to decode encrypted choices: {e:?}"),
                                        context,
                                    );
                                },
                            }
                        },
                        key => {
                            report.other(
                                &format!("Unexpected content ballot payload key value: {key:?}"),
                                context,
                            );
                        },
                    }
                },
                t => {
                    report.other(
                        &format!("Unexpected content ballot payload key type: {t:?}"),
                        context,
                    );
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

impl Encode<()> for ContestBallotPayload {
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

        for (key, val) in self.choices.iter().enumerate() {
            e.u64(key.try_into().map_err(minicbor::encode::Error::message)?)?
                .encode(val)?;
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
    use encrypted_block::EncryptedBlock;

    use super::*;

    #[test]
    fn roundtrip() {
        let original = ContestBallotPayload {
            choices: [Choices::Clear(vec![1, 2, 3, 4, 5])].into(),
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
        let mut report = ProblemReport::new("test");
        let decoded =
            ContestBallotPayload::decode(&mut Decoder::new(&buffer), &mut report).unwrap();
        assert_eq!(original, decoded);
        println!("{report:?}");
        assert!(!report.is_problematic());
    }
}
