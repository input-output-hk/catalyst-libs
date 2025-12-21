//! Unencrypted Tally Proposal Result

use std::collections::HashMap;

use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

/// Placeholder map of `tally-proposal-result`.
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ClearChoice(i64);

impl Encode<()> for ClearChoice {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.i64(self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for ClearChoice {
    fn decode(
        d: &mut Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = d.i64()?;
        Ok(Self(entries))
    }
}

/// Placeholder map of `tally-proposal-result`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct VotingPower(i64);

impl Encode<()> for VotingPower {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.i64(self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for VotingPower {
    fn decode(
        d: &mut Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = d.i64()?;
        Ok(Self(entries))
    }
}

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
        for (choice, voting_power) in &self.0 {
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
        // According to CDDL: encrypted-tally-proposal-result = [ 0, undefined ]
        // For now, decode and store as placeholder String
        let Some(arr_len) = d.array()? else {
            return Err(DecodeError::message(
                "tally-proposal-result must be a defined-size array",
            ));
        };
        if arr_len != 2 {
            return Err(DecodeError::message(format!(
                "tally-proposal-result must have 2 elements, got {arr_len}",
            )));
        }

        let version = d.u8()?;
        if version != 0 {
            return Err(DecodeError::message(format!(
                "tally-proposal-result version must be 0, got {version}",
            )));
        }
        let Some(map_len) = d.map()? else {
            return Err(DecodeError::message(
                "tally-proposal-result must have defined-size choice=>voting_power map",
            ));
        };

        let mut choice_map = HashMap::new();
        for _ in 0..map_len {
            let choice = ClearChoice::decode(d, ctx)?;
            let voting_power = VotingPower::decode(d, ctx)?;
            choice_map.insert(choice, voting_power);
        }

        Ok(Self(choice_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_clear_choice() {
        let test_values = [0i64, 1, -1, 42, -42, i64::MAX, i64::MIN];

        for value in test_values {
            let original = ClearChoice(value);

            let mut buffer = Vec::new();
            original
                .encode(&mut Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded = ClearChoice::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn roundtrip_voting_power() {
        let test_values = [0i64, 1, -1, 1000, -1000, i64::MAX, i64::MIN];

        for value in test_values {
            let original = VotingPower(value);

            let mut buffer = Vec::new();
            original
                .encode(&mut Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded = VotingPower::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
            assert_eq!(original, decoded);
        }
    }

    #[test]
    fn roundtrip_proposal_result() {
        let mut choice_map = HashMap::new();
        choice_map.insert(ClearChoice(0), VotingPower(100));
        choice_map.insert(ClearChoice(1), VotingPower(200));
        choice_map.insert(ClearChoice(-1), VotingPower(50));

        let original = ProposalResult(choice_map);

        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ProposalResult::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
