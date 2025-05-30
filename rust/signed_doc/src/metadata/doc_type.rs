//! Document Type.

use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

use catalyst_types::uuid::{CborContext, Uuid, UuidV4};
use minicbor::{Decode, Decoder, Encode};
use serde::{Deserialize, Deserializer};
use tracing::warn;

use crate::{
    decode_context::{CompatibilityPolicy, DecodeContext},
    doc_types::{
        ACTION_UUID_TYPE, COMMENT_UUID_TYPE, PROPOSAL_ACTION_DOC, PROPOSAL_COMMENT_DOC,
        PROPOSAL_DOC_TYPE, PROPOSAL_UUID_TYPE,
    },
};

/// List of `UUIDv4` document type.
#[derive(Clone, Debug, serde::Serialize, Eq)]
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
impl Decode<'_, DecodeContext<'_>> for DocType {
    fn decode(
        d: &mut Decoder, decode_context: &mut DecodeContext,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocType decoding";
        let parse_uuid = |d: &mut Decoder| UuidV4::decode(d, &mut CborContext::Tagged);

        match d.datatype()? {
            minicbor::data::Type::Array => {
                let len = d.array()?.ok_or_else(|| {
                    decode_context
                        .report
                        .other("Unable to decode array length", CONTEXT);
                    minicbor::decode::Error::message(format!(
                        "{CONTEXT}: Unable to decode array length"
                    ))
                })?;

                if len == 0 {
                    decode_context.report.invalid_value(
                        "array length",
                        "0",
                        "must contain at least one UUIDv4",
                        CONTEXT,
                    );
                    return Err(minicbor::decode::Error::message(format!(
                        "{CONTEXT}: empty array"
                    )));
                }

                (0..len)
                    .map(|_| parse_uuid(d))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Self)
                    .map_err(|e| {
                        decode_context
                            .report
                            .other(&format!("Invalid UUIDv4 in array: {e}"), CONTEXT);
                        minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Invalid UUIDv4 in array: {e}"
                        ))
                    })
            },
            minicbor::data::Type::Tag => {
                // Handle single tagged UUID
                match decode_context.compatibility_policy {
                    CompatibilityPolicy::Accept | CompatibilityPolicy::Warn => {
                        if matches!(
                            decode_context.compatibility_policy,
                            CompatibilityPolicy::Warn
                        ) {
                            warn!("{CONTEXT}: Conversion of document type single UUID to type DocType");
                        }

                        let uuid = parse_uuid(d).map_err(|e| {
                            let msg = format!("Cannot decode single UUIDv4: {e}");
                            decode_context.report.invalid_value(
                                "Decode single UUIDv4",
                                &e.to_string(),
                                &msg,
                                CONTEXT,
                            );
                            minicbor::decode::Error::message(format!("{CONTEXT}: {msg}"))
                        })?;

                        Ok(map_doc_type(uuid))
                    },

                    CompatibilityPolicy::Fail => {
                        let msg = "Conversion of document type single UUID to type DocType is not allowed";
                        decode_context.report.other(msg, CONTEXT);
                        Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: {msg}"
                        )))
                    },
                }
            },
            other => {
                decode_context.report.invalid_value(
                    "decoding type",
                    &format!("{other:?}"),
                    "array or tag cbor",
                    CONTEXT,
                );
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: expected array of UUIDor tagged UUIDv4, got {other:?}",
                )))
            },
        }
    }
}

