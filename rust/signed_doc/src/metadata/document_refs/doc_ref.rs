//! Document reference.

use std::fmt::Display;

use catalyst_types::uuid::{CborContext, UuidV7};
use minicbor::{Decode, Decoder, Encode};

use super::doc_locator::DocLocator;

/// Number of item that should be in each document reference instance.
const DOC_REF_ARR_ITEM: u64 = 3;

/// Reference to a Document.
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
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

impl Decode<'_, ()> for DocumentRef {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRef decoding";
        let parse_uuid = |d: &mut Decoder| UuidV7::decode(d, &mut CborContext::Tagged);

        let arr = d.array()?.ok_or_else(|| {
            minicbor::decode::Error::message(format!("{CONTEXT}: Unable to decode array length"))
        })?;
        if arr != DOC_REF_ARR_ITEM {
            return Err(minicbor::decode::Error::message(format!(
                "{CONTEXT}: expected {DOC_REF_ARR_ITEM} items, found {arr}"
            )));
        }
        let id = parse_uuid(d).map_err(|e| e.with_message("Invalid ID UUIDv7"))?;

        let ver = parse_uuid(d).map_err(|e| e.with_message("Invalid Ver UUIDv7"))?;

        let locator = d
            .decode::<DocLocator>()
            .map_err(|e| e.with_message("Failed to decode locator"))?;

        Ok(DocumentRef {
            id,
            ver,
            doc_locator: locator,
        })
    }
}

impl Encode<()> for DocumentRef {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(DOC_REF_ARR_ITEM)?;
        self.id.encode(e, &mut CborContext::Tagged)?;
        self.ver.encode(e, &mut CborContext::Tagged)?;
        self.doc_locator.encode(e, ctx)?;
        Ok(())
    }
}
