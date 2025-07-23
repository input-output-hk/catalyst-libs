//! Document Type.

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

use catalyst_types::uuid::{CborContext, Uuid, UuidV4};
use minicbor::{Decode, Decoder, Encode};

/// Document type - `UUIDv4`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DocType(UuidV4);

/// `DocType` Errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DocTypeError {
    /// Invalid UUID.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(Uuid),
    /// Invalid string conversion
    #[error("Invalid string conversion: {0}")]
    StringConversion(String),
}

impl From<UuidV4> for DocType {
    fn from(value: UuidV4) -> Self {
        DocType(value)
    }
}

impl TryFrom<Uuid> for DocType {
    type Error = DocTypeError;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        UuidV4::try_from(value)
            .map_err(|_| DocTypeError::InvalidUuid(value))
            .map(Into::into)
    }
}

impl TryFrom<String> for DocType {
    type Error = DocTypeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse::<UuidV4>()
            .map_err(|_| DocTypeError::StringConversion(s))
            .map(Self)
    }
}

impl Display for DocType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Decode<'_, ()> for DocType {
    fn decode(d: &mut Decoder, _ctx: &mut ()) -> Result<Self, minicbor::decode::Error> {
        UuidV4::decode(d, &mut CborContext::Tagged)
            .map_err(|e| {
                minicbor::decode::Error::message(format!(
                    "DocType decoding Cannot decode single UUIDv4: {e}"
                ))
            })
            .map(Self)
    }
}

impl<C> Encode<C> for DocType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, &mut CborContext::Tagged)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::UuidV7;
    use minicbor::Encoder;
    use test_case::test_case;

    use super::*;

    #[test_case(
        {
            Encoder::new(Vec::new())
        } ;
        "Invalid empty CBOR bytes"
    )]
    #[test_case(
        {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(UuidV4::new(), &mut CborContext::Untagged).unwrap();
            e
        } ;
        "Invalid untagged uuid v4"
    )]
    #[test_case(
        {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(UuidV7::new(), &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Invalid tagged uuid v7"
    )]
    fn test_invalid_cbor_decode(e: Encoder<Vec<u8>>) {
        assert!(DocType::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut ()).is_err());
    }

    #[test_case(
        |uuid: UuidV4| {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(uuid, &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Valid uuid v4"
    )]
    fn test_valid_cbor_decode(e_gen: impl FnOnce(UuidV4) -> Encoder<Vec<u8>>) {
        let uuid = UuidV4::new();
        let e = e_gen(uuid);

        let doc_type =
            DocType::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut ()).unwrap();
        assert_eq!(doc_type.0, uuid);
    }

    #[test_case(
        |uuid: Uuid| { uuid.to_string() } ;
        "strings"
    )]
    #[test_case(
        |uuid: Uuid| { uuid } ;
        "Uuid"
    )]
    fn test_valid_try_from<T>(input_gen: impl FnOnce(Uuid) -> T)
    where DocType: TryFrom<T, Error = DocTypeError> {
        let uuid = Uuid::new_v4();
        let doc_type = DocType::try_from(input_gen(uuid)).unwrap();
        assert_eq!(doc_type.0.uuid(), uuid);
    }

    #[test_case(
        "not-a-uuid".to_string() => matches
        Err(DocTypeError::StringConversion(_)) ;         "Not a valid Uuid string"
    )]
    #[test_case(
        Uuid::now_v7() => matches Err(DocTypeError::InvalidUuid(_)) ;
        "Not a valid uuid v4"
    )]
    fn test_invalid_try_from<T>(input: T) -> Result<DocType, DocTypeError>
    where DocType: TryFrom<T, Error = DocTypeError> {
        DocType::try_from(input)
    }

    #[test_case(
        serde_json::json!(UuidV4::new()) ;
        "Document type old format"
    )]
    fn test_json_valid_serde(json: serde_json::Value) {
        let refs: DocType = serde_json::from_value(json).unwrap();
        let json_from_refs = serde_json::to_value(&refs).unwrap();
        assert_eq!(refs, serde_json::from_value(json_from_refs).unwrap());
    }
}
