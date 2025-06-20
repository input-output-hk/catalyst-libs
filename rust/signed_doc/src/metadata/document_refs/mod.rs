//! Document references.

mod doc_locator;
mod doc_ref;
use std::{fmt::Display, str::FromStr};

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, UuidV7},
};
use coset::cbor::Value;
pub use doc_locator::DocLocator;
pub use doc_ref::DocumentRef;
use minicbor::{Decode, Decoder, Encode};
use serde::{Deserialize, Deserializer};
use tracing::warn;

use crate::{CompatibilityPolicy, DecodeContext};

/// List of document reference instance.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize)]
pub struct DocumentRefs(Vec<DocumentRef>);

/// Document reference error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DocRefError {
    /// Invalid `UUIDv7`.
    #[error("Invalid UUID: {0} for field {1}")]
    InvalidUuidV7(UuidV7, String),
    /// `DocRef` cannot be empty.
    #[error("DocType cannot be empty")]
    Empty,
    /// Invalid string conversion
    #[error("Invalid string conversion: {0}")]
    StringConversion(String),
    /// Cannot decode hex.
    #[error("Cannot decode hex: {0}")]
    HexDecode(String),
}

impl DocumentRefs {
    /// Get a list of document reference instance.
    #[must_use]
    pub fn doc_refs(&self) -> &Vec<DocumentRef> {
        &self.0
    }
}

impl From<Vec<DocumentRef>> for DocumentRefs {
    fn from(value: Vec<DocumentRef>) -> Self {
        DocumentRefs(value)
    }
}

impl Display for DocumentRefs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = self
            .0
            .iter()
            .map(|inner| format!("{inner}"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "[{items}]")
    }
}

impl Decode<'_, DecodeContext<'_>> for DocumentRefs {
    fn decode(
        d: &mut minicbor::Decoder<'_>, decode_context: &mut DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRefs decoding";
        let parse_uuid = |d: &mut Decoder| UuidV7::decode(d, &mut CborContext::Tagged);

        // Old: [id, ver]
        // New: [ 1* [id, ver, locator] ]
        let outer_arr = d.array()?.ok_or_else(|| {
            decode_context.report.invalid_value(
                "Array",
                "Invalid array length",
                "Valid array length",
                CONTEXT,
            );
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid array length"))
        })?;

        match d.datatype()? {
            // New structure inner part [id, ver, locator]
            minicbor::data::Type::Array => {
                let mut doc_refs = vec![];
                for _ in 0..outer_arr {
                    let doc_ref = DocumentRef::decode(d, decode_context)?;
                    doc_refs.push(doc_ref);
                }
                Ok(DocumentRefs(doc_refs))
            },
            // Old structure [id, ver]
            minicbor::data::Type::Tag => {
                match decode_context.compatibility_policy {
                    CompatibilityPolicy::Accept | CompatibilityPolicy::Warn => {
                        if matches!(
                            decode_context.compatibility_policy,
                            CompatibilityPolicy::Warn
                        ) {
                            warn!("{CONTEXT}: Conversion of document reference, id and version, to list of document reference with doc locator");
                        }
                        let id = parse_uuid(d).map_err(|e| {
                            decode_context
                                .report
                                .other(&format!("Invalid ID UUIDv7: {e}"), CONTEXT);
                            e.with_message("Invalid ID UUIDv7")
                        })?;
                        let ver = parse_uuid(d).map_err(|e| {
                            decode_context
                                .report
                                .other(&format!("Invalid Ver UUIDv7: {e}"), CONTEXT);
                            e.with_message("Invalid Ver UUIDv7")
                        })?;

                        Ok(DocumentRefs(vec![DocumentRef::new(
                            id,
                            ver,
                            // If old implementation is used, the locator will be empty
                            DocLocator::default(),
                        )]))
                    },
                    CompatibilityPolicy::Fail => {
                        let msg = "Conversion of document reference id and version to list of document reference with doc locator is not allowed";
                        decode_context.report.other(msg, CONTEXT);
                        Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: {msg}"
                        )))
                    },
                }
            },
            other => {
                decode_context.report.invalid_value(
                    "Decoding type",
                    &other.to_string(),
                    "Array or tag",
                    CONTEXT,
                );
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: Expected array of document reference, or tag of version and id, found {other}"
                )))
            },
        }
    }
}

