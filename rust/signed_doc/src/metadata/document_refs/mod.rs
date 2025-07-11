//! Document references.

mod doc_locator;
mod doc_ref;
use std::{fmt::Display, str::FromStr};

use catalyst_types::uuid::{CborContext, UuidV7};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
pub use doc_locator::DocLocator;
pub use doc_ref::DocumentRef;
use minicbor::{Decode, Encode};
use serde::{Deserialize, Deserializer};
use tracing::warn;

use crate::CompatibilityPolicy;

/// List of document reference instance.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize)]
pub struct DocumentRefs(Vec<DocumentRef>);

/// Document reference error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DocRefError {
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

impl Decode<'_, CompatibilityPolicy> for DocumentRefs {
    fn decode(
        d: &mut minicbor::Decoder<'_>, policy: &mut CompatibilityPolicy,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRefs decoding";

        // Old: [id, ver]
        // New: [ 1* [id, ver, locator] ]
        let outer_arr = Array::decode(d, &mut DecodeCtx::Deterministic)
            .map_err(|e| minicbor::decode::Error::message(format!("{CONTEXT}: {e}")))?;

        match outer_arr.as_slice() {
            [first, rest @ ..] => {
                match minicbor::Decoder::new(first).datatype()? {
                    // New structure inner part [id, ver, locator]
                    minicbor::data::Type::Array => {
                        let mut arr = vec![first];
                        arr.extend(rest);

                        let doc_refs = arr
                            .iter()
                            .map(|bytes| minicbor::Decoder::new(bytes).decode())
                            .collect::<Result<_, _>>()?;

                        Ok(DocumentRefs(doc_refs))
                    },
                    // Old structure (id, ver)
                    minicbor::data::Type::Tag => {
                        match policy {
                            CompatibilityPolicy::Accept | CompatibilityPolicy::Warn => {
                                if matches!(policy, CompatibilityPolicy::Warn) {
                                    warn!("{CONTEXT}: Conversion of document reference, id and version, to list of document reference with doc locator");
                                }
                                if rest.len() != 1 {
                                    return Err(minicbor::decode::Error::message(format!(
                                        "{CONTEXT}: Must have extactly 2 elements inside array for document reference id and document reference version"
                                    )));
                                }

                                let id = UuidV7::decode(&mut minicbor::Decoder::new(first), &mut CborContext::Tagged).map_err(|e| {
                                    e.with_message("Invalid ID UUIDv7")
                                })?;
                                let ver = rest
                                    .first()
                                    .map(|ver| UuidV7::decode(&mut minicbor::Decoder::new(ver), &mut CborContext::Tagged).map_err(|e| {
                                        e.with_message("Invalid Ver UUIDv7")
                                    }))
                                    .transpose()?
                                    .ok_or_else(|| minicbor::decode::Error::message(format!("{CONTEXT}: Missing document reference version after document reference id")))?;

                                Ok(DocumentRefs(vec![DocumentRef::new(
                                    id,
                                    ver,
                                    // If old implementation is used, the locator will be empty
                                    DocLocator::default(),
                                )]))
                            },
                            CompatibilityPolicy::Fail => {
                                Err(minicbor::decode::Error::message(format!(
                                    "{CONTEXT}: Conversion of document reference id and version to list of document reference with doc locator is not allowed"
                                )))
                            },
                        }
                    },
                    other => {
                        Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Expected array of document reference, or tag of version and id, found {other}",
                        )))
                    },
                }
            },
            _ => {
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: Empty array",
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

    use minicbor::{Decoder, Encoder};
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
            e.array(2)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Tagged)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Tagged)
                .unwrap();
            e
        } ;
        "Valid array of two uuid v7 (old format), fail policy"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        {
            let mut e = Encoder::new(Vec::new());
            e.array(2)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Untagged)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Untagged)
                .unwrap();
            e
        } ;
        "Invalid untagged uuids v7 (old format)"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        {
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Untagged)
                .unwrap()
                .encode_with(UuidV7::new(), &mut CborContext::Untagged)
                .unwrap()
                .encode(DocLocator::default())
                .unwrap();
            e
        } ;
        "Invalid untagged uuid uuids v7 (new format)"
    )]
    fn test_invalid_cbor_decode(mut policy: CompatibilityPolicy, e: Encoder<Vec<u8>>) {
        assert!(
            DocumentRefs::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy)
                .is_err()
        );
    }

    #[test_case(
        CompatibilityPolicy::Accept,
        |uuid: UuidV7, _: DocLocator| {
            let mut e = Encoder::new(Vec::new());
            e.array(2)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap();
            e
        } ;
        "Valid single doc ref (old format)"
    )]
    #[test_case(
        CompatibilityPolicy::Warn,
        |uuid: UuidV7, _: DocLocator| {
            let mut e = Encoder::new(Vec::new());
            e.array(2)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap();
            e
        } ;
        "Valid single doc ref (old format), warn policy"
    )]
    #[test_case(
        CompatibilityPolicy::Accept,
        |uuid: UuidV7, doc_loc: DocLocator| {
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode(doc_loc)
                .unwrap();
            e
        } ;
        "Array of new doc ref (new format)"
    )]
    #[test_case(
        CompatibilityPolicy::Fail,
        |uuid: UuidV7, doc_loc: DocLocator| {
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(uuid, &mut CborContext::Tagged)
                .unwrap()
                .encode(doc_loc)
                .unwrap();
            e
        } ;
        "Array of new doc ref (new format), fail policy"
    )]
    fn test_valid_cbor_decode(
        mut policy: CompatibilityPolicy, e_gen: impl FnOnce(UuidV7, DocLocator) -> Encoder<Vec<u8>>,
    ) {
        let uuid = UuidV7::new();
        let doc_loc = DocLocator::default();
        let e = e_gen(uuid, doc_loc.clone());

        let doc_refs =
            DocumentRefs::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy)
                .unwrap();
        assert_eq!(doc_refs.0, vec![DocumentRef::new(uuid, uuid, doc_loc)]);
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
