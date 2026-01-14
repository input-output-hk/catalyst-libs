//! CID v1 (Content Identifier version 1) implementation for Catalyst Signed Documents.
//!
//! This module provides functionality to generate IPFS-compatible CID v1 identifiers
//! for CBOR-encoded Catalyst Signed Documents.
//!
//! ## CID v1 Structure
//!
//! The binary format follows the IPFS specification:
//! ```text
//! <cidv1> = <version> || <multicodec> || <multihash>
//! ```
//!
//! Where:
//! - `version`: varint(1) - CID version 1
//! - `multicodec`: varint(0x51) - CBOR codec
//! - `multihash`: varint(0x12) || varint(32) || digest\[32\] - SHA2-256 multihash
//!
//! ## Constraints
//!
//! - **Hash function**: Only SHA2-256 is supported (32-byte digest)
//! - **Codec**: Fixed to CBOR (0x51)
//! - **Output size**: 36 bytes in binary format

use std::{fmt, ops::Deref, str::FromStr};

use minicbor::{Decoder, Encoder, data::Tag, decode::Error as DecodeError};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// CBOR multicodec identifier.
///
/// See: <https://github.com/multiformats/multicodec/blob/master/table.csv>
const CBOR_CODEC: u64 = 0x51;

/// SHA2-256 multihash code.
const SHA2_256_CODE: u64 = 0x12;

/// CBOR tag for IPLD CID (Content Identifier).
///
/// See: <https://github.com/ipld/cid-cbor/>
const CID_CBOR_TAG: u64 = 42;

/// Errors that can occur during CID v1 operations.
#[derive(Debug, Clone, Error)]
pub enum CidError {
    /// Invalid CID bytes format.
    #[error("Invalid CID bytes: {0}")]
    InvalidCidBytes(String),

    /// Invalid CID string format.
    #[error("Invalid CID string: {0}")]
    InvalidCidString(String),

    /// Multihash creation failed.
    #[error("Multihash error: {0}")]
    MultihashError(String),

    /// Encoding or decoding error.
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// A new type wrapper around `cid::Cid` for type-safe CID v1 handling.
///
/// This type provides conversion methods and trait implementations for working with
/// CID v1 identifiers, especially in the context of CBOR-encoded Catalyst Signed
/// Documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cid(cid::Cid);

impl Deref for Cid {
    type Target = cid::Cid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<cid::Cid> for Cid {
    fn from(cid: cid::Cid) -> Self {
        Cid(cid)
    }
}

impl From<Cid> for cid::Cid {
    fn from(cid: Cid) -> Self {
        cid.0
    }
}

impl From<Cid> for Vec<u8> {
    fn from(cid: Cid) -> Self {
        cid.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for Cid {
    type Error = CidError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        cid::Cid::try_from(bytes)
            .map(Cid)
            .map_err(|e| CidError::InvalidCidBytes(e.to_string()))
    }
}

impl FromStr for Cid {
    type Err = CidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        cid::Cid::try_from(s)
            .map(Cid)
            .map_err(|e| CidError::InvalidCidString(e.to_string()))
    }
}

impl fmt::Display for Cid {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Cid {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Cid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl minicbor::Encode<()> for Cid {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // Encode as tag(42) containing the CID bytes
        e.tag(Tag::new(CID_CBOR_TAG))?;
        e.bytes(&self.0.to_bytes())?;
        Ok(())
    }
}

impl<'de> minicbor::Decode<'de, ()> for Cid {
    fn decode(
        d: &mut Decoder<'de>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let tag = d
            .tag()
            .map_err(|e| DecodeError::message(e.to_string()))?
            .as_u64();
        if tag != CID_CBOR_TAG {
            return Err(DecodeError::message(format!(
                "Expected IPLD CID tag ({CID_CBOR_TAG}), got {tag}",
            )));
        }
        let bytes = d.bytes().map_err(|e| DecodeError::message(e.to_string()))?;
        cid::Cid::try_from(bytes)
            .map(Cid)
            .map_err(|e| DecodeError::message(e.to_string()))
    }
}

/// Generates a CID v1 for the given CBOR bytes.
///
/// # Arguments
///
/// * `cbor_bytes` - The CBOR-encoded data (typically from a `CatalystSignedDocument`)
///
/// # Returns
///
/// A `Cid` object representing the CID v1
///
/// # Errors
///
/// Returns a `CidError` if:
/// - SHA2-256 multihash construction fails
/// - The digest size is invalid
pub(crate) fn to_cid_v1(cbor_bytes: &[u8]) -> Result<Cid, CidError> {
    // Compute SHA2-256 hash
    let mut hasher = Sha256::new();
    hasher.update(cbor_bytes);
    let hash_digest = hasher.finalize();

    // Create multihash from digest using the wrap() API
    // The generic parameter <64> is the max digest size we support
    let multihash = multihash::Multihash::<64>::wrap(SHA2_256_CODE, &hash_digest)
        .map_err(|e| CidError::MultihashError(e.to_string()))?;

    // Create CID v1 with CBOR codec
    let cid = cid::Cid::new_v1(CBOR_CODEC, multihash);

    Ok(Cid(cid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, CatalystSignedDocument, ContentType};

    /// SHA2-256 digest size in bytes.
    const SHA2_256_SIZE: usize = 32;

    fn create_test_document() -> CatalystSignedDocument {
        Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": "0197f398-9f43-7c23-a576-f765131b81f2",
                "ver": "0197f398-9f43-7c23-a576-f765131b81f2",
                "type": "ab7c2428-c353-4331-856e-385b2eb20546",
                "content-type": ContentType::Json,
            }))
            .expect("Should create metadata")
            .with_json_content(&serde_json::json!({
                "test": "content"
            }))
            .expect("Should set content")
            .build()
            .expect("Should build document")
    }

