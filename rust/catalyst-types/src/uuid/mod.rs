//! `UUID` types.

mod uuid_v4;
mod uuid_v7;

use minicbor::data::Tag;
pub use uuid_v4::UuidV4;
pub use uuid_v7::UuidV7;

/// Invalid Doc Type UUID
pub const INVALID_UUID: uuid::Uuid = uuid::Uuid::from_bytes([0x00; 16]);

/// UUID CBOR tag <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml/>.
#[allow(dead_code)]
const UUID_CBOR_TAG: u64 = 37;

/// Context for `CBOR` encoding and decoding
pub enum CborContext {
    /// Untagged bytes
    Untagged,
    /// IANA CBOR tag and bytes
    Tagged,
    /// Optional tag
    Optional,
}

/// Validate UUID CBOR Tag.
fn validate_uuid_tag(tag: u64) -> Result<(), minicbor::decode::Error> {
    if UUID_CBOR_TAG != tag {
        return Err(minicbor::decode::Error::message(format!(
            "tag value must be: {UUID_CBOR_TAG}, provided: {tag}"
        )));
    }
    Ok(())
}

/// Decode from `CBOR` into `UUID`
fn decode_cbor_uuid(
    d: &mut minicbor::Decoder<'_>, ctx: &mut CborContext,
) -> Result<uuid::Uuid, minicbor::decode::Error> {
    let bytes = match ctx {
        CborContext::Untagged => d.bytes()?,
        CborContext::Tagged => {
            let tag = d.tag()?;
            validate_uuid_tag(tag.as_u64())?;
            d.bytes()?
        },
        CborContext::Optional => {
            let pos = d.position();
            if let Ok(tag) = d.tag() {
                validate_uuid_tag(tag.as_u64())?;
                d.bytes()?
            } else {
                d.set_position(pos);
                d.bytes()?
            }
        },
    };
    let decoded: [u8; 16] = bytes.try_into().map_err(|_| {
        minicbor::decode::Error::message("Invalid CBOR encoded UUID type: invalid bytes size")
    })?;
    let uuid = uuid::Uuid::from_bytes(decoded);
    Ok(uuid)
}

/// Encode `UUID` into `CBOR`
fn encode_cbor_uuid<W: minicbor::encode::Write>(
    uuid: uuid::Uuid, e: &mut minicbor::Encoder<W>, ctx: &mut CborContext,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    if let CborContext::Tagged = ctx {
        e.tag(Tag::new(UUID_CBOR_TAG))?;
    }
    e.bytes(uuid.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::{UuidV4, UuidV7};
    use crate::uuid::CborContext;

    #[test]
    fn test_cbor_uuid_v4_roundtrip() {
        let uuid: UuidV4 = uuid::Uuid::new_v4().into();
        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Untagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_cbor_uuid_v7_roundtrip() {
        let uuid: UuidV7 = uuid::Uuid::now_v7().into();
        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Untagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_tagged_cbor_uuid_v4_roundtrip() {
        let uuid: UuidV4 = uuid::Uuid::new_v4().into();
        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Tagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Tagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_tagged_cbor_uuid_v7_roundtrip() {
        let uuid: UuidV7 = uuid::Uuid::now_v7().into();
        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Tagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Tagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_optional_cbor_uuid_v4_roundtrip() {
        let uuid: UuidV4 = uuid::Uuid::new_v4().into();

        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Untagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Optional).unwrap();
        assert_eq!(uuid, decoded);

        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Tagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Optional).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_optional_cbor_uuid_v7_roundtrip() {
        let uuid: UuidV7 = uuid::Uuid::now_v7().into();

        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Untagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Optional).unwrap();
        assert_eq!(uuid, decoded);

        let mut bytes = Vec::new();
        minicbor::encode_with(uuid, &mut bytes, &mut CborContext::Tagged).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Optional).unwrap();
        assert_eq!(uuid, decoded);
    }
}
