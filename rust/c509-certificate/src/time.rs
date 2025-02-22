//! C509 Time

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};
use serde::{Deserialize, Serialize};

use crate::helper::{
    decode::{decode_datatype, decode_helper, decode_null},
    encode::{encode_helper, encode_null},
};

/// A struct representing a time where it accept seconds since the Unix epoch.
/// Doesn't support dates before the Unix epoch (January 1, 1970, 00:00:00 UTC)
/// so unsigned integer is used.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Time(u64);

/// No expiration date in seconds since the Unix epoch.
const NO_EXP_DATE: u64 = 253_402_300_799;

impl Time {
    /// Create a new instance of `Time`.
    #[must_use]
    pub fn new(time: u64) -> Self {
        Self(time)
    }

    /// Get the u64 of `Time`.
    #[must_use]
    pub fn time(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Time {
    fn from(value: u64) -> Self {
        Time::new(value)
    }
}

impl From<Time> for u64 {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl Encode<()> for Time {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.0 == NO_EXP_DATE {
            encode_null(e, "Time")?;
        } else {
            encode_helper(e, "Time", ctx, &self.0)?;
        }
        Ok(())
    }
}

impl Decode<'_, ()> for Time {
    fn decode(d: &mut Decoder<'_>, _ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        match decode_datatype(d, "Time")? {
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64 => {
                let time = decode_helper(d, "Time", &mut ())?;
                Ok(Time::new(time))
            },
            minicbor::data::Type::Null => {
                decode_null(d, "Time")?;
                Ok(Time::new(NO_EXP_DATE))
            },
            _ => Err(minicbor::decode::Error::message("Invalid type for Time")),
        }
    }
}

#[cfg(test)]
mod test_time {

    use super::*;

    #[test]
    fn test_encode_decode_no_exp_date() {
        let mut buffer = Vec::new();
        let mut encoder = minicbor::Encoder::new(&mut buffer);
        let time = Time::new(NO_EXP_DATE);
        time.encode(&mut encoder, &mut ())
            .expect("Failed to encode Time");
        // null: 0xf6
        assert_eq!(hex::encode(buffer.clone()), "f6");

        let mut decoder = minicbor::Decoder::new(&buffer);
        let decoded_time = Time::decode(&mut decoder, &mut ()).expect("Failed to decode Time");

        assert_eq!(decoded_time, time);
    }

    // Test reference https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
    // A.1.  Example RFC 7925 profiled X.509 Certificate
    #[test]
    fn test_encode_decode() {
        let mut buffer = Vec::new();
        let mut encoder = minicbor::Encoder::new(&mut buffer);
        // Jan 1 00:00:00 2023 GMT
        let time = Time::new(1_672_531_200);
        time.encode(&mut encoder, &mut ())
            .expect("Failed to encode Time");
        // 1A 63B0CD00 # unsigned(1672531200)
        assert_eq!(hex::encode(buffer.clone()), "1a63b0cd00");

        let mut decoder = minicbor::Decoder::new(&buffer);
        let decoded_time = Time::decode(&mut decoder, &mut ()).expect("Failed to decode Time");

        assert_eq!(decoded_time, time);
    }
}