    #[test]
    fn test_cid_wrapping() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");

        // Test that we can access inner cid::Cid through Deref
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), CBOR_CODEC);
    }

    #[test]
    fn test_cid_from_bytes() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");
        let cid_bytes = Vec::<u8>::from(cid);

        // Convert back from bytes
        let cid_from_bytes =
            Cid::try_from(cid_bytes.as_slice()).expect("Should parse CID from bytes");

        assert_eq!(cid, cid_from_bytes);
    }

    #[test]
    fn test_cid_from_str() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");
        let cid_string = cid.to_string();

        // Parse the string back to a CID
        let cid_from_str = Cid::from_str(&cid_string).expect("CID string should be parsable");

        assert_eq!(cid, cid_from_str);
    }

    #[test]
    fn test_cid_display() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");
        let cid_string = cid.to_string();

        // Should be multibase encoded (starts with 'b' for base32)
        assert!(
            cid_string.starts_with('b'),
            "CID v1 base32 string should start with 'b'"
        );
    }

    #[test]
    fn test_cid_from_inner_cid() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");
        let inner_cid: cid::Cid = cid.into();

        // Should preserve the inner CID
        assert_eq!(inner_cid.version(), cid::Version::V1);
    }

    #[test]
    fn test_inner_cid_from_cid() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID generation should succeed");
        let recovered: cid::Cid = cid.into();

        assert_eq!(recovered, *cid);
    }

    #[test]
    fn test_cid_generation_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let result = to_cid_v1(&cbor_bytes);
        assert!(result.is_ok(), "CID generation should succeed");

        let cid = result.expect("CID should be valid");
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), CBOR_CODEC);
    }

    #[test]
    fn test_binary_format_size_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID should be valid");
        let cid_bytes = Vec::<u8>::from(cid);

        assert_eq!(
            cid_bytes.len(),
            36,
            "CID v1 binary format should be exactly 36 bytes"
        );
    }

    #[test]
    fn test_determinism_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid1 = to_cid_v1(&cbor_bytes).expect("First CID should be valid");
        let cid2 = to_cid_v1(&cbor_bytes).expect("Second CID should be valid");

        assert_eq!(cid1, cid2, "Same document should produce identical CIDs");
    }

    #[test]
    fn test_string_format_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID should be valid");
        let cid_string = cid.to_string();

        assert!(
            cid_string.starts_with('b'),
            "CID v1 base32 string should start with 'b'"
        );
    }

    #[test]
    fn test_multihash_properties_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID should be valid");
        let multihash = cid.hash();

        assert_eq!(
            multihash.code(),
            SHA2_256_CODE,
            "Multihash code should be SHA2-256 (0x12)"
        );
        assert_eq!(
            multihash.size() as usize,
            SHA2_256_SIZE,
            "Multihash digest size should be 32 bytes"
        );
    }

    #[test]
    fn test_different_documents_different_cids() {
        let doc1 = create_test_document();
        let doc2 = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": "0197f398-9f43-7c23-a576-f765131b81f3",
                "ver": "0197f398-9f43-7c23-a576-f765131b81f3",
                "type": "ab7c2428-c353-4331-856e-385b2eb20546",
                "content-type": ContentType::Json,
            }))
            .expect("Should create metadata")
            .with_json_content(&serde_json::json!({
                "different": "content"
            }))
            .expect("Should set content")
            .build()
            .expect("Should build document");

        let cid1 =
            to_cid_v1(&doc1.to_bytes().expect("Should serialize")).expect("CID 1 should be valid");
        let cid2 =
            to_cid_v1(&doc2.to_bytes().expect("Should serialize")).expect("CID 2 should be valid");

        assert_ne!(
            cid1, cid2,
            "Different documents should produce different CIDs"
        );
    }

    #[test]
    fn test_cid_string_properties() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid = to_cid_v1(&cbor_bytes).expect("CID should be valid");
        let cid_string = cid.to_string();

        // Base32 strings start with 'b'
        assert!(
            cid_string.starts_with('b'),
            "CID v1 base32 string should start with 'b'"
        );

        // Base32 encoding uses lowercase letters and digits 2-7
        assert!(
            cid_string
                .chars()
                .skip(1)
                .all(|c| c.is_ascii_lowercase() || ('2'..='7').contains(&c)),
            "CID v1 base32 string should only contain lowercase letters and digits 2-7"
        );

        // Should be non-empty and reasonably sized
        assert!(
            cid_string.len() > 10,
            "CID string should have reasonable length"
        );
    }

    #[test]
    fn test_full_cid_round_trip() {
        let doc = create_test_document();
        let original_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        // Generate CID
        let cid = to_cid_v1(&original_bytes).expect("CID should be valid");
        let cid_string = cid.to_string();

        // Convert CID to bytes
        let cid_bytes = Vec::<u8>::from(cid);

        // Parse bytes back to CID
        let cid_from_bytes =
            Cid::try_from(cid_bytes.as_slice()).expect("Should parse CID from bytes");

        // Convert back to string
        let final_string = cid_from_bytes.to_string();

        assert_eq!(
            cid_string, final_string,
            "Full round-trip should preserve CID string"
        );
    }
}