impl Encode<()> for DocumentRefs {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        const CONTEXT: &str = "DocumentRefs encoding";
        if self.0.is_empty() {
            return Err(minicbor::encode::Error::message(format!(
                "{CONTEXT}: DocumentRefs cannot be empty"
            )));
        }
        e.array(
            self.0
                .len()
                .try_into()
                .map_err(|e| minicbor::encode::Error::message(format!("{CONTEXT}, {e}")))?,
        )?;

        for doc_ref in &self.0 {
            doc_ref.encode(e, ctx)?;
        }
        Ok(())
    }
}

impl TryFrom<DocumentRefs> for Value {
    type Error = DocRefError;

    fn try_from(value: DocumentRefs) -> Result<Self, Self::Error> {
        if value.0.is_empty() {
            return Err(DocRefError::Empty);
        }

        let array_values: Result<Vec<Value>, Self::Error> = value
            .0
            .iter()
            .map(|inner| Value::try_from(inner.to_owned()))
            .collect();

        Ok(Value::Array(array_values?))
    }
}

impl TryFrom<&DocumentRefs> for Value {
    type Error = DocRefError;

    fn try_from(value: &DocumentRefs) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl<'de> Deserialize<'de> for DocumentRefs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        /// Old structure deserialize as map {id, ver}
        #[derive(Deserialize)]
        struct OldRef {
            /// "id": "uuidv7
            id: String,
            /// "ver": "uuidv7"
            ver: String,
        }

        /// New structure as deserialize as map {id, ver, cid}
        #[derive(Deserialize)]
        struct NewRef {
            /// "id": "uuidv7"
            id: String,
            /// "ver": "uuidv7"
            ver: String,
            /// "cid": "0x..."
            cid: String,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DocRefInput {
            /// Old structure of document reference.
            Old(OldRef),
            /// New structure of document reference.
            New(Vec<NewRef>),
        }

        let input = DocRefInput::deserialize(deserializer)?;
        let dr = match input {
            DocRefInput::Old(value) => {
                let id = UuidV7::from_str(&value.id).map_err(|_| {
                    serde::de::Error::custom(DocRefError::StringConversion(value.id.clone()))
                })?;
                let ver = UuidV7::from_str(&value.ver).map_err(|_| {
                    serde::de::Error::custom(DocRefError::StringConversion(value.ver.clone()))
                })?;

                DocumentRefs(vec![DocumentRef::new(id, ver, DocLocator::default())])
            },
            DocRefInput::New(value) => {
                let mut dr = vec![];
                for v in value {
                    let id = UuidV7::from_str(&v.id).map_err(|_| {
                        serde::de::Error::custom(DocRefError::StringConversion(v.id.clone()))
                    })?;
                    let ver = UuidV7::from_str(&v.ver).map_err(|_| {
                        serde::de::Error::custom(DocRefError::StringConversion(v.ver.clone()))
                    })?;
                    let cid = &v.cid.strip_prefix("0x").unwrap_or(&v.cid);
                    let locator = hex::decode(cid).map_err(|_| {
                        serde::de::Error::custom(DocRefError::HexDecode(v.cid.clone()))
                    })?;
                    dr.push(DocumentRef::new(id, ver, locator.into()));
                }
                DocumentRefs(dr)
            },
        };

        Ok(dr)
    }
}

#[cfg(test)]
mod tests {

    use minicbor::Encoder;
    use serde_json::json;

    use super::*;

