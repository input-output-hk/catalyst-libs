//! A serialize/deserialize implementation for `ElectionPublicKey`.

use serde::{Deserialize, Deserializer, Serializer};

use crate::{crypto::group::GroupElement, vote_protocol::committee::ElectionPublicKey};

/// A serialization implementation for `ElectionPublicKey`.
// Allow dead code as for now it is only used in tests.
#[allow(dead_code)]
pub fn serialize<S>(
    val: &ElectionPublicKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex = hex::encode(val.to_bytes());
    serializer.serialize_str(&hex)
}

/// A deserialization implementation for `ElectionPublicKey`.
pub fn deserialize<'de, D>(deserializer: D) -> Result<ElectionPublicKey, D::Error>
where D: Deserializer<'de> {
    let hex = String::deserialize(deserializer)?;
    let bytes = hex::decode(hex).map_err(serde::de::Error::custom)?;
    let array = <[u8; GroupElement::BYTES_SIZE]>::try_from(bytes.as_slice())
        .map_err(serde::de::Error::custom)?;
    Ok(GroupElement::from_bytes(&array)
        .map_err(serde::de::Error::custom)?
        .into())
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrapper {
        #[serde(with = "super")]
        value: ElectionPublicKey,
    }

    #[test]
    fn group_element_json_roundtrip() {
        let original = Wrapper {
            value: GroupElement::zero().into(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }
}
