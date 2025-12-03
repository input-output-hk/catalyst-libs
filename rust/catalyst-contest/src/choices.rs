//! Voters Choices.

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// Voters Choices.
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

/// An elgamal encrypted ciphertext `(c1, c2)`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ElgamalRistretto255Choice {
    /// An individual Elgamal group element that composes the elgamal cipher text.
    pub c1: [u8; 32],
    /// An individual Elgamal group element that composes the elgamal cipher text.
    pub c2: [u8; 32],
}

/// A universal encrypted row proof.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RowProof {
    // TODO: FIXME:
}

impl Decode<'_, ()> for Choices {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // TODO: FIXME:
        todo!()
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
                e.array(choices.len() as u64 + row_proof.is_some() as u64)?;
                choices.encode(e, ctx)?;
                if let Some(row_proof) = row_proof {
                    row_proof.encode(e, ctx)?;
                }
            },
        }
        Ok(())
    }
}

impl Decode<'_, ()> for ElgamalRistretto255Choice {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // TODO: FIXME:
        todo!()
    }
}

impl Encode<()> for ElgamalRistretto255Choice {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: FIXME:
        todo!()
    }
}

impl Decode<'_, ()> for RowProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // TODO: FIXME:
        todo!()
    }
}

impl Encode<()> for RowProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: FIXME:
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_choices_roundtrip() {
        let original = Choices::Clear(vec![1, 2, 3]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn elgamal_ristretto255_choices_roundtrip() {
        let original = Choices::ElgamalRistretto255 {
            choices: vec![],
            row_proof: Some(RowProof {}),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn elgamal_ristretto255_choice_roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = ElgamalRistretto255Choice {
            c1: bytes,
            c2: bytes,
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            ElgamalRistretto255Choice::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn row_proof_roundtrip() {
        let original = RowProof {};
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = RowProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
