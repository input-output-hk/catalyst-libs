//! Document Type.

use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

use catalyst_types::uuid::{CborContext, Uuid, UuidV4};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
use minicbor::{Decode, Decoder, Encode};
use serde::{Deserialize, Deserializer};
use tracing::warn;

use crate::{
    decode_context::CompatibilityPolicy,
    doc_types::{deprecated, PROPOSAL, PROPOSAL_COMMENT, PROPOSAL_SUBMISSION_ACTION},
};

/// List of `UUIDv4` document type.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct DocType(Vec<UuidV4>);

/// `DocType` Errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DocTypeError {
    /// Invalid UUID.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(Uuid),
    /// `DocType` cannot be empty.
    #[error("DocType cannot be empty")]
    Empty,
    /// Invalid string conversion
    #[error("Invalid string conversion: {0}")]
    StringConversion(String),
}

impl DocType {
    /// Get a list of `UUIDv4` document types.
    #[must_use]
    pub fn doc_types(&self) -> &Vec<UuidV4> {
        &self.0
    }
}

impl Hash for DocType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let list = self
            .0
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        list.hash(state);
    }
}

impl From<UuidV4> for DocType {
    fn from(value: UuidV4) -> Self {
        DocType(vec![value])
    }
}

impl TryFrom<Uuid> for DocType {
    type Error = DocTypeError;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        let uuid_v4 = UuidV4::try_from(value).map_err(|_| DocTypeError::InvalidUuid(value))?;
        Ok(DocType(vec![uuid_v4]))
    }
}

impl TryFrom<Vec<Uuid>> for DocType {
    type Error = DocTypeError;

    fn try_from(value: Vec<Uuid>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(DocTypeError::Empty);
        }

        let converted = value
            .into_iter()
            .map(|u| UuidV4::try_from(u).map_err(|_| DocTypeError::InvalidUuid(u)))
            .collect::<Result<Vec<UuidV4>, DocTypeError>>()?;

        DocType::try_from(converted)
    }
}

impl From<DocType> for Vec<Uuid> {
    fn from(value: DocType) -> Vec<Uuid> {
        value.0.into_iter().map(Uuid::from).collect()
    }
}

impl From<DocType> for Vec<String> {
    fn from(val: DocType) -> Self {
        val.0.into_iter().map(|uuid| uuid.to_string()).collect()
    }
}

impl TryFrom<Vec<UuidV4>> for DocType {
    type Error = DocTypeError;

    fn try_from(value: Vec<UuidV4>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(DocTypeError::Empty);
        }
        Ok(DocType(value))
    }
}

impl TryFrom<Vec<String>> for DocType {
    type Error = DocTypeError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(DocTypeError::Empty);
        }
        let converted = value
            .into_iter()
            .map(|s| {
                s.parse::<UuidV4>()
                    .map_err(|_| DocTypeError::StringConversion(s))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DocType(converted))
    }
}

impl Display for DocType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(UuidV4::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

// ; Document Type
// document_type = [ 1* uuid_v4 ]
// ; UUIDv4
// uuid_v4 = #6.37(bytes .size 16)
impl Decode<'_, CompatibilityPolicy> for DocType {
    fn decode(
        d: &mut Decoder, policy: &mut CompatibilityPolicy,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocType decoding";

        match d.datatype()? {
            minicbor::data::Type::Array => {
                let arr = Array::decode(d, &mut DecodeCtx::Deterministic)?;

                if arr.is_empty() {
                    return Err(minicbor::decode::Error::message(format!(
                        "{CONTEXT}: empty array"
                    )));
                }

                arr.into_iter()
                    .map(|uuid| {
                        UuidV4::decode(&mut minicbor::Decoder::new(&uuid), &mut CborContext::Tagged)
                    })
                    .collect::<Result<_, _>>()
                    .map(Self)
                    .map_err(|e| {
                        minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Invalid UUIDv4 in array: {e}"
                        ))
                    })
            },
            minicbor::data::Type::Tag => {
                // Handle single tagged UUID
                match policy {
                    CompatibilityPolicy::Accept | CompatibilityPolicy::Warn => {
                        if matches!(policy, CompatibilityPolicy::Warn) {
                            warn!("{CONTEXT}: Conversion of document type single UUID to type DocType");
                        }

                        let uuid = UuidV4::decode(d, &mut CborContext::Tagged).map_err(|e| {
                            minicbor::decode::Error::message(format!(
                                "{CONTEXT}: Cannot decode single UUIDv4: {e}"
                            ))
                        })?;

                        Ok(map_doc_type(uuid))
                    },

                    CompatibilityPolicy::Fail => {
                        Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Conversion of document type single UUID to type DocType is not allowed"
                        )))
                    },
                }
            },
            other => {
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: expected array of UUIDor tagged UUIDv4, got {other}",
                )))
            },
        }
    }
}

