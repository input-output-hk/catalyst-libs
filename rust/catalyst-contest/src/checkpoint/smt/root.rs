//! Root of a Sparse Merkle Tree (SMT).

use cbork_utils::BLAKE3_CBOR_TAG;
use minicbor::{
    Decode, Encode, data::Tag, decode::Error as DecodeError, encode::Error as EncodeError,
};

/// Root of a Sparse Merkle Tree (SMT).
///
/// Hash size is determined by length of bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtRoot(pub(crate) Vec<u8>);

impl Encode<()> for SmtRoot {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.tag(Tag::new(BLAKE3_CBOR_TAG))?;
        e.bytes(&self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for SmtRoot {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let tag = d.tag()?.as_u64();
        if tag != BLAKE3_CBOR_TAG {
            return Err(DecodeError::message(format!(
                "Expected Blake3 CBOR Tag {BLAKE3_CBOR_TAG}, got {tag}"
            )));
        }
        let bytes = d.bytes()?;
        let root = bytes.to_vec();
        Ok(Self(root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let original = SmtRoot(vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ]);

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = SmtRoot::decode(&mut minicbor::Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
