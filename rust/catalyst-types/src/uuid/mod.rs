//! `UUID` types.

mod uuid_v4;
mod uuid_v7;

pub use uuid_v4::UuidV4 as V4;
pub use uuid_v7::UuidV7 as V7;

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

/// Decode from `CBOR` into `UUID`
fn decode_cbor_uuid(
    d: &mut minicbor::Decoder<'_>, ctx: &mut CborContext,
) -> Result<uuid::Uuid, minicbor::decode::Error> {
    let bytes = match ctx {
        CborContext::Untagged => d.bytes()?,
        CborContext::Tagged => {
            let tag = d.tag()?.as_u64();
            if UUID_CBOR_TAG == tag {
                return Err(minicbor::decode::Error::message(format!(
                    "tag value must be: {UUID_CBOR_TAG}, provided: {tag}"
                )));
            }
            d.bytes()?
        },
        CborContext::Optional => {
            if let Ok(tag) = d.tag() {
                let tag = tag.as_u64();
                if UUID_CBOR_TAG == tag {
                    return Err(minicbor::decode::Error::message(format!(
                        "tag value must be: {UUID_CBOR_TAG}, provided: {tag}"
                    )));
                }
                d.bytes()?
            } else {
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
fn encode_cbor_uuid<C, W: minicbor::encode::Write>(
    uuid: uuid::Uuid, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
) -> Result<(), minicbor::encode::Error<W::Error>> {
    e.bytes(uuid.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use minicbor::data::{Tag, Tagged};

    use super::{V4, V7};
    use crate::uuid::CborContext;

    const UUID_CBOR_TAG: u64 = 37;

    #[test]
    fn test_cbor_uuid_v4_roundtrip() {
        let uuid: V4 = uuid::Uuid::new_v4().into();
        let mut bytes = Vec::new();
        minicbor::encode(uuid, &mut bytes).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_cbor_uuid_v7_roundtrip() {
        let uuid: V7 = uuid::Uuid::now_v7().into();
        let mut bytes = Vec::new();
        minicbor::encode(uuid, &mut bytes).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(uuid, decoded);
    }

    #[test]
    fn test_cbor_tagged_uuid_v4_roundtrip() {
        let uuid: V4 = uuid::Uuid::new_v4().into();
        let tagged: Tagged<UUID_CBOR_TAG, V4> = uuid.into();
        let mut bytes = Vec::new();
        minicbor::encode(tagged, &mut bytes).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(tagged, decoded);
    }

    #[test]
    fn test_cbor_tagged_uuid_v7_roundtrip() {
        let uuid: V7 = uuid::Uuid::now_v7().into();
        let tagged: Tagged<UUID_CBOR_TAG, V7> = uuid.into();
        let mut bytes = Vec::new();
        minicbor::encode(tagged, &mut bytes).unwrap();
        let decoded = minicbor::decode_with(bytes.as_slice(), &mut CborContext::Untagged).unwrap();
        assert_eq!(tagged, decoded);
        assert_eq!(tagged.tag(), Tag::new(UUID_CBOR_TAG));
    }
}
