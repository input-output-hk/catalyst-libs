//! Unencrypted Tally Proposal Result

use std::collections::HashMap;

use cbork_utils::{array::Array, decode_context::DecodeCtx, map::Map};
use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

use super::{ClearChoice, VotingPower};

/// Placeholder map of `tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ProposalResult(HashMap<ClearChoice, VotingPower>);

impl Encode<()> for ProposalResult {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        // Encode as [ 0, undefined ] per CDDL
        e.array(2)?;
        e.u8(0)?;
        e.map(self.0.len() as u64)?;

        // Sort entries by their CBOR-encoded key for RFC 8949 canonical ordering
        // (length-first, then lexicographic for equal-length keys)
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_cached_key(|(choice, _)| {
            let mut buf = Vec::new();
            drop(choice.encode(&mut Encoder::new(&mut buf), &mut ()));
            buf
        });

        for (choice, voting_power) in entries {
            choice.encode(e, ctx)?;
            voting_power.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for ProposalResult {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let array = Array::decode(d, &mut DecodeCtx::Deterministic)?;
        if array.len() != 2 {
            return Err(DecodeError::message(format!(
                "tally-proposal-result must have 2 elements, got {}",
                array.len()
            )));
        }

        let mut version_decoder = Decoder::new(&array[0]);
        let version = version_decoder.u8()?;
        if version != 0 {
            return Err(DecodeError::message(format!(
                "tally-proposal-result version must be 0, got {version}",
            )));
        }

        let mut map_decoder = Decoder::new(&array[1]);
        let entries = Map::decode(&mut map_decoder, &mut DecodeCtx::Deterministic)?;

        let mut choice_map = HashMap::new();
        for entry in entries.as_slice() {
            let mut key_decoder = Decoder::new(&entry.key_bytes);
            let choice = ClearChoice::decode(&mut key_decoder, ctx)?;

            let mut value_decoder = Decoder::new(&entry.value);
            let voting_power = VotingPower::decode(&mut value_decoder, ctx)?;

            if choice_map.insert(choice, voting_power).is_some() {
                return Err(DecodeError::message("Duplicate choice key"));
            }
        }

        Ok(Self(choice_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_proposal_result() {
        let mut choice_map = HashMap::new();
        choice_map.insert(0.into(), VotingPower::from(100));
        choice_map.insert(1.into(), VotingPower::from(200));
        choice_map.insert(ClearChoice::from(-1), VotingPower::from(50));

        let original = ProposalResult(choice_map);

        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ProposalResult::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
