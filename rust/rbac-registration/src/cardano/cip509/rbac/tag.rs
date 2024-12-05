//! Tags used in CIP-0509 besides from the original tags defined in minicbor.

use minicbor::data::Tag;

/// Enum of possible key tag which are use in public key and certificates.
pub(crate) enum KeyTag {
    /// Deleted Key tag 31.
    Deleted,
    /// Ed25519 Key tag 32773.
    Ed25519,
}

impl KeyTag {
    /// Get the tag value.
    pub(crate) fn tag(self) -> Tag {
        match self {
            KeyTag::Deleted => Tag::new(0x31),
            KeyTag::Ed25519 => Tag::new(0x8005),
        }
    }
}
