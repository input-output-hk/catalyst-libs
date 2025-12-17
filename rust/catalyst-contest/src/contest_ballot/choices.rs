//! Voters Choices.

use catalyst_voting::crypto::{elgamal::Ciphertext, zk_unit_vector::UnitVectorProof};
use cbork_utils::decode_helper::decode_array_len;
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
        choices: Vec<Ciphertext>,
        /// A universal encrypted row proof.
        row_proof: Option<UnitVectorProof>,
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
            CLEAR_CHOICE => {
                let mut values = Vec::with_capacity(
                    len.checked_sub(1)
                        .ok_or_else(|| {
                            minicbor::decode::Error::message("Choices array length underflow")
                        })?
                        .try_into()
                        .map_err(minicbor::decode::Error::message)?,
                );
                for _ in 1..len {
                    values.push(u64::decode(d, ctx)?);
                }
                Ok(Self::Clear(values))
            },
            ENCRYPTED_CHOICE => {
                if len > 2 {
                    return Err(minicbor::decode::Error::message(format!(
                        "Unexpected encrypted choices array length {len}, expected 2"
                    )));
                }

                let len = decode_array_len(d, "elgamal-ristretto255-encrypted-choices")?;
                if !(1..=2).contains(&len) {
                    return Err(minicbor::decode::Error::message(format!(
                        "Unexpected elgamal-ristretto255-encrypted-choices array length {len}, expected 1 or 2"
                    )));
                }
                let choices = <Vec<Ciphertext>>::decode(d, ctx)?;
                let mut row_proof = None;
                if len == 2 {
                    row_proof = Some(UnitVectorProof::decode(d, ctx)?);
                }
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
    use super::*;

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
    fn encrypted_roundtrip() {
        let original = Choices::Encrypted {
            choices: vec![Ciphertext::zero(), Ciphertext::zero()],
            row_proof: None,
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Choices::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
