//! Document Type.

use std::fmt::{Display, Formatter};

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, Uuid, UuidV4},
};
use minicbor::{Decode, Decoder, Encode};
use tracing::warn;

use crate::{
    decode_context::{CompatibilityPolicy, DecodeContext},
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        PROPOSAL_DOCUMENT_UUID_TYPE, SUBMISSION_ACTION,
    },
};

/// List of `UUIDv4` document type.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
}

impl DocType {
    /// Get a list of `UUIDv4` document types.
    #[must_use]
    pub fn doc_types(&self) -> &Vec<UuidV4> {
        &self.0
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

                        let ids = map_doc_type(uuid.into()).map_err(|e| {
                            decode_context.report.other(&e.to_string(), CONTEXT);
                            minicbor::decode::Error::message(format!("{CONTEXT}: {e}"))
                        })?;

                        let doc_type = ids.to_vec().try_into().map_err(|e: DocTypeError| {
                            decode_context.report.other(&e.to_string(), CONTEXT);
                            minicbor::decode::Error::message(format!("{CONTEXT}: {e}"))
                        })?;

                        Ok(doc_type)
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
fn map_doc_type(uuid: Uuid) -> anyhow::Result<&'static [Uuid]> {
    const PROPOSAL_DOC: &[Uuid] = &[PROPOSAL_DOCUMENT_UUID_TYPE];
    const PROPOSAL_COMMENT_DOC: &[Uuid] =
        &[COMMENT_DOCUMENT_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE];
    const PROPOSAL_ACTION_DOC: &[Uuid] = &[
        PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        PROPOSAL_DOCUMENT_UUID_TYPE,
        SUBMISSION_ACTION,
    ];

    match uuid {
        id if id == PROPOSAL_DOCUMENT_UUID_TYPE => Ok(PROPOSAL_DOC),
        id if id == COMMENT_DOCUMENT_UUID_TYPE => Ok(PROPOSAL_COMMENT_DOC),
        id if id == PROPOSAL_ACTION_DOCUMENT_UUID_TYPE => Ok(PROPOSAL_ACTION_DOC),
        _ => anyhow::bail!("Unknown document type: {uuid}"),
    }
}

impl Encode<ProblemReport> for DocType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, report: &mut ProblemReport,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        const CONTEXT: &str = "DocType encoding";
        if self.0.is_empty() {
            report.invalid_value("DocType", "empty", "DocType cannot be empty", CONTEXT);
            return Err(minicbor::encode::Error::message(format!(
                "{CONTEXT}: DocType cannot be empty"
            )));
        }

        e.array(self.0.len().try_into().map_err(|_| {
            report.other("Unable to encode array length", CONTEXT);
            minicbor::encode::Error::message(format!("{CONTEXT}, unable to encode array length"))
        })?)?;

        for id in &self.0 {
            id.encode(e, &mut CborContext::Tagged).map_err(|_| {
                report.other("Failed to encode UUIDv4", CONTEXT);
                minicbor::encode::Error::message(format!("{CONTEXT}: UUIDv4 encoding failed"))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use minicbor::Encoder;

    use super::*;

    #[test]
    fn test_empty_doc_type() {
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
    fn test_single_uuid_doc_type() {
        // <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>
        // Action = 37(h'5e60e623ad024a1ba1ac406db978ee48')
        let proposal_uuid = hex::decode("D825505E60E623AD024A1BA1AC406DB978EE48").unwrap();
        let mut report = ProblemReport::new("Test single uuid doc type");
        let decoder = Decoder::new(&proposal_uuid);
        // Failing Policy
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Fail,
            report: &mut report,
        };
        assert!(DocType::decode(&mut decoder.clone(), &mut decoded_context).is_err());
        // Warning Policy
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Warn,
            report: &mut report,
        };
        let decoded_doc_type = DocType::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type.doc_types().len(), 3);
        // Accept Policy
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report,
        };
        let decoded_doc_type = DocType::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type.doc_types().len(), 3);
    }

    #[test]
    fn test_multi_uuid_doc_type() {
        let uuidv4 = UuidV4::new();
        let mut report = ProblemReport::new("Test multi uuid doc type");
        let doc_type_list: DocType = vec![uuidv4, uuidv4].try_into().unwrap();
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        doc_type_list
            .encode(&mut encoder, &mut report)
            .expect("Failed to encode Doc Type");
        let mut decoder = Decoder::new(&buffer);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report.clone(),
        };
        let decoded_doc_type = DocType::decode(&mut decoder, &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_type, doc_type_list);
    }
}
