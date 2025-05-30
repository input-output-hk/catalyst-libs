//! Catalyst Signed Document Metadata.

mod doc_locator;
use std::fmt::Display;

use catalyst_types::uuid::CborContext;
use coset::cbor::Value;
use minicbor::{Decode, Decoder};
use tracing::warn;

use crate::{CompatibilityPolicy, DecodeContext};

use super::{doc_locator::DocLocator, utils::CborUuidV7, UuidV7};

#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRef(Vec<DocumentRefInner>);

/// Reference to a Document.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRefInner {
    /// Reference to the Document Id
    id: UuidV7,
    /// Reference to the Document Ver
    ver: UuidV7,
    /// Document locator
    doc_locator: DocLocator,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DocRefError {
    /// Invalid UUIDv7.
    #[error("Invalid UUID: {0} for field {1}")]
    InvalidUuidV7(UuidV7, String),
    /// `DocRef` cannot be empty.
    #[error("DocType cannot be empty")]
    Empty,
}

impl Display for DocumentRefInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, ver: {}, document_locator: {}",
            self.id, self.ver, self.doc_locator
        )
    }
}

impl TryFrom<DocumentRef> for Value {
    type Error = DocRefError;

    fn try_from(value: DocumentRef) -> Result<Self, Self::Error> {
        if value.0.is_empty() {
            return Err(DocRefError::Empty);
        }

        let array_values: Result<Vec<Value>, Self::Error> = value
            .0
            .iter()
            .map(|inner| {
                let id = Value::try_from(CborUuidV7(inner.id.clone()))
                    .map_err(|_| DocRefError::InvalidUuidV7(inner.id.clone(), "id".to_string()))?;

                let ver = Value::try_from(CborUuidV7(inner.ver.clone())).map_err(|_| {
                    DocRefError::InvalidUuidV7(inner.ver.clone(), "ver".to_string())
                })?;

                let locator = Value::Bytes(inner.doc_locator.0.clone());

                Ok(Value::Array(vec![id, ver, locator]))
            })
            .collect();

        Ok(Value::Array(array_values?))
    }
}

impl Decode<'_, DecodeContext<'_>> for DocumentRef {
    fn decode(
        d: &mut minicbor::Decoder<'_>, decode_context: &mut DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRef decoding";
        let parse_uuid = |d: &mut Decoder| UuidV7::decode(d, &mut CborContext::Tagged);

        // Old: [id, ver]
        // New: [ 1* [document_id, document_ver,document_locator] ]
        let outer_arr = d.array()?.ok_or_else(|| {
            decode_context
                .report
                .invalid_value("array", "invalid array", "valid array", CONTEXT);
            minicbor::decode::Error::message(format!("{CONTEXT}: expected valid array"))
        })?;

        match d.datatype()? {
            // [id, ver, locator]
            minicbor::data::Type::Array => {
                let doc_ref = vec![];
                for _ in 0..outer_arr {
                    let inner_arr = d.array()?.ok_or_else(|| {
                        decode_context
                            .report
                            .other("Unable to decode inner array length", CONTEXT);
                        minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Unable to decode inner array length"
                        ))
                    })?;

                    if inner_arr != 3 {
                        decode_context.report.invalid_value(
                            "inner array length",
                            &inner_arr.to_string(),
                            "Expect 3 items",
                            CONTEXT,
                        );
                        return Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: expected 3 item in inner array, found {inner_arr}"
                        )));
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

                    let locator = DocLocator::decode(d, decode_context).map_err(|e| {
                        decode_context
                            .report
                            .other(&format!("Failed to decode locator {e}"), CONTEXT);
                        e.with_message("Failed to decode locator")
                    })?;

                    doc_ref.push(DocumentRefInner {
                        id,
                        ver,
                        doc_locator: locator,
                    });
                }
                Ok(DocumentRef(doc_ref))
            },
            // id, ver
            minicbor::data::Type::Tag => match decode_context.compatibility_policy {
                CompatibilityPolicy::Accept | CompatibilityPolicy::Warn => {
                    if matches!(
                        decode_context.compatibility_policy,
                        CompatibilityPolicy::Warn
                    ) {
                        warn!("{CONTEXT}: Conversion of document reference, id and version, to type list of document reference with doc locator");
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

                    Ok(DocumentRef(vec![DocumentRefInner {
                        id,
                        ver,
                        doc_locator: DocLocator::default(),
                    }]))
                },
                CompatibilityPolicy::Fail => {
                    let msg = "Conversion of document reference id and version to type list of document reference with doc locator is not allowed";
                    decode_context.report.other(msg, CONTEXT);
                    return Err(minicbor::decode::Error::message(format!(
                        "{CONTEXT}: {msg}"
                    )));
                },
            },
            other => {
                decode_context
                    .report
                    .invalid_value("decoding type", &other.to_string(), "array or tag", CONTEXT);
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: Expected array of document reference or tag of version and id, found {other}"
                )))
            },
        }
    }
}

// impl TryFrom<&Value> for DocumentRef {
//     type Error = anyhow::Error;

//     #[allow(clippy::indexing_slicing)]
//     fn try_from(val: &Value) -> anyhow::Result<DocumentRef> {
//         // The old value is single uuid
//         // The version and id are the same, the locator will be empty

//         // or array of id and ver
//         let Some(array) = val.as_array() else {
//             anyhow::bail!("Document Reference must be either a single UUID or an array of two");
//         };
//         anyhow::ensure!(
//             array.len() == 2,
//             "Document Reference array of two UUIDs was expected"
//         );
//         let CborUuidV7(id) = CborUuidV7::try_from(&array[0])?;
//         let CborUuidV7(ver) = CborUuidV7::try_from(&array[1])?;
//         anyhow::ensure!(
//             ver >= id,
//             "Document Reference Version can never be smaller than its ID"
//         );
//         Ok(DocumentRef {
//             id,
//             ver,
//             doc_locator: 0,
//         })
//     }
// }
