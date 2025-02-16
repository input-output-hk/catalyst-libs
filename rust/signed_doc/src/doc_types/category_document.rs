//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::uuid::Uuid;

/// Category document `UuidV4` type.
pub const CATEGORY_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x48C2_0109_362A_4D32_9BBA_E0A9_CF8B_45BE);