/// Map single UUID doc type to new list of doc types
/// <https://github.com/input-output-hk/catalyst-libs/blob/main/docs/src/architecture/08_concepts/signed_doc/types.md#document-types>
pub fn map_doc_type(uuid: UuidV4) -> DocType {
    match uuid {
        id if Uuid::from(id) == deprecated::PROPOSAL_DOCUMENT_UUID_TYPE => PROPOSAL.clone(),
        id if Uuid::from(id) == deprecated::COMMENT_DOCUMENT_UUID_TYPE => PROPOSAL_COMMENT.clone(),
        id if Uuid::from(id) == deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE => {
            PROPOSAL_SUBMISSION_ACTION.clone()
        },
        id => DocType(vec![id]),
    }
}

/// Maps `DocType` to the deprecated corresponding doc type.
#[allow(dead_code)]
pub fn to_deprecated_doc_type(doc_type: &DocType) -> Option<UuidV4> {
    if doc_type == &*PROPOSAL {
        UuidV4::try_from(deprecated::PROPOSAL_DOCUMENT_UUID_TYPE).ok()
    } else if doc_type == &*PROPOSAL_COMMENT {
        UuidV4::try_from(deprecated::COMMENT_DOCUMENT_UUID_TYPE).ok()
    } else if doc_type == &*PROPOSAL_SUBMISSION_ACTION {
        UuidV4::try_from(deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE).ok()
    } else {
        doc_type.0.first().cloned()
    }
}

impl<C> Encode<C> for DocType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(
            self.0
                .len()
                .try_into()
                .map_err(minicbor::encode::Error::message)?,
        )?;

        for id in &self.0 {
            id.encode(e, &mut CborContext::Tagged)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for DocType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DocTypeInput {
            /// Single UUID string.
            Single(String),
            /// List of UUID string.
            Multiple(Vec<String>),
        }

        let input = DocTypeInput::deserialize(deserializer)?;
        let dt = match input {
            DocTypeInput::Single(s) => {
                let uuid = s.parse().map_err(|_| {
                    serde::de::Error::custom(DocTypeError::StringConversion(s.clone()))
                })?;
                // If there is a map from old (single uuid) to new use that list, else convert that
                // single uuid to [uuid] - of type DocType
                map_doc_type(uuid)
            },
            DocTypeInput::Multiple(v) => v.try_into().map_err(serde::de::Error::custom)?,
        };
        Ok(dt)
    }
}

#[cfg(test)]
mod tests {
    use minicbor::Encoder;
    use serde_json::json;
    use test_case::test_case;

    use super::*;

