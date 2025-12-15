//! Doc Sync message payload for .syn topic
//! For more information, see <https://github.com/input-output-hk/hermes/blob/main/docs/src/architecture/08_concepts/document_sync/protocol_spec.md#syn-topic-basesyn>

use derive_more::Display;
use minicbor::{
    Decode, Decoder, Encode, Encoder, decode,
    encode::{self, Write},
};

/// Syn payload
#[derive(Decode, Encode, derive_more::Deref, derive_more::From, derive_more::Into)]
#[cbor(transparent)]
pub struct Syn(#[n(0)] MsgSyn);

/// Maximum length of prefix array: 2^14.
const MAX_PREFIX_ARRAY_LENGTH: u64 = 16384;

/// If doc count exceeds this, prefix entries are required.
const MIN_DOC_COUNT_PREFIX_THRESHOLD: u64 = 64;

/// Numeric keys of the payload map.
#[derive(Copy, Clone, PartialEq, derive_more::TryFrom, Display)]
#[try_from(repr)]
#[repr(u8)]
enum SynNumericKeys {
    Root = 1,
    Count = 2,
    To = 3,
    Prefix = 4,
    PeerRoot = 5,
    PeerCount = 6,
}

impl<C> Encode<C> for SynNumericKeys {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.u8(*self as u8)?.ok()
    }
}

impl<C> Decode<'_, C> for SynNumericKeys {
    fn decode(
        d: &mut Decoder<'_>,
        _: &mut C,
    ) -> Result<Self, decode::Error> {
        d.u8().and_then(|repr| {
            Self::try_from(repr).map_err(|err| decode::Error::custom(err).at(d.position()))
        })
    }
}

/// Ed25519 public key type.
pub type Ed25519Pubkey = [u8; 32];
/// BLAKE3-256 hash type.
pub type Blake3256 = [u8; 32];
/// Prefix array type.
pub type PrefixArray = Vec<Blake3256>;

/// Payload for .syn topic.
#[derive(Clone, Default)]
pub struct MsgSyn {
    /// The root BLAKE3-256 hash of the requester’s SMT root.
    pub root: Blake3256,
    /// Current number of documents of the requester.
    pub count: u64,
    /// Target peer to respond.
    pub to: Ed25519Pubkey,
    /// Array of SMT node hashes at depth D left-to-right across the tree.
    /// The size of the array MUST be 2 to power of N (2,4,8...,16384).
    pub prefix: Option<PrefixArray>,
    /// Last observed SMT root (BLAKE3-256 hash) of the target peer.
    pub peer_root: Blake3256,
    /// Last observed document count of the target peer.
    pub peer_count: u64,
}

impl MsgSyn {
    /// The maximum number of fields for .syn payload
    const MAX_NUM_FIELDS: u64 = 6;

    /// Get the number of fields for .syn payload
    fn num_fields(&self) -> u64 {
        // Prefix is optional
        if self.prefix.is_none() {
            Self::MAX_NUM_FIELDS.saturating_sub(1)
        } else {
            Self::MAX_NUM_FIELDS
        }
    }
}

impl<C> Encode<C> for MsgSyn {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.map(self.num_fields())?;
        encode_blake3_256(SynNumericKeys::Root, &self.root, e)?;
        encode_count(SynNumericKeys::Count, self.count, e)?;
        encode_to(&self.to, e)?;
        if let Some(prefix) = &self.prefix {
            // Only encode the prefix if the doc count exceeds the threshold
            if self.count > MIN_DOC_COUNT_PREFIX_THRESHOLD {
                encode_prefix(prefix, e)?;
            }
        }
        encode_blake3_256(SynNumericKeys::PeerRoot, &self.peer_root, e)?;
        encode_count(SynNumericKeys::PeerCount, self.peer_count, e)?;
        Ok(())
    }
}

/// Helper function to encode BLAKE3-256 hash and its key.
fn encode_blake3_256<W: Write>(
    k: SynNumericKeys,
    h: &Blake3256,
    e: &mut Encoder<W>,
) -> Result<(), encode::Error<W::Error>> {
    e.encode(k)?.bytes(h)?.ok()
}

/// Helper function to encode count and its key.
fn encode_count<W: Write>(
    k: SynNumericKeys,
    count: u64,
    e: &mut Encoder<W>,
) -> Result<(), encode::Error<W::Error>> {
    e.encode(k)?.u64(count)?.ok()
}

