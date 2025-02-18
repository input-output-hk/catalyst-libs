//! Comment Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/comment/#comment-document>

use catalyst_types::uuid::Uuid;

/// Comment document `UuidV4` type.
pub const COMMENT_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA);
