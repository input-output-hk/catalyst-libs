//! Root of a Sparse Merkle Tree (SMT).

use minicbor::{
    Decode, Encode, data::Tag, decode::Error as DecodeError, encode::Error as EncodeError,
};

/// CBOR tag for BLAKE3 HASH
/// <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml>
const BLAKE3_CBOR_TAG: u64 = 32781;

/// Default Hash Size for BLAKE3 is 32 bytes.
const DEFAULT_HASH_SIZE: usize = 32;

/// Root of a Sparse Merkle Tree (SMT).
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
        let mut root = vec![0u8; DEFAULT_HASH_SIZE];
        root.copy_from_slice(bytes);
        Ok(Self(root))
    }
}