/// Helper function to encode `to` (Ed25519) public key and its key.
fn encode_to<W: Write>(
    pubkey: &Ed25519Pubkey,
    e: &mut Encoder<W>,
) -> Result<(), encode::Error<W::Error>> {
    e.encode(SynNumericKeys::To)?.bytes(pubkey)?.ok()
}

/// Helper function to encode prefix array and its key.
fn encode_prefix<W: Write>(
    prefix: &PrefixArray,
    e: &mut Encoder<W>,
) -> Result<(), encode::Error<W::Error>> {
    let l: u64 = prefix
        .len()
        .try_into()
        .map_err(|e| encode::Error::message(format!("Failed to convert prefix length: {e}")))?;

    if is_valid_prefix_len(l) {
        e.encode(SynNumericKeys::Prefix)?.array(l)?;
        for p in prefix {
            e.bytes(p)?;
        }
        Ok(())
    } else {
        Err(encode::Error::message(
            "Invalid prefix length, need to be 2^N and N <= 14",
        ))
    }
}

impl<C> Decode<'_, C> for MsgSyn {
    fn decode(
        d: &mut Decoder<'_>,
        _: &mut C,
    ) -> Result<Self, decode::Error> {
        let map_len = d.map()?.ok_or_else(|| {
            decode::Error::message("Expected definite-sized map").at(d.position())
        })?;
        if map_len > Self::MAX_NUM_FIELDS {
            Err(decode::Error::message("Too many fields in a map").at(d.position()))?;
        }

        let root = decode_blake3_256(SynNumericKeys::Root, d)?;
        let count = decode_count(SynNumericKeys::Count, d)?;
        let to = decode_to(d)?;
        // Only try to decode prefix if count exceeds threshold
        let prefix = if count > MIN_DOC_COUNT_PREFIX_THRESHOLD {
            decode_prefix(d)?
        } else {
            None
        };
        let peer_root = decode_blake3_256(SynNumericKeys::PeerRoot, d)?;
        let peer_count = decode_count(SynNumericKeys::PeerCount, d)?;

        Ok(MsgSyn {
            root,
            count,
            to,
            prefix,
            peer_root,
            peer_count,
        })
    }
}

/// Helper function to decode BLAKE3-256 hash and its key.
fn decode_blake3_256(
    k: SynNumericKeys,
    d: &mut Decoder<'_>,
) -> Result<Blake3256, decode::Error> {
    if d.decode::<SynNumericKeys>().is_ok_and(|key| key == k) {
        d.bytes()?
            .try_into()
            .map_err(|err| decode::Error::custom(err).at(d.position()))
    } else {
        Err(decode::Error::message(format!("Expected key number {k}")).at(d.position()))
    }
}

/// Helper function to decode count and its key.
fn decode_count(
    k: SynNumericKeys,
    d: &mut Decoder<'_>,
) -> Result<u64, decode::Error> {
    if d.decode::<SynNumericKeys>().is_ok_and(|key| key == k) {
        d.u64()
    } else {
        Err(decode::Error::message(format!("Expected key number {k}")).at(d.position()))
    }
}

/// Helper function to decode `to` (Ed25519) public key and its key.
fn decode_to(d: &mut Decoder<'_>) -> Result<Ed25519Pubkey, decode::Error> {
    if d.decode::<SynNumericKeys>()
        .is_ok_and(|key| matches!(key, SynNumericKeys::To))
    {
        d.bytes()?
            .try_into()
            .map_err(|err| decode::Error::custom(err).at(d.position()))
    } else {
        Err(
            decode::Error::message(format!("Expected `to` key number {}", SynNumericKeys::To))
                .at(d.position()),
        )
    }
}

/// Helper function to decode prefix array and its key.
fn decode_prefix(d: &mut Decoder<'_>) -> Result<Option<PrefixArray>, decode::Error> {
    if d.decode::<SynNumericKeys>()
        .is_ok_and(|key| matches!(key, SynNumericKeys::Prefix))
    {
        let len = d.array()?.ok_or_else(|| {
            decode::Error::message("Expected definite-sized array").at(d.position())
        })?;

        if !is_valid_prefix_len(len) {
            Err(
                decode::Error::message("Invalid prefix length, need to be 2^N and N <= 14")
                    .at(d.position()),
            )?;
        }
        let mut arr = vec![];
        for _ in 0..len {
            let bytes = d.bytes()?;
            let hash: [u8; 32] = bytes.try_into().map_err(|_| {
                decode::Error::message("Invalid hash length, expected 32 bytes").at(d.position())
            })?;
            arr.push(hash);
        }
        return Ok(Some(arr));
    }
    Ok(None)
}

