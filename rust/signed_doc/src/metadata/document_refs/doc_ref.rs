//! Document reference.

use std::fmt::Display;

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, UuidV7},
};
use coset::cbor::Value;
use minicbor::{Decode, Decoder, Encode};

use super::{doc_locator::DocLocator, DocRefError};
use crate::{metadata::utils::CborUuidV7, DecodeContext};

/// Number of item that should be in each document reference instance.
const DOC_REF_ARR_ITEM: u64 = 3;

/// Reference to a Document.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize)]
pub struct DocumentRef {
    /// Reference to the Document Id
    id: UuidV7,
    /// Reference to the Document Ver
    ver: UuidV7,
    /// Document locator
    doc_locator: DocLocator,
}

impl DocumentRef {
    /// Create a new instance of document reference.
    #[must_use]
    pub fn new(id: UuidV7, ver: UuidV7, doc_locator: DocLocator) -> Self {
        Self {
            id,
            ver,
            doc_locator,
        }
    }

    /// Get Document Id.
    #[must_use]
    pub fn id(&self) -> &UuidV7 {
        &self.id
    }

    /// Get Document Ver.
    #[must_use]
    pub fn ver(&self) -> &UuidV7 {
        &self.ver
    }

    /// Get Document Locator.
    #[must_use]
    pub fn doc_locator(&self) -> &DocLocator {
        &self.doc_locator
    }
}

impl Display for DocumentRef {
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
        let id = Value::try_from(CborUuidV7(value.id))
            .map_err(|_| DocRefError::InvalidUuidV7(value.id, "id".to_string()))?;

        let ver = Value::try_from(CborUuidV7(value.ver))
            .map_err(|_| DocRefError::InvalidUuidV7(value.ver, "ver".to_string()))?;

        let locator = value.doc_locator.clone().into();

        Ok(Value::Array(vec![id, ver, locator]))
    }
}

impl Decode<'_, DecodeContext<'_>> for DocumentRef {
    fn decode(
        d: &mut minicbor::Decoder<'_>, decode_context: &mut DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRef decoding";
        let parse_uuid = |d: &mut Decoder| UuidV7::decode(d, &mut CborContext::Tagged);

        let arr = d.array()?.ok_or_else(|| {
            decode_context
                .report
                .other("Unable to decode array length", CONTEXT);
            minicbor::decode::Error::message(format!("{CONTEXT}: Unable to decode array length"))
        })?;
        if arr != DOC_REF_ARR_ITEM {
            decode_context.report.invalid_value(
                "Array length",
                &arr.to_string(),
                &DOC_REF_ARR_ITEM.to_string(),
                CONTEXT,
            );
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected {DOC_REF_ARR_ITEM} items, found {arr}"
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

        let locator = DocLocator::decode(d, decode_context.report).map_err(|e| {
            decode_context
                .report
                .other(&format!("Failed to decode locator {e}"), CONTEXT);
            e.with_message("Failed to decode locator")
        })?;

        Ok(DocumentRef {
            id,
            ver,
            doc_locator: locator,
        })
    }
}

impl Encode<ProblemReport> for DocumentRef {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, report: &mut ProblemReport,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        const CONTEXT: &str = "DocumentRef encoding";
        e.array(DOC_REF_ARR_ITEM)?;
        self.id.encode(e, &mut CborContext::Tagged).map_err(|_| {
            report.invalid_encoding("ID", &self.id.to_string(), "Valid UUIDv7", CONTEXT);
            minicbor::encode::Error::message(format!("{CONTEXT}: ID UUIDv7 encoding failed"))
        })?;
        self.ver.encode(e, &mut CborContext::Tagged).map_err(|_| {
            report.invalid_encoding("Ver", &self.id.to_string(), "Valid UUIDv7", CONTEXT);
            minicbor::encode::Error::message(format!("{CONTEXT}: Ver UUIDv7 encoding failed"))
        })?;
        self.doc_locator.encode(e, report).map_err(|e| {
            report.invalid_encoding(
                "Doc locator",
                &self.doc_locator.to_string(),
                "Valid doc locator",
                CONTEXT,
            );
            e.with_message("{CONTEXT}: Failed to encode doc locator")
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use catalyst_types::uuid::{UuidV7, UUID_CBOR_TAG};
    use coset::cbor::Value;

    use crate::{metadata::document_refs::doc_ref::DOC_REF_ARR_ITEM, DocumentRef};

    #[test]
    #[allow(clippy::indexing_slicing)]
    fn test_doc_refs_to_value() {
        let uuidv7 = UuidV7::new();
        let doc_ref = DocumentRef::new(uuidv7, uuidv7, vec![1, 2, 3].into());
        let value: Value = doc_ref.try_into().unwrap();
        let arr = value.into_array().unwrap();
        assert_eq!(arr.len(), usize::try_from(DOC_REF_ARR_ITEM).unwrap());
        let (id_tag, value) = arr[0].clone().into_tag().unwrap();
        assert_eq!(id_tag, UUID_CBOR_TAG);
        assert_eq!(value.as_bytes().unwrap().len(), 16);
        let (ver_tag, value) = arr[1].clone().into_tag().unwrap();
        assert_eq!(ver_tag, UUID_CBOR_TAG);
        assert_eq!(value.as_bytes().unwrap().len(), 16);
        let map = arr[2].clone().into_map().unwrap();
        assert_eq!(map.len(), 1);
    }
}