    #[test_case(
        CompatibilityPolicy::Accept,
        {
            Encoder::new(Vec::new())
        } ;
        "Invalid empty CBOR bytes"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        {
            let mut e = Encoder::new(Vec::new());
            e.array(0).unwrap();
            e
        } ;
        "Invalid empty CBOR array"
    )]
    #[test_case(
        CompatibilityPolicy::Fail,
        {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(UuidV4::new(), &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Valid single uuid v4 (old format), fail policy"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(UuidV4::new(), &mut CborContext::Untagged).unwrap();
            e
        } ;
        "Invalid single untagged uuid v4 (old format)"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        {
            let mut e = Encoder::new(Vec::new());
            e.array(1).unwrap().encode_with(UuidV4::new(), &mut CborContext::Untagged).unwrap();
            e
        } ;
        "Invalid untagged uuid v4 array (new format)"
    )]
    fn test_invalid_cbor_decode(mut policy: CompatibilityPolicy, e: Encoder<Vec<u8>>) {
        assert!(
            DocType::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy).is_err()
        );
    }

    #[test_case(
        CompatibilityPolicy::Accept,
        |uuid: UuidV4| {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(uuid, &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Valid single uuid v4 (old format)"
    )]
    #[test_case(
        CompatibilityPolicy::Warn,
        |uuid: UuidV4| {
            let mut e = Encoder::new(Vec::new());
            e.encode_with(uuid, &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Valid single uuid v4 (old format), warn policy"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        |uuid: UuidV4| {
            let mut e = Encoder::new(Vec::new());
            e.array(1).unwrap().encode_with(uuid, &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Array of uuid v4 (new format)"
    )]
    #[test_case(
        CompatibilityPolicy::Fail,
        |uuid: UuidV4| {
            let mut e = Encoder::new(Vec::new());
            e.array(1).unwrap().encode_with(uuid, &mut CborContext::Tagged).unwrap();
            e
        } ;
        "Array of uuid v4 (new format), fail policy"
    )]
    fn test_valid_cbor_decode(
        mut policy: CompatibilityPolicy, e_gen: impl FnOnce(UuidV4) -> Encoder<Vec<u8>>,
    ) {
        let uuid = UuidV4::new();
        let e = e_gen(uuid);

        let doc_type =
            DocType::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy).unwrap();
        assert_eq!(doc_type.0, vec![uuid]);
    }

    #[test_case(
        |uuid: Uuid| { vec![uuid.to_string()] } ;
        "vec of strings"
    )]
    #[test_case(
        |uuid: Uuid| { vec![uuid] } ;
        "vec of uuid"
    )]
    #[test_case(
        |uuid: Uuid| { vec![UuidV4::try_from(uuid).unwrap()] } ;
        "vec of UuidV4"
    )]
    #[test_case(
        |uuid: Uuid| { uuid } ;
        "single uuid"
    )]
    fn test_valid_try_from<T>(input_gen: impl FnOnce(Uuid) -> T)
    where DocType: TryFrom<T, Error = DocTypeError> {
        let uuid = Uuid::new_v4();
        let doc_type = DocType::try_from(input_gen(uuid)).unwrap();
        assert_eq!(doc_type.0.len(), 1);
        assert_eq!(doc_type.0.first().unwrap().uuid(), uuid);
    }

    #[test_case(
        Vec::<String>::new() => matches Err(DocTypeError::Empty) ;
        "Empty string vec"
    )]
    #[test_case(
        Vec::<Uuid>::new() => matches Err(DocTypeError::Empty) ;
        "Empty Uuid vec"
    )]
    #[test_case(
        Vec::<UuidV4>::new() => matches Err(DocTypeError::Empty) ;
        "Empty UuidV4 vec"
    )]
    #[test_case(
        vec!["not-a-uuid".to_string()] => matches Err(DocTypeError::StringConversion(_)) ;
        "Not a valid Uuid string"
    )]
    #[test_case(
        vec![Uuid::now_v7()] => matches Err(DocTypeError::InvalidUuid(_)) ;
        "Not a valid vec of uuid v4"
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
        deprecated::PROPOSAL_DOCUMENT_UUID_TYPE => PROPOSAL.clone() ;
        "deprecated proposal document type"
    )]
    #[test_case(
        deprecated::COMMENT_DOCUMENT_UUID_TYPE => PROPOSAL_COMMENT.clone() ;
        "deprecated proposal comment document type"
    )]
    #[test_case(
        deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE => PROPOSAL_SUBMISSION_ACTION.clone() ;
        "deprecated proposal submission action type"
    )]
    fn test_compatibility_mapping(uuid: Uuid) -> DocType {
        let mut e = Encoder::new(Vec::new());
        e.encode_with(UuidV4::try_from(uuid).unwrap(), &mut CborContext::Tagged)
            .unwrap();

        // cbor decoding
        let cbor_doc_type = DocType::decode(
            &mut Decoder::new(e.into_writer().as_slice()),
            &mut CompatibilityPolicy::Accept,
        )
        .unwrap();

        // json decoding
        let json = json!(uuid);
        let json_doc_type = serde_json::from_value(json).unwrap();

        assert!(cbor_doc_type == json_doc_type);

        cbor_doc_type
    }

    #[test_case(
        serde_json::json!(UuidV4::new()) ;
        "Document type old format"
    )]
    #[test_case(
        serde_json::json!([UuidV4::new(), UuidV4::new()]) ;
        "Document type new format"
    )]
    fn test_json_valid_serde(json: serde_json::Value) {
        let refs: DocType = serde_json::from_value(json).unwrap();
        let json_from_refs = serde_json::to_value(&refs).unwrap();
        assert_eq!(refs, serde_json::from_value(json_from_refs).unwrap());
    }
}
