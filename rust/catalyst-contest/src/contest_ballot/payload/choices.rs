//! Voters Choices.

use catalyst_signed_doc::problem_report::ProblemReport;
use catalyst_voting::vote_protocol::voter::{EncryptedVote, proof::VoterProof};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A clear choice indicator. See the `Choices` CBOR schema for the details.
const CLEAR_CHOICE: u8 = 0;

/// An encrypted choice indicator. See the `Choices` CBOR schema for the details.
const ENCRYPTED_CHOICE: u8 = 1;

/// Voters Choices.
///
/// The CDDL schema:
/// ```cddl
/// choices = [ 0, clear-choices ] /
/// [ 1, elgamal-ristretto255-encrypted-choices ]
///
/// clear-choices = ( +clear-choice )
///
/// clear-choice = uint
///
/// elgamal-ristretto255-encrypted-choices = [
///     [+ elgamal-ristretto255-encrypted-choice]
///     ? row-proof
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Choices {
    /// A universal unencrypted set of choices.
    Clear(Vec<u64>),
    /// ElGamal/Ristretto255 encrypted choices.
    Encrypted {
        /// A list of ElGamal/Ristretto255 encrypted choices.
        choices: EncryptedVote,
        /// A universal encrypted row proof.
        row_proof: Option<VoterProof>,
    },
}

impl Decode<'_, ProblemReport> for Choices {
    fn decode(
        d: &mut Decoder<'_>,
        report: &mut ProblemReport,
    ) -> Result<Self, minicbor::decode::Error> {
        let array = Array::decode(d, &mut DecodeCtx::Deterministic)?;
        let [type_, choices @ ..] = array.as_slice() else {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected choices array length: {}, expected at least 2",
                array.len()
            )));
        };

        let mut type_decoder = Decoder::new(type_);
        match type_decoder.u8()? {
            CLEAR_CHOICE => {
                let mut values = Vec::with_capacity(choices.len());
                for choice in choices {
                    let mut d = Decoder::new(choice);
                    values.push(u64::decode(&mut d, &mut ())?);
                }
                Ok(Self::Clear(values))
            },
            ENCRYPTED_CHOICE => {
                let [choices] = choices else {
                    return Err(minicbor::decode::Error::message(format!(
                        "Unexpected encrypted choices array length {}, expected 2",
                        choices.len().saturating_add(1)
                    )));
                };

                let mut d = Decoder::new(choices);
                let array = Array::decode(&mut d, &mut DecodeCtx::Deterministic)?;
                let (choices, row_proof) = match array.as_slice() {
                    [choices] => {
                        let mut d = Decoder::new(choices);
                        let choices = EncryptedVote::decode(&mut d, &mut ())?;
                        report.missing_field("row_proof", "Contest ballot payload decoding");
                        (choices, None)
                    },
                    [choices, proof] => {
                        let mut d = Decoder::new(choices);
                        let choices = EncryptedVote::decode(&mut d, &mut ())?;

                        let mut d = Decoder::new(proof);
                        let row_proof = VoterProof::decode(&mut d, &mut ())?;

                        (choices, Some(row_proof))
                    },
                    _ => {
                        return Err(minicbor::decode::Error::message(format!(
                            "Unexpected elgamal-ristretto255-encrypted-choices array length {}, expected 1 or 2",
                            array.len()
                        )));
                    },
                };

                Ok(Self::Encrypted { choices, row_proof })
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
                e.array((choices.len() as u64).checked_add(1).ok_or_else(|| {
                    minicbor::encode::Error::message("Clear choices array length overflow")
                })?)?;
                0.encode(e, ctx)?;
                for choice in choices {
                    choice.encode(e, ctx)?;
                }
            },
            Choices::Encrypted { choices, row_proof } => {
                e.array(2)?;
                1.encode(e, ctx)?;
                // Allowed because 1 + 1 will never result in overflow.
                #[allow(clippy::arithmetic_side_effects)]
                e.array(1 + u64::from(row_proof.is_some()))?;
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
    use catalyst_voting::crypto::elgamal::Ciphertext;

    use super::*;

    #[test]
    fn clear_roundtrip() {
        let original = Choices::Clear(vec![1, 2, 3]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let mut report = ProblemReport::new("test");
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut report).unwrap();
        assert_eq!(original, decoded);
        assert!(!report.is_problematic());
    }

    #[test]
    fn encrypted_roundtrip() {
        let original = Choices::Encrypted {
            choices: vec![Ciphertext::zero(), Ciphertext::zero()].into(),
            row_proof: None,
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let mut report = ProblemReport::new("test");
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut report).unwrap();
        assert_eq!(original, decoded);
        assert!(!report.is_problematic());
    }
}
