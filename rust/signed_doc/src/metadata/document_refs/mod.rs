//! Document references.

pub(crate) mod doc_locator;
mod doc_ref;
use std::{fmt::Display, ops::Deref};

use cbork_utils::{array::Array, decode_context::DecodeCtx};
pub use doc_locator::DocLocator;
pub use doc_ref::DocumentRef;
use minicbor::{Decode, Encode};

use crate::CompatibilityPolicy;

/// List of document reference instance.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Deserialize, serde::Serialize)]
#[serde(from = "DocumentRefOrList")]
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
        _policy: &mut CompatibilityPolicy,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRefs decoding";

        // Old: [id, ver]
        // New: [ 1* [id, ver, locator] ]
        let outer_arr = Array::decode(d, &mut DecodeCtx::ArrayDeterministic)
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
                    // Old structure (id, ver) - no longer supported as DocLocator requires a valid
                    // CID
                    minicbor::data::Type::Tag => {
                        Err(minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Legacy document reference format (id, ver) without CID is no longer supported. \
                             DocLocator now requires a valid Content Identifier (CID)."
                        )))
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

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum DocumentRefOrList {
    Single(DocumentRef),
    Multiple(Vec<DocumentRef>),
}

// Convert the helper enum back into our desired struct
impl From<DocumentRefOrList> for DocumentRefs {
    fn from(value: DocumentRefOrList) -> Self {
        match value {
            DocumentRefOrList::Single(ref_item) => DocumentRefs(vec![ref_item]),
            DocumentRefOrList::Multiple(list) => DocumentRefs(list),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {

    use catalyst_types::uuid::{CborContext, UuidV7};
    use minicbor::{Decoder, Encoder};
    use test_case::test_case;

    use super::*;
    use crate::tests_utils::create_dummy_doc_ref;

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
            let doc_ref = create_dummy_doc_ref();
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(*doc_ref.id(), &mut CborContext::Untagged)
                .unwrap()
                .encode_with(*doc_ref.ver(), &mut CborContext::Untagged)
                .unwrap()
                .encode(doc_ref.doc_locator().clone())
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
        |id, ver, doc_loc| {
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(id, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(ver, &mut CborContext::Tagged)
                .unwrap()
                .encode(doc_loc)
                .unwrap();
            e
        } ;
        "Array of new doc ref (new format)"
    )]
    #[test_case(
        CompatibilityPolicy::Fail,
        |id, ver, doc_loc| {
            let mut e = Encoder::new(Vec::new());
            e.array(1)
                .unwrap()
                .array(3)
                .unwrap()
                .encode_with(id, &mut CborContext::Tagged)
                .unwrap()
                .encode_with(ver, &mut CborContext::Tagged)
                .unwrap()
                .encode(doc_loc)
                .unwrap();
            e
        } ;
        "Array of new doc ref (new format), fail policy"
    )]
    fn test_valid_cbor_decode(
        mut policy: CompatibilityPolicy,
        e_gen: impl FnOnce(UuidV7, UuidV7, DocLocator) -> Encoder<Vec<u8>>,
    ) {
        let doc_ref = create_dummy_doc_ref();
        let e = e_gen(*doc_ref.id(), *doc_ref.ver(), doc_ref.doc_locator().clone());

        let doc_refs =
            DocumentRefs::decode(&mut Decoder::new(e.into_writer().as_slice()), &mut policy)
                .unwrap();
        assert_eq!(doc_refs.0, vec![doc_ref]);
    }

    #[test]
    fn test_json_valid_serde() {
        let doc_ref1 = create_dummy_doc_ref();
        let doc_ref2 = create_dummy_doc_ref();

        let refs = DocumentRefs(vec![doc_ref1, doc_ref2]);

        let json = serde_json::to_value(&refs).unwrap();
        let refs_from_json: DocumentRefs = serde_json::from_value(json).unwrap();

        assert_eq!(refs, refs_from_json);
    }

    #[test]
    fn test_deterministic_decoding() {
        let mut refs = vec![create_dummy_doc_ref(), create_dummy_doc_ref()];
        refs.sort_by(|a, b| {
            let a_bytes = {
                let mut e = Encoder::new(Vec::new());
                a.encode(&mut e, &mut ()).unwrap();
                e.into_writer()
            };
            let b_bytes = {
                let mut e = Encoder::new(Vec::new());
                b.encode(&mut e, &mut ()).unwrap();
                e.into_writer()
            };

            match a_bytes.len().cmp(&b_bytes.len()) {
                std::cmp::Ordering::Equal => a_bytes.as_slice().cmp(&b_bytes),
                other => other,
            }
        });

        let mut e = Encoder::new(Vec::new());
        refs.encode(&mut e, &mut ()).unwrap();

        let result = DocumentRefs::decode(
            &mut Decoder::new(e.into_writer().as_slice()),
            &mut CompatibilityPolicy::Fail,
        );
        assert!(result.is_ok());

        let mut e = Encoder::new(Vec::new());
        refs.reverse();
        refs.encode(&mut e, &mut ()).unwrap();

        let result = DocumentRefs::decode(
            &mut Decoder::new(e.into_writer().as_slice()),
            &mut CompatibilityPolicy::Fail,
        );
        assert!(result.is_err());
    }
}
