//! Voters Choices.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::{elgamal_ristretto255_choice::ElgamalRistretto255Choice, row_proof::RowProof};

/// Voters Choices.
///
/// The CDDL schema:
/// ```cddl
/// choices = [ 0, clear-choices ] /
/// [ 1, elgamal-ristretto255-encrypted-choices ]
///
/// clear-choices = ( +clear-choice )
///
/// clear-choice = int
///
/// elgamal-ristretto255-encrypted-choices = [
///     [+ elgamal-ristretto255-encrypted-choice]
///     ? row-proof
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Choices {
    /// A universal unencrypted set of choices.
    Clear(Vec<i64>),
    /// ElGamal/Ristretto255 encrypted choices.
    ElgamalRistretto255 {
        /// ElGamal/Ristretto255 encrypted choices.
        choices: Vec<ElgamalRistretto255Choice>,
        /// A universal encrypted row proof.
        row_proof: Option<RowProof>,
    },
}

impl Decode<'_, ()> for Choices {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "choices")?;
        if len < 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected choices array length {len}, expected at least 2"
            )));
        }
        match u8::decode(d, ctx)? {
            0 => Ok(Self::Clear(<Vec<i64>>::decode(d, ctx)?)),
            1 => {
                let len = decode_array_len(d, "elgamal-ristretto255-encrypted-choices")?;
                if !(1..=2).contains(&len) {
                    return Err(minicbor::decode::Error::message(format!(
                        "Unexpected elgamal-ristretto255-encrypted-choices array length {len}, expected 1 or 2"
                    )));
                }
                let choices = <Vec<ElgamalRistretto255Choice>>::decode(d, ctx)?;
                let mut row_proof = None;
                if len == 2 {
                    row_proof = Some(RowProof::decode(d, ctx)?);
                }
                Ok(Self::ElgamalRistretto255 { choices, row_proof })
            },
            val => {
                Err(minicbor::decode::Error::message(format!(
                    "Unexpected choices value: {val}"
                )))
            },
        }
    }
}

impl Encode<()> for Choices {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Choices::Clear(choices) => {
                e.array(choices.len() as u64 + 1)?;
                0.encode(e, ctx)?;
                for choice in choices {
                    choice.encode(e, ctx)?;
                }
            },
            Choices::ElgamalRistretto255 { choices, row_proof } => {
                e.array(2)?;
                1.encode(e, ctx)?;
                e.array(choices.len() as u64 + u64::from(row_proof.is_some()))?;
                choices.encode(e, ctx)?;
                if let Some(row_proof) = row_proof {
                    row_proof.encode(e, ctx)?;
                }
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row_proof::{
        ProofAnnouncement, ProofAnnouncementElement, ProofResponse, ProofScalar,
        SingleSelectionProof,
    };

    #[test]
    fn clear_roundtrip() {
        let original = Choices::Clear(vec![1, 2, 3]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn elgamal_ristretto255_roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = Choices::ElgamalRistretto255 {
            choices: vec![],
            row_proof: Some(RowProof {
                selections: vec![SingleSelectionProof {
                    announcement: ProofAnnouncement(
                        ProofAnnouncementElement(bytes),
                        ProofAnnouncementElement(bytes),
                        ProofAnnouncementElement(bytes),
                    ),
                    choice: ElgamalRistretto255Choice {
                        c1: bytes,
                        c2: bytes,
                    },
                    response: ProofResponse(
                        ProofScalar(bytes),
                        ProofScalar(bytes),
                        ProofScalar(bytes),
                    ),
                }],
                scalar: ProofScalar(bytes),
            }),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
