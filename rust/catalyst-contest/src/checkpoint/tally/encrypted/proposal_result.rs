//! Encrypted Tally Proposal Result

use cbork_utils::{array::Array, decode_context::DecodeCtx};
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
        let array = Array::decode(d, &mut DecodeCtx::Deterministic)?;
        let [version, undefined] = array.as_slice() else {
            return Err(DecodeError::message(format!(
                "encrypted-tally-proposal-result must have 2 elements, got {}",
                array.len()
            )));
        };

        let mut version_decoder = Decoder::new(version);
        let version = version_decoder.u8()?;
        if version != 1 {
            return Err(DecodeError::message(format!(
                "encrypted-tally-proposal-result version must be 1, got {version}",
            )));
        }

        let mut undefined_decoder = Decoder::new(undefined);
        undefined_decoder.undefined()?;

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
