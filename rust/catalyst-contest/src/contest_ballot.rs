//! An individual Ballot cast in a Contest by a registered user.

use std::collections::BTreeMap;

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::{Choices, ColumnProof, EncryptedChoices, MatrixProof};

/// An individual Ballot cast in a Contest by a registered user.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentBallot {
    /// A map of voters choices.
    pub choices: BTreeMap<u64, Choices>,
    /// A universal encrypted column proof.
    pub column_proof: Option<ColumnProof>,
    /// A universal encrypted matrix proof.
    pub matrix_proof: Option<MatrixProof>,
    /// An encrypted voter choice payload.
    pub voter_choices: Option<EncryptedChoices>,
}

impl Decode<'_, ()> for ContentBallot {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        d.map()?;
        // TODO: FIXME:
        todo!()
    }
}

impl Encode<()> for ContentBallot {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.begin_map()?;

        for (&key, val) in self.choices.iter() {
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

        e.end()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElgamalRistretto255Choice, EncryptedBlock, RowProof};

    #[test]
    fn roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = ContentBallot {
            choices: [
                (1, Choices::Clear(vec![1, 2, 3, -4, -5].into())),
                (2, Choices::ElgamalRistretto255 {
                    choices: vec![ElgamalRistretto255Choice {
                        c1: bytes,
                        c2: bytes,
                    }],
                    row_proof: None,
                }),
                (3, Choices::ElgamalRistretto255 {
                    choices: vec![ElgamalRistretto255Choice {
                        c1: bytes,
                        c2: bytes,
                    }],
                    row_proof: Some(RowProof {}),
                }),
            ]
            .into(),
            column_proof: Some(ColumnProof(1)),
            matrix_proof: Some(MatrixProof(2)),
            voter_choices: Some(EncryptedChoices(vec![
                EncryptedBlock([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
                EncryptedBlock([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            ])),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ContentBallot::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
