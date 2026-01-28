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
        vote: EncryptedVote,
        /// A universal encrypted row proof.
        row_proof: Option<VoterProof>,
    },
}

impl Choices {
    /// Creates a clear single choice `Choices::Clear`
    pub fn new_clear_single(
        choice: usize,
        n_options: usize,
    ) -> Self {
        Self::Clear((0..n_options).map(|i| u64::from(i == choice)).collect())
    }

    /// Returns `true` if the underlying choice is a single choice, not a multi weighted
    /// choice.
    pub fn is_single(&self) -> bool {
        match self {
            Self::Clear(v) => {
                v.iter()
                    .try_fold(0_u64, |sum, b| sum.checked_add(*b))
                    .is_some_and(|sum| sum == 1)
            },
            // underlying `EncryptedVote` with its ZK proof `VoterProof` represents only a single
            // choice
            Self::Encrypted { .. } => true,
        }
    }

    /// Returns a number of options between which choice was made
    pub fn n_options(&self) -> usize {
        match self {
            Self::Clear(v) => v.len(),
            Self::Encrypted { vote, .. } => vote.n_options(),
        }
    }
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
                let (vote, row_proof) = match array.as_slice() {
                    [choices] => {
                        let mut d = Decoder::new(choices);
                        let vote = EncryptedVote::decode(&mut d, &mut ())?;
                        report.missing_field("row_proof", "Contest ballot payload decoding");
                        (vote, None)
                    },
                    [choices, proof] => {
                        let mut d = Decoder::new(choices);
                        let vote = EncryptedVote::decode(&mut d, &mut ())?;

                        let mut d = Decoder::new(proof);
                        let row_proof = VoterProof::decode(&mut d, &mut ())?;

                        (vote, Some(row_proof))
                    },
                    _ => {
                        return Err(minicbor::decode::Error::message(format!(
                            "Unexpected elgamal-ristretto255-encrypted-choices array length {}, expected 1 or 2",
                            array.len()
                        )));
                    },
                };

                Ok(Self::Encrypted { vote, row_proof })
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
            Choices::Encrypted { vote, row_proof } => {
                e.array(2)?;
                1.encode(e, ctx)?;
                // Allowed because 1 + 1 will never result in overflow.
                #[allow(clippy::arithmetic_side_effects)]
                e.array(1 + u64::from(row_proof.is_some()))?;
                vote.encode(e, ctx)?;
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
    use test_case::test_case;

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
        println!("{report:?}");
        assert!(!report.is_problematic());
    }

    #[test_case( &Choices::Clear(vec![0, 1, 0]) => true ; "clear single choice" )]
    #[test_case( &Choices::Clear(vec![1, 2, 3]) => false ; "clear multiple weighted choice" )]
    #[test_case( &Choices::Clear(vec![1, u64::MAX, 0]) => false ; "clear overflowed choice" )]
    fn clear_is_single(choices: &Choices) -> bool {
        choices.is_single()
    }
}
