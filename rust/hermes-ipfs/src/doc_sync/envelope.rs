//! Document synchronization envelope module.

use ed25519_dalek::VerifyingKey;
use minicbor::{Decode, Encode, Encoder, encode::Write};
use uuid::Timestamp;

use crate::doc_sync::PROTOCOL_VERSION;

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

/// New type wrapper.
struct UUIDv7(uuid::Uuid);

impl UUIDv7 {
    /// Create a new version 7 UUID using a time value and random bytes.
    fn new(timestamp: Timestamp) -> Self {
        Self(uuid::Uuid::new_v7(timestamp))
    }

    /// Create a new version 7 UUID using the current time value.
    fn now() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl<C> Encode<C> for UUIDv7 {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.bytes(self.0.as_bytes())?;
        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for UUIDv7 {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        uuid::Uuid::from_slice(d.bytes()?)
            .map_err(|err| {
                minicbor::decode::Error::message(format!("error during UUIDv7 decode: {err}"))
            })
            .map(UUIDv7)
    }
}

/// The unsigned portion of the message envelope.
/// This structure corresponds to the **signature input** array:
/// `[peer, seq, ver, payload]`.
///
/// The entire array is deterministically CBOR encoded and then signed to create the final
/// `signed-payload`.
#[derive(Encode, Decode)]
#[cbor(array)]
pub struct EnvelopePayload {
    /// Matches sender's Peer ID in IPFS Network
    /// Peer ID can be derived from this public key
    #[n(0)]
    peer: PublicKey,
    /// Unique nonce and timestamp
    /// Prevents and helps detect message duplication
    #[n(1)]
    seq: UUIDv7,
    /// Protocol version number
    /// This should be `1` for the current specification.
    #[n(2)]
    ver: minicbor::data::Int,
    /// This is an inner, deterministically-encoded CBOR map (`payload-body`).
    #[n(3)]
    payload: Vec<u8>,
}

impl EnvelopePayload {
    /// Create new instance of `EnvelopePayload`.
    #[must_use]
    pub fn new(
        peer: ed25519_dalek::VerifyingKey,
        seq: Timestamp,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            peer: PublicKey(peer),
            seq: UUIDv7::new(seq),
            ver: minicbor::data::Int::from(PROTOCOL_VERSION),
            payload,
        }
    }

    /// Create new instance of `EnvelopePayload` using the current time value.
    #[must_use]
    pub fn now(
        peer: ed25519_dalek::VerifyingKey,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            peer: PublicKey(peer),
            seq: UUIDv7::now(),
            ver: minicbor::data::Int::from(PROTOCOL_VERSION),
            payload,
        }
    }

    /// Returns cbor bstr for `EnvelopePayload` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `EnvelopePayload` failed to encode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, minicbor::encode::Error<std::convert::Infallible>> {
        minicbor::to_vec(self)
    }
}

/// The final outer message structure.
/// This structure represents the `signed-payload` array with the signature appended,
/// then wrapped in an outer `bstr` for framing:
/// `[peer, seq, ver, payload, signature_bstr]`.
pub struct Envelope {
    /// The inner payload array: `[peer, seq, ver, payload]`.
    /// This is the exact content that is deterministically CBOR encoded and signed.
    payload: EnvelopePayload,
    /// This is the signature computed over the deterministic CBOR bytes of
    /// `self.payload`.
    signature: Signature,
}

impl Envelope {
    /// Creates new doc sync envelope.
    ///
    /// Performs signature validation (step 1 of verification) as per spec.
    ///
    /// # Arguments
    ///
    /// * `payload` - The unsigned `EnvelopePayload`.
    /// * `signature` - `ed25519_dalek::Signature` of provided payload's deterministic
    ///   bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if signature is invalid.
    pub fn new(
        payload: EnvelopePayload,
        signature: ed25519_dalek::Signature,
    ) -> anyhow::Result<Self> {
        payload
            .peer
            .0
            .verify_strict(&payload.to_bytes()?, &signature)?;
        Ok(Self {
            payload,
            signature: Signature(signature),
        })
    }

    /// Returns cbor bstr for `Envelope` instance.
    ///
    /// The final output is the `envelope` defined in the spec:
    /// `envelope = bstr .size (82..1048576) .cbor signed-payload`.
    ///
    /// # Errors
    ///
    /// Returns an error if `Envelope` failed to encode.
    pub fn to_bytes(&self) -> Result<Vec<u8>, minicbor::encode::Error<std::convert::Infallible>> {
        minicbor::to_vec(self)
    }
}

impl<C> Encode<C> for Envelope {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(5)?;

        e.encode(&self.payload.peer)?
            .encode(&self.payload.seq)?
            .encode(self.payload.ver)?
            .encode(&self.payload.payload)?;

        e.encode(&self.signature)?;

        Ok(())
    }
}

impl<'b, C> Decode<'b, C> for Envelope {
    fn decode(
        d: &mut minicbor::Decoder<'b>,
        ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        d.array()?;

        let peer: PublicKey = d.decode_with(ctx)?;
        let seq: UUIDv7 = d.decode_with(ctx)?;
        let ver_int = d.int()?;
        let payload_bytes = d.bytes()?.to_vec();

        let inner_payload = EnvelopePayload {
            peer,
            seq,
            ver: ver_int,
            payload: payload_bytes,
        };

        let signature: Signature = d.decode_with(ctx)?;

        Ok(Self {
            payload: inner_payload,
            signature,
        })
    }
}
