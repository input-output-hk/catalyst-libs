//! `CatalystSignedDocumentProvider::try_find_doc` search query argument type
//! implementation.

use catalyst_types::catalyst_id::CatalystId;

use crate::{DocType, DocumentRef, DocumentRefs, uuid::UuidV7};

/// `CatalystSignedDocumentProvider::try_find_doc` search query argument type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalystSignedDocumentSearchQuery {
    /// 'id' metadata field search.
    pub id: Option<UuidV7Selector>,
    /// 'ver' metadata field search.
    pub ver: Option<UuidV7Selector>,
    /// 'type' metadata field search.
    /// Search `DocType` in the given list.
    pub doc_type: Vec<DocType>,
    /// `ref` metadata field search.
    pub doc_ref: Option<DocumentRefSelector>,
    /// `template` metadata field search.
    pub template: Option<DocumentRefSelector>,
    /// `reply` metadata field search.
    pub reply: Option<DocumentRefSelector>,
    /// `parameters` metadata field search.
    pub parameters: Option<DocumentRefSelector>,
    /// `collaborators` metadata field search.
    pub collaborators: Option<CatalystIdSelector>,
    /// `CatalystSignedDocument::authors` search.
    pub authors: Option<CatalystIdSelector>,
}

/// `UUIDv7` search selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UuidV7Selector {
    /// Search by the exact UUIDv7
    Eq(UuidV7),
    /// Search in this UUIDv7's range
    Range {
        /// Minimum UUIDv7 to find (inclusive)
        min: UuidV7,
        /// Maximum UUIDv7 to find (inclusive)
        max: UuidV7,
    },
    /// Search UUIDv7s in the given list.
    In(Vec<UuidV7>),
}

impl UuidV7Selector {
    /// Applying `UuidV7Selector` for the provided `UuidV7` value.
    pub fn filter(
        &self,
        uuid: &UuidV7,
    ) -> bool {
        match self {
            Self::Eq(eq) => uuid == eq,
            Self::Range { min, max } => uuid >= min && uuid <= max,
            Self::In(inclussion) => inclussion.contains(uuid),
        }
    }
}

/// `DocumentRef` search selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentRefSelector {
    /// Search by the exact `DocumentRefs`
    Eq(DocumentRefs),
    /// Search with `DocumentRef` in the given list.
    In(Vec<DocumentRef>),
}

impl DocumentRefSelector {
    /// Applying `DocumentRefSelector` for the provided `DocumentRefs` value.
    pub fn filter(
        &self,
        doc_refs: &DocumentRefs,
    ) -> bool {
        match self {
            Self::Eq(eq) => doc_refs == eq,
            Self::In(inclussion) => doc_refs.iter().any(|v| inclussion.contains(v)),
        }
    }
}

/// `CatalystId` search selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalystIdSelector {
    /// Search by the exact `Vec<CatalystId>`
    Eq(Vec<CatalystId>),
    /// Search with `CatalystId` in the given list.
    In(Vec<CatalystId>),
}

impl CatalystIdSelector {
    /// Applying `CatalystIdSelector` for the provided `Vec<CatalystId>` value.
    pub fn filter(
        &self,
        cat_ids: &[CatalystId],
    ) -> bool {
        match self {
            Self::Eq(eq) => cat_ids == eq,
            Self::In(inclussion) => cat_ids.iter().any(|v| inclussion.contains(v)),
        }
    }
}
