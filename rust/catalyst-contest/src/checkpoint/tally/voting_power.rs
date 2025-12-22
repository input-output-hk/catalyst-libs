//! Voting Power.

use minicbor::{
    Decode, Decoder, Encode, Encoder,
    decode::Error as DecodeError,
    encode::{Error as EncodeError, Write},
};

/// Placeholder map of `voting-power`.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct VotingPower(i64);

impl From<i64> for VotingPower {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_voting_power() {
        let test_values = [0i64, 1, -1, 1000, -1000, i64::MAX, i64::MIN];

        for value in test_values {
            let original = VotingPower::from(value);

            let mut buffer = Vec::new();
            original
                .encode(&mut Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded = VotingPower::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
            assert_eq!(original, decoded);
        }
    }
}
