//! A Catalyst vote transaction v2 objects, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/)

// cspell: words Coap

use anyhow::anyhow;
use minicbor::{Decode, Decoder, Encode, Encoder};

pub mod gen_tx;

/// Cbor encodable and decodable type trait.
pub trait Cbor<'a>: Encode<()> + Decode<'a, ()> {
    /// Encodes to CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot encode
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut e = Encoder::new(&mut bytes);
        self.encode(&mut e, &mut ())
            .map_err(|e| anyhow!("Cannot encode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(bytes)
    }

    /// Decodes from the CBOR encoded bytes.
    ///
    /// # Errors
    ///  - Cannot decode
    fn from_bytes(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let mut decoder = Decoder::new(bytes);
        let res = Self::decode(&mut decoder, &mut ())
            .map_err(|e| anyhow!("Cannot decode `{}`, {e}.", std::any::type_name::<Self>()))?;
        Ok(res)
    }
}

impl<'a, T> Cbor<'a> for T where T: Encode<()> + Decode<'a, ()> {}