/// Map single UUID doc type to new list of doc types
/// <https://github.com/input-output-hk/catalyst-libs/blob/main/docs/src/architecture/08_concepts/signed_doc/types.md#document-types>
fn map_doc_type(uuid: UuidV4) -> DocType {
    match uuid {
        id if Uuid::from(id) == PROPOSAL_UUID_TYPE => PROPOSAL_DOC_TYPE.clone(),
        id if Uuid::from(id) == COMMENT_UUID_TYPE => PROPOSAL_COMMENT_DOC.clone(),
        id if Uuid::from(id) == ACTION_UUID_TYPE => PROPOSAL_ACTION_DOC.clone(),
        id => DocType(vec![id]),
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

// This is needed to preserve backward compatibility with the old solution.
impl PartialEq for DocType {
    fn eq(&self, other: &Self) -> bool {
        // List of special-case (single UUID) -> new DocType
        // The old one should equal to the new one
        let special_cases = [
            (PROPOSAL_UUID_TYPE, &*PROPOSAL_DOC_TYPE),
            (COMMENT_UUID_TYPE, &*PROPOSAL_COMMENT_DOC),
            (ACTION_UUID_TYPE, &*PROPOSAL_ACTION_DOC),
        ];
        for (uuid, expected) in special_cases {
            match DocType::try_from(uuid) {
                Ok(single) => {
                    if (self.0 == single.0 && other.0 == expected.0)
                        || (other.0 == single.0 && self.0 == expected.0)
                    {
                        return true;
                    }
                },
                Err(_) => return false,
            }
        }
        self.0 == other.0
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::problem_report::ProblemReport;
    use minicbor::Encoder;
    use serde_json::json;

    use super::*;

    // <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>
    // Proposal Submission Action = 37(h'5e60e623ad024a1ba1ac406db978ee48') should map to
    // [37(h'5e60e623ad024a1ba1ac406db978ee48'), 37(h'7808d2bad51140af84e8c0d1625fdfdc'),
    // 37(h'78927329cfd94ea19c710e019b126a65')]
    const PSA: &str = "D825505E60E623AD024A1BA1AC406DB978EE48";

    #[test]
    fn test_empty_doc_type_cbor_decode() {
        assert!(<DocType as TryFrom<Vec<UuidV4>>>::try_from(vec![]).is_err());

        let mut report = ProblemReport::new("Test empty doc type");
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report,
        };
        let mut decoder = Decoder::new(&[]);
        assert!(DocType::decode(&mut decoder, &mut decoded_context).is_err());
    }

    #[test]
    fn test_single_uuid_doc_type_fail_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test single uuid doc type - fail");
        let data = hex::decode(PSA).unwrap();
        let decoder = Decoder::new(&data);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Fail,
            report: &mut report,
        };
        assert!(DocType::decode(&mut decoder.clone(), &mut decoded_context).is_err());
    }

    #[test]
    fn test_single_uuid_doc_type_warn_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test single uuid doc type - warn");
        let data = hex::decode(PSA).unwrap();
        let decoder = Decoder::new(&data);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Warn,
            report: &mut report,
        };
        let decoded_doc_type = DocType::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type.doc_types().len(), 3);
    }

    #[test]
    fn test_single_uuid_doc_type_accept_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test single uuid doc type - accept");
        let data = hex::decode(PSA).unwrap();
        let decoder = Decoder::new(&data);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report,
        };
        let decoded_doc_type = DocType::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type.doc_types().len(), 3);
    }

    #[test]
    fn test_multi_uuid_doc_type_cbor_decode_encode() {
        let uuidv4 = UuidV4::new();
        let mut report = ProblemReport::new("Test multi uuid doc type");
        let doc_type_list: DocType = vec![uuidv4, uuidv4].try_into().unwrap();
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        doc_type_list.encode(&mut encoder, &mut report).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report.clone(),
        };
        let decoded_doc_type = DocType::decode(&mut decoder, &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type, doc_type_list);
    }

    #[test]
    fn test_valid_vec_string() {
        let uuid = Uuid::new_v4().to_string();
        let input = vec![uuid.clone()];
        let doc_type = DocType::try_from(input).expect("should succeed");

        assert_eq!(doc_type.0.len(), 1);
        assert_eq!(doc_type.0.first().unwrap().to_string(), uuid);
    }

    #[test]
    fn test_empty_vec_string_fails() {
        let input: Vec<String> = vec![];
        let result = DocType::try_from(input);
        assert!(matches!(result, Err(DocTypeError::Empty)));
    }

    #[test]
    fn test_invalid_uuid_vec_string() {
        let input = vec!["not-a-uuid".to_string()];
        let result = DocType::try_from(input);
        assert!(matches!(result, Err(DocTypeError::StringConversion(s)) if s == "not-a-uuid"));
    }

    #[test]
    fn test_doctype_equal_special_cases() {
        // Direct equal
        let uuid: UuidV4 = PROPOSAL_UUID_TYPE.try_into().unwrap();
        let dt1 = DocType::try_from(vec![uuid]).unwrap();
        let dt2 = DocType::try_from(vec![uuid]).unwrap();
        assert_eq!(dt1, dt2);

        // single -> special mapped type
        let single = DocType::try_from(PROPOSAL_UUID_TYPE).unwrap();
        assert_eq!(single, *PROPOSAL_DOC_TYPE);
        let single = DocType::try_from(COMMENT_UUID_TYPE).unwrap();
        assert_eq!(single, *PROPOSAL_COMMENT_DOC);
        let single = DocType::try_from(ACTION_UUID_TYPE).unwrap();
        assert_eq!(single, *PROPOSAL_ACTION_DOC);
    }

    #[test]
    fn test_deserialize_single_uuid_normal() {
        let uuid = uuid::Uuid::new_v4().to_string();
        let json = json!(uuid);
        let dt: DocType = serde_json::from_value(json).unwrap();

        assert_eq!(dt.0.len(), 1);
        assert_eq!(dt.0.first().unwrap().to_string(), uuid);
    }

    #[test]
    fn test_deserialize_multiple_uuids() {
        let uuid1 = uuid::Uuid::new_v4().to_string();
        let uuid2 = uuid::Uuid::new_v4().to_string();
        let json = json!([uuid1.clone(), uuid2.clone()]);

        let dt: DocType = serde_json::from_value(json).unwrap();
        let actual =
            dt.0.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();
        assert_eq!(actual, vec![uuid1, uuid2]);
    }

    #[test]
    fn test_deserialize_special_case() {
        let uuid = PROPOSAL_UUID_TYPE.to_string();
        let json = json!(uuid);
        let dt: DocType = serde_json::from_value(json).unwrap();

        assert_eq!(dt, *PROPOSAL_DOC_TYPE);
    }
}
