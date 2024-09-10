//! `OtherNameHardwareModuleName`, special type for `hardwareModuleName` type of
//! otherName. When 'otherName + hardwareModuleName' is used, then `[ ~oid, bytes ]` is
//! used to contain the pair ( hwType, hwSerialNum ) directly as specified in
//! [RFC4108](https://datatracker.ietf.org/doc/rfc4108/)

use asn1_rs::Oid;
use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

use crate::{
    helper::{
        decode::{decode_array_len, decode_bytes},
        encode::{encode_array_len, encode_bytes},
    },
    oid::C509oid,
};

/// A struct represents the hardwareModuleName type of otherName.
/// Containing a pair of ( hwType, hwSerialNum ) as mentioned in
/// [RFC4108](https://datatracker.ietf.org/doc/rfc4108/)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct OtherNameHardwareModuleName {
    /// The hardware type OID.
    hw_type: C509oid,
    /// The hardware serial number represent in bytes.
    hw_serial_num: Vec<u8>,
}

impl OtherNameHardwareModuleName {
    /// Create a new instance of `OtherNameHardwareModuleName`.
    #[must_use]
    pub fn new(hw_type: Oid<'static>, hw_serial_num: Vec<u8>) -> Self {
        Self {
            hw_type: C509oid::new(hw_type),
            hw_serial_num,
        }
    }

    /// Get the c509 OID hardware type.
    #[must_use]
    pub fn hw_type(&self) -> &C509oid {
        &self.hw_type
    }

    /// Get the hardware serial number.
    #[must_use]
    pub fn hw_serial_num(&self) -> &[u8] {
        &self.hw_serial_num
    }
}

impl Encode<()> for OtherNameHardwareModuleName {
    fn encode<W: Write>(
        &self, e: &mut Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_array_len(e, "Other name hardware module", 2)?;
        self.hw_type.encode(e, ctx)?;
        encode_bytes(
            e,
            "Other name hardware module serial number",
            &self.hw_serial_num,
        )?;
        Ok(())
    }
}

impl<'a> Decode<'a, ()> for OtherNameHardwareModuleName {
    fn decode(d: &mut Decoder<'a>, ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        decode_array_len(d, "Other name hardware module")?;
        let hw_type = C509oid::decode(d, ctx)?;
        let hw_serial_num = decode_bytes(d, "Other name hardware module serial number")?;
        Ok(OtherNameHardwareModuleName::new(
            hw_type.oid().clone(),
            hw_serial_num,
        ))
    }
}
