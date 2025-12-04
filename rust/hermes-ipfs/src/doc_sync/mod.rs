//! IPFS document synchronization module.

mod envelope;

pub mod payload;

pub use envelope::{Envelope, EnvelopePayload};

/// Current document synchronization protocol version.
const PROTOCOL_VERSION: u64 = 1;
