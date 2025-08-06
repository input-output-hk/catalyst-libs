//! Document reference.

use std::fmt::Display;

use catalyst_types::uuid::{CborContext, UuidV7};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
use minicbor::{Decode, Encode};

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
    pub fn new(
        id: UuidV7,
        ver: UuidV7,
        doc_locator: DocLocator,
    ) -> Self {
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
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "id: {}, ver: {}, document_locator: {}",
            self.id, self.ver, self.doc_locator
        )
    }
}

impl Decode<'_, ()> for DocumentRef {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocumentRef decoding";

        let arr = Array::decode(d, &mut DecodeCtx::Deterministic)
            .map_err(|e| minicbor::decode::Error::message(format!("{CONTEXT}: {e}")))?;

        let doc_ref = match arr.as_slice() {
            [id_bytes, ver_bytes, locator_bytes] => {
                let id = UuidV7::decode(
                    &mut minicbor::Decoder::new(id_bytes.as_slice()),
                    &mut CborContext::Tagged,
                )
                .map_err(|e| e.with_message("Invalid ID UUIDv7"))?;

                let ver = UuidV7::decode(
                    &mut minicbor::Decoder::new(ver_bytes.as_slice()),
                    &mut CborContext::Tagged,
                )
                .map_err(|e| e.with_message("Invalid Ver UUIDv7"))?;

                let doc_locator = minicbor::Decoder::new(locator_bytes.as_slice())
                    .decode()
                    .map_err(|e| e.with_message("Failed to decode locator"))?;

                DocumentRef {
                    id,
                    ver,
                    doc_locator,
                }
            },
            _ => {
                return Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: expected {DOC_REF_ARR_ITEM} items, found {}",
                    arr.len()
                )));
            },
        };

        Ok(doc_ref)
    }
}

impl Encode<()> for DocumentRef {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(DOC_REF_ARR_ITEM)?;
        self.id.encode(e, &mut CborContext::Tagged)?;
        self.ver.encode(e, &mut CborContext::Tagged)?;
        self.doc_locator.encode(e, ctx)?;
        Ok(())
    }
}
