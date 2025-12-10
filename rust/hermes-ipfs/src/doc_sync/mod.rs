//! IPFS document synchronization module.

mod envelope;

pub mod payload;
pub mod timers;

pub use envelope::{Envelope, EnvelopePayload};

/// Current document synchronization protocol version.
const PROTOCOL_VERSION: u64 = 1;

/// `CID` version that Doc Sync supports.
const CID_VERSION: u8 = 1;

/// `CID` codec that Doc Sync supports (CBOR).
const CID_CODEC: u8 = 0x51;

/// `CID` multihash code that Doc Sync supports (SHA256).
const CID_MULTIHASH_CODE: u8 = 0x12;

/// `CID` multihash digest size that Doc Sync supports.
const CID_DIGEST_SIZE: u8 = 32;

/// Validates CID according to Doc Sync specification constraints.
fn validate_cid(cid: &crate::Cid) -> bool {
    cid.version() as u8 == CID_VERSION
        && cid.codec() == u64::from(CID_CODEC)
        && cid.hash().code() == u64::from(CID_MULTIHASH_CODE)
        && cid.hash().digest().len() == usize::from(CID_DIGEST_SIZE)
}
