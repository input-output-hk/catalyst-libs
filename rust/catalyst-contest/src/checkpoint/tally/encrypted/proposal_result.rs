//! Encrypted Tally Proposal Result

use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

/// Placeholder map of `encrypted-tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct EncryptedTallyProposalResult;

impl Encode<()> for EncryptedTallyProposalResult {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        // Encode as [ 1, undefined ] per CDDL
        e.array(2)?;
        e.u8(1)?;
        e.undefined()?;
        Ok(())
    }
}

impl Decode<'_, ()> for EncryptedTallyProposalResult {
    fn decode(
        d: &mut Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        // According to CDDL: encrypted-tally-proposal-result = [ 1, undefined ]
        // For now, decode and store as placeholder String
        let Some(arr_len) = d.array()? else {
            return Err(DecodeError::message(
                "encrypted-tally-proposal-result must be a defined-size array",
            ));
        };
        if arr_len != 2 {
            return Err(DecodeError::message(format!(
                "encrypted-tally-proposal-result must have 2 elements, got {arr_len}",
            )));
        }

        let version = d.u8()?;
        if version != 1 {
            return Err(DecodeError::message(format!(
                "encrypted-tally-proposal-result version must be 1, got {version}",
            )));
        }
        d.undefined()?;
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let original = EncryptedTallyProposalResult;

        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            EncryptedTallyProposalResult::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
