//! Document references.

mod doc_locator;
mod doc_ref;
use std::{fmt::Display, ops::Deref};

use catalyst_types::uuid::{CborContext, UuidV7};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
pub use doc_locator::DocLocator;
pub use doc_ref::DocumentRef;
use minicbor::{Decode, Encode};
use tracing::warn;

use crate::CompatibilityPolicy;

/// List of document reference instance.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct DocumentRefs(Vec<DocumentRef>);

impl Deref for DocumentRefs {
    type Target = Vec<DocumentRef>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DocumentRefs {
    /// Returns true if provided `cbor` bytes is a valid old format.
    /// ```cddl
    /// old_format = [id, ver]
    /// ```
    /// Returns false if provided `cbor` bytes is a valid new format.
    /// ```cddl
    /// new_format = [ +[id, ver, cid] ]
    /// ```
    pub(crate) fn is_deprecated_cbor(cbor: &[u8]) -> Result<bool, minicbor::decode::Error> {
        let mut d = minicbor::Decoder::new(cbor);
        d.array()?;
        match d.datatype()? {
            // new_format = [ +[id, ver, cid] ]
            minicbor::data::Type::Array => Ok(false),
            // old_format = [id, ver]
            minicbor::data::Type::Tag => Ok(true),
            ty => Err(minicbor::decode::Error::type_mismatch(ty)),
        }
    }
}

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

impl From<Vec<DocumentRef>> for DocumentRefs {
    fn from(value: Vec<DocumentRef>) -> Self {
        DocumentRefs(value)
    }
}

impl Display for DocumentRefs {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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
        d: &mut minicbor::Decoder<'_>,
        policy: &mut CompatibilityPolicy,
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
                                    warn!(
                                        "{CONTEXT}: Conversion of document reference, id and version, to list of document reference with doc locator"
                                    );
                                }
                                if rest.len() != 1 {
                                    return Err(minicbor::decode::Error::message(format!(
                                        "{CONTEXT}: Must have exactly 2 elements inside array for document reference id and document reference version, found {}",
                                        rest.len().overflowing_add(1).0
                                    )));
                                }

                                let id = UuidV7::decode(
                                    &mut minicbor::Decoder::new(first),
                                    &mut CborContext::Tagged,
                                )
                                .map_err(|e| e.with_message("Invalid ID UUIDv7"))?;
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
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
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

mod serde_impl {
    //! `serde::Deserialize` and `serde::Serialize` trait implementations

    use super::{DocumentRef, DocumentRefs};

    /// A struct to support deserializing for both the old and new version of `ref`.
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum DocRefSerde {
        /// Old structure of document reference.
        Old(DocumentRef),
        /// New structure of document reference.
        New(Vec<DocumentRef>),
    }

    impl serde::Serialize for DocumentRefs {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for DocumentRefs {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            match DocRefSerde::deserialize(deserializer)? {
                DocRefSerde::Old(v) => Ok(DocumentRefs(vec![v])),
                DocRefSerde::New(v) => Ok(DocumentRefs(v)),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use minicbor::{Decoder, Encoder};
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
    fn test_invalid_cbor_decode(
        mut policy: CompatibilityPolicy,
        e: Encoder<Vec<u8>>,
    ) {
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
        mut policy: CompatibilityPolicy,
        e_gen: impl FnOnce(UuidV7, DocLocator) -> Encoder<Vec<u8>>,
    ) {
        let uuid = UuidV7::new();
        let doc_loc = DocLocator::default();
        let e = e_gen(uuid, doc_loc.clone());

        let doc_refs =
            DocumentRefs::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy)
                .unwrap();
        assert_eq!(doc_refs.0, vec![DocumentRef::new(uuid, uuid, doc_loc)]);
    }

    #[test_case(
        serde_json::json!(
            {
                "id": UuidV7::new(),
                "ver": UuidV7::new(),
            }
        ) ;
        "Document reference type old format"
    )]
    #[test_case(
        serde_json::json!(
            [
                {
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "cid": format!("0x{}", hex::encode([1, 2, 3]))
                },
                {
                    "id": UuidV7::new(),
                    "ver": UuidV7::new(),
                    "cid": format!("0x{}", hex::encode([1, 2, 3]))
                }
            ]
        ) ;
        "Document reference type new format"
    )]
    fn test_json_valid_serde(json: serde_json::Value) {
        let refs: DocumentRefs = serde_json::from_value(json).unwrap();
        let json_from_refs = serde_json::to_value(&refs).unwrap();
        assert_eq!(refs, serde_json::from_value(json_from_refs).unwrap());
    }
}