/// Check whether the prefix length is valid.
fn is_valid_prefix_len(n: u64) -> bool {
    (n != 0) && (n.is_power_of_two()) && n <= MAX_PREFIX_ARRAY_LENGTH
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode_decode_without_prefix() {
        let msg = MsgSyn {
            root: [0u8; 32],
            count: 10,
            to: [0u8; 32],
            prefix: None,
            peer_root: [0u8; 32],
            peer_count: 20,
        };

        let mut buf = minicbor::Encoder::new(vec![]);
        buf.encode(&msg).expect("Encoding should succeed");
        let encoded = buf.into_writer();
        let decoded = Syn::decode(&mut minicbor::Decoder::new(&encoded), &mut ())
            .expect("Decoding should succeed");

        assert_eq!(msg.root, decoded.root);
        assert_eq!(msg.count, decoded.count);
        assert_eq!(msg.to, decoded.to);
        assert_eq!(msg.prefix, decoded.prefix);
        assert_eq!(msg.peer_root, decoded.peer_root);
        assert_eq!(msg.peer_count, decoded.peer_count);
    }

    #[test]
    fn test_encode_decode_with_prefix_count_below_threshold() {
        let msg = MsgSyn {
            root: [0u8; 32],
            count: 64,
            to: [0u8; 32],
            prefix: Some(vec![[0u8; 32]; 4]),
            peer_root: [0u8; 32],
            peer_count: 20,
        };

        let mut buf = minicbor::Encoder::new(vec![]);
        buf.encode(&msg).expect("Encoding should succeed");
        let encoded = buf.into_writer();
        let decoded = Syn::decode(&mut minicbor::Decoder::new(&encoded), &mut ())
            .expect("Decoding should succeed");

        assert_eq!(msg.root, decoded.root);
        assert_eq!(msg.count, decoded.count);
        assert_eq!(msg.to, decoded.to);
        // Prefix shouldn't be encoded
        assert_eq!(None, decoded.prefix);
        assert_eq!(msg.peer_root, decoded.peer_root);
        assert_eq!(msg.peer_count, decoded.peer_count);
    }

    #[test]
    fn test_encode_decode_with_prefix_count_above_threshold() {
        let msg = MsgSyn {
            root: [0u8; 32],
            count: 80,
            to: [0u8; 32],
            prefix: Some(vec![[0u8; 32]; 4]),
            peer_root: [0u8; 32],
            peer_count: 20,
        };

        let mut buf = minicbor::Encoder::new(vec![]);
        buf.encode(&msg).expect("Encoding should succeed");
        let encoded = buf.into_writer();
        let decoded = Syn::decode(&mut minicbor::Decoder::new(&encoded), &mut ())
            .expect("Decoding should succeed");

        assert_eq!(msg.root, decoded.root);
        assert_eq!(msg.count, decoded.count);
        assert_eq!(msg.to, decoded.to);
        // Prefix shouldn't be encoded
        assert_eq!(msg.prefix, decoded.prefix);
        assert_eq!(msg.peer_root, decoded.peer_root);
        assert_eq!(msg.peer_count, decoded.peer_count);
    }

    #[test]
    fn test_encode_with_invalid_prefix_len() {
        let msg = MsgSyn {
            root: [0u8; 32],
            count: 80,
            to: [0u8; 32],
            prefix: Some(vec![[0u8; 32]; 3]),
            peer_root: [0u8; 32],
            peer_count: 20,
        };

        let mut buf = minicbor::Encoder::new(vec![]);
        assert!(
            buf.encode(&msg)
                .unwrap_err()
                .to_string()
                .contains("Invalid prefix length, need to be 2^N and N <= 14")
        );
    }

    #[test]
    fn test_encode_with_invalid_prefix_len_above_limit() {
        let msg = MsgSyn {
            root: [0u8; 32],
            count: 80,
            to: [0u8; 32],
            prefix: Some(vec![
                [0u8; 32];
                usize::try_from(MAX_PREFIX_ARRAY_LENGTH + 1).unwrap()
            ]),
            peer_root: [0u8; 32],
            peer_count: 20,
        };

        let mut buf = minicbor::Encoder::new(vec![]);
        assert!(
            buf.encode(&msg)
                .unwrap_err()
                .to_string()
                .contains("Invalid prefix length, need to be 2^N and N <= 14")
        );
    }
}
