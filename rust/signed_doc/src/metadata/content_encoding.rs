//! Document Payload Content Encoding.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr, VariantArray};

use super::utils::transcode_ciborium_with;

/// IANA `CoAP` Content Encoding.
// TODO: add custom parse error type when the [strum issue]([`issue`](https://github.com/Peternator7/strum/issues/430)) fix is merged.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    VariantArray,
    EnumString,
    Display,
    IntoStaticStr,
    Serialize,
    Deserialize,
)]
#[serde(try_from = "&str", into = "&str")]
pub enum ContentEncoding {
    /// Brotli compression.format.
    #[strum(to_string = "br")]
    Brotli,
}

impl ContentEncoding {
    /// Compress a Brotli payload
    ///
    /// # Errors
    /// Returns compression failure
    pub fn encode(self, mut payload: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Brotli => {
                let brotli_params = brotli::enc::BrotliEncoderParams::default();
                let mut buf = Vec::new();
                brotli::BrotliCompress(&mut payload, &mut buf, &brotli_params)?;
                Ok(buf)
            },
        }
    }

    /// Decompress a Brotli payload
    ///
    /// # Errors
    ///  Returns decompression failure
    pub fn decode(self, mut payload: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Brotli => {
                let mut buf = Vec::new();
                brotli::BrotliDecompress(&mut payload, &mut buf)?;
                Ok(buf)
            },
        }
    }

    /// An error returned on [`minicbor::Decode::decode`] failure.
    fn decode_error(input: &str) -> minicbor::decode::Error {
        minicbor::decode::Error::message(format!(
            "Unsupported Content Type {input:?}, Supported only: {:?}",
            ContentEncoding::VARIANTS
                .iter()
                .map(<&str>::from)
                .collect::<Vec<_>>()
        ))
    }
}

impl<C> minicbor::Encode<C> for ContentEncoding {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.str(<&str>::from(self))?.ok()
    }
}

impl<'b, C> minicbor::Decode<'b, C> for ContentEncoding {
    fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        let s = d.str()?;
        s.parse().map_err(|_| Self::decode_error(s))
    }
}

impl TryFrom<&coset::cbor::Value> for ContentEncoding {
    type Error = minicbor::decode::Error;

    fn try_from(val: &coset::cbor::Value) -> Result<Self, minicbor::decode::Error> {
        transcode_ciborium_with(val, &mut ())
    }
}
