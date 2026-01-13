//! Document reference.

use std::fmt::Display;

use catalyst_types::uuid::{CborContext, UuidV7};
use cbork_utils::{array::Array, decode_context::DecodeCtx};
use minicbor::{Decode, Encode};

use super::doc_locator::DocLocator;

/// Number of item that should be in each document reference instance.
const DOC_REF_ARR_ITEM: u64 = 3;

/// Reference to a Document.
#[derive(Clone, Debug, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocumentRef {
    /// Reference to the Document Id
    id: UuidV7,
    /// Reference to the Document Ver
    ver: UuidV7,
    /// Document locator
    #[serde(rename = "cid")]
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

impl PartialOrd for DocumentRef {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DocumentRef {
    fn cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.ver.cmp(&other.ver))
            .then_with(|| self.doc_locator.cmp(&other.doc_locator))
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

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, str::FromStr};

    use test_case::test_case;

    use super::*;

    // spell:disable
    const CID_SMALL: &str = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
    const CID_LARGE: &str = "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
    // spell:enable

    #[test_case(
        (0,0, CID_SMALL),
        (0,0, CID_SMALL)
        => Ordering::Equal;
        "fully equal"
    )]
    #[test_case(
        (1,0, CID_SMALL),
        (0,0, CID_SMALL)
        => Ordering::Greater;
        "first 'id' greater"
    )]
    #[test_case(
        (0,1, CID_SMALL),
        (0,0, CID_SMALL)
        => Ordering::Greater;
        "first 'ver' greater"
    )]
    #[test_case(
        (0,0, CID_LARGE),
        (0,0, CID_SMALL)
        => Ordering::Greater;
        "first 'doc_locator' greater"
    )]
    // --- 3. MIRRORING TESTS (The "Less" Scenarios) ---
    // These test when the left side is smaller than the right side.
    #[test_case(
        (0,0, CID_SMALL),
        (1,0, CID_SMALL)
        => Ordering::Less;
        "first 'id' smaller"
    )]
    #[test_case(
        (0,0, CID_SMALL),
        (0,1, CID_SMALL)
        => Ordering::Less;
        "first 'ver' smaller"
    )]
    #[test_case(
        (0,0, CID_SMALL),
        (0,0, CID_LARGE)
        => Ordering::Less;
        "first 'doc_locator' smaller"
    )]
    // --- 4. PRIORITY / DOMINANCE TESTS (Crucial) ---
    // These prove that ID beats Ver, and Ver beats Locator.

    // Left has Greater ID (1 > 0), but Smaller Ver (0 < 5).
    // Result should be Greater because ID is checked first.
    #[test_case(
        (1, 0, CID_SMALL),
        (0, 5, CID_SMALL)
        => Ordering::Greater;
        "id dominates ver"
    )]
    // Left has Equal ID, Greater Ver (1 > 0), but Smaller Locator (a < b).
    // Result should be Greater because Ver is checked before Locator.
    #[test_case(
        (0, 1, CID_SMALL),
        (0, 0, CID_LARGE)
        => Ordering::Greater;
        "ver dominates doc_locator"
    )]
    fn ord_test(
        (a_id, a_ver, a_doc_loc_str): (u128, u128, &'static str),
        (b_id, b_ver, b_doc_loc_str): (u128, u128, &'static str),
    ) -> Ordering {
        let a = DocumentRef::new(
            uuid::Builder::from_u128(a_id)
                .with_version(uuid::Version::SortRand)
                .into_uuid()
                .try_into()
                .unwrap(),
            uuid::Builder::from_u128(a_ver)
                .with_version(uuid::Version::SortRand)
                .into_uuid()
                .try_into()
                .unwrap(),
            DocLocator::from_str(a_doc_loc_str).unwrap(),
        );
        let b = DocumentRef::new(
            uuid::Builder::from_u128(b_id)
                .with_version(uuid::Version::SortRand)
                .into_uuid()
                .try_into()
                .unwrap(),
            uuid::Builder::from_u128(b_ver)
                .with_version(uuid::Version::SortRand)
                .into_uuid()
                .try_into()
                .unwrap(),
            DocLocator::from_str(b_doc_loc_str).unwrap(),
        );
        a.cmp(&b)
    }
}
