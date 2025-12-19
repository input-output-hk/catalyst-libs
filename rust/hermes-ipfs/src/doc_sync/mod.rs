//! IPFS document synchronization module.

mod envelope;
mod state_machine;

pub mod payload;
pub mod timers;

use ed25519_dalek::VerifyingKey;
pub use envelope::{Envelope, EnvelopePayload};
use minicbor::{Decode, Encode, Encoder, encode::Write};
pub use state_machine::{StateMachine, StateSnapshot, SyncState};

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

/// Ed25519 public key instance.
/// Wrapper over `ed25519_dalek::VerifyingKey`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PublicKey(VerifyingKey);

impl<C> Encode<C> for PublicKey {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(self.0.as_bytes())?;
        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for PublicKey {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        VerifyingKey::try_from(d.bytes()?)
            .map_err(|err| {
                minicbor::decode::Error::message(format!("error during PublicKey decode: {err}"))
            })
            .map(PublicKey)
    }
}

/// Ed25519 signature instance.
/// Wrapper over `ed25519_dalek::Signature`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Signature(ed25519_dalek::Signature);

impl<C> Encode<C> for Signature {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(&self.0.to_bytes())?;
        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for Signature {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        ed25519_dalek::Signature::try_from(d.bytes()?)
            .map_err(|err| {
                minicbor::decode::Error::message(format!("error during Signature decode: {err}"))
            })
            .map(Signature)
    }
}