    #[allow(clippy::unwrap_used)]
    fn gen_old_doc_ref(id: UuidV7, ver: UuidV7) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        encoder.array(2).unwrap();
        id.encode(&mut encoder, &mut CborContext::Tagged).unwrap();
        ver.encode(&mut encoder, &mut CborContext::Tagged).unwrap();
        buffer
    }

    #[test]
    fn test_old_doc_refs_fail_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test doc ref fail policy");
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Fail,
            report: &mut report,
        };
        let uuidv7 = UuidV7::new();
        let old_doc_ref = gen_old_doc_ref(uuidv7, uuidv7);
        let decoder = Decoder::new(&old_doc_ref);
        assert!(DocumentRefs::decode(&mut decoder.clone(), &mut decoded_context).is_err());
    }

    #[test]
    fn test_old_doc_refs_warn_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test doc ref warn policy");
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Warn,
            report: &mut report,
        };
        let uuidv7 = UuidV7::new();
        let old_doc_ref = gen_old_doc_ref(uuidv7, uuidv7);
        let decoder = Decoder::new(&old_doc_ref);
        let decoded_doc_ref =
            DocumentRefs::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_ref.doc_refs().len(), 1);
        assert_eq!(
            decoded_doc_ref
                .doc_refs()
                .first()
                .unwrap()
                .doc_locator()
                .len(),
            0
        );
    }

    #[test]
    fn test_old_doc_refs_accept_policy_cbor_decode() {
        let mut report = ProblemReport::new("Test doc ref accept policy");
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report,
        };
        let uuidv7 = UuidV7::new();
        let old_doc_ref = gen_old_doc_ref(uuidv7, uuidv7);
        let decoder = Decoder::new(&old_doc_ref);
        let decoded_doc_ref =
            DocumentRefs::decode(&mut decoder.clone(), &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_ref.doc_refs().len(), 1);
        assert_eq!(
            decoded_doc_ref
                .doc_refs()
                .first()
                .unwrap()
                .doc_locator()
                .len(),
            0
        );
    }

    #[test]
    fn test_doc_refs_cbor_encode_decode() {
        let mut report = ProblemReport::new("Test doc refs");

        let uuidv7 = UuidV7::new();
        let doc_ref = DocumentRef::new(uuidv7, uuidv7, vec![1, 2, 3, 4].into());
        let doc_refs = DocumentRefs(vec![doc_ref.clone(), doc_ref]);
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        doc_refs.encode(&mut encoder, &mut report).unwrap();
        let mut decoder = Decoder::new(&buffer);
        let mut decoded_context = DecodeContext {
            compatibility_policy: CompatibilityPolicy::Accept,
            report: &mut report.clone(),
        };
        let decoded_doc_refs = DocumentRefs::decode(&mut decoder, &mut decoded_context).unwrap();
        assert_eq!(decoded_doc_refs, doc_refs);
    }

    #[test]
    fn test_doc_refs_to_value() {
        let uuidv7 = UuidV7::new();
        let doc_ref = DocumentRef::new(uuidv7, uuidv7, vec![1, 2, 3].into());
        let doc_ref = DocumentRefs(vec![doc_ref.clone(), doc_ref]);
        let value: Value = doc_ref.try_into().unwrap();
        assert_eq!(value.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_deserialize_old_doc_ref() {
        let uuidv7 = UuidV7::new();
        let json = json!(
            {
                "id": uuidv7.to_string(),
                "ver": uuidv7.to_string(),
            }
        );
        let doc_ref: DocumentRefs = serde_json::from_value(json).unwrap();
        let dr = doc_ref.doc_refs().first().unwrap();
        assert_eq!(*dr.id(), uuidv7);
        assert_eq!(*dr.ver(), uuidv7);
        assert_eq!(dr.doc_locator().len(), 0);
    }

    #[test]
    fn test_deserialize_new_doc_ref() {
        let uuidv7 = UuidV7::new();
        let data = vec![1, 2, 3, 4];
        let hex_data = format!("0x{}", hex::encode(data.clone()));
        let json = json!(
            [{
                "id": uuidv7.to_string(),
                "ver": uuidv7.to_string(),
                "cid": hex_data,
            },
            {
                "id": uuidv7.to_string(),
                "ver": uuidv7.to_string(),
                "cid": hex_data,
            },
            ]
        );
        let doc_ref: DocumentRefs = serde_json::from_value(json).unwrap();
        assert!(doc_ref.doc_refs().len() == 2);
        let dr = doc_ref.doc_refs().first().unwrap();
        assert_eq!(*dr.id(), uuidv7);
        assert_eq!(*dr.ver(), uuidv7);
        assert_eq!(*dr.doc_locator(), data.into());
    }
}
