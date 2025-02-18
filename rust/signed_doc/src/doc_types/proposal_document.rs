//! Proposal Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/proposal/#proposal-document>

use catalyst_types::uuid::Uuid;

/// Proposal document `UuidV4` type.
pub const PROPOSAL_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);
