//! CBOR utility modules.

pub mod array;
pub mod decode_context;
pub mod decode_helper;
pub mod deterministic_helper;
pub mod map;
pub mod with_cbor_bytes;

/// CBOR tag for BLAKE3 HASH
/// <https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml>
pub const BLAKE3_CBOR_TAG: u64 = 32781;
