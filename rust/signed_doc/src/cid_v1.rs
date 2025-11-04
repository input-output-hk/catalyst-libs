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
//! - `multihash`: varint(0x12) || varint(32) || digest[32] - SHA2-256 multihash
//!
//! ## Constraints
//!
//! - **Hash function**: Only SHA2-256 is supported (32-byte digest)
//! - **Codec**: Fixed to CBOR (0x51)
//! - **Output size**: 36 bytes in binary format
//!
//! ## Example
//!
//! ```no_run
//! # use catalyst_signed_doc::CatalystSignedDocument;
//! # let doc: CatalystSignedDocument = todo!();
//! let cid_string = doc.to_cid_v1_string()?;
//! // Result: "bafyrei..." (base32-encoded CID v1)
//! # Ok::<(), anyhow::Error>(())
//! ```

use sha2::{Digest, Sha256};

/// CBOR multicodec identifier.
///
/// See: <https://github.com/multiformats/multicodec/blob/master/table.csv>
const CBOR_CODEC: u64 = 0x51;

/// SHA2-256 multihash code.
const SHA2_256_CODE: u64 = 0x12;

/// Generates a CID v1 for the given CBOR bytes.
///
/// # Arguments
///
/// * `cbor_bytes` - The CBOR-encoded data to generate a CID for
///
/// # Returns
///
/// A `cid::Cid` object representing the CID v1
///
/// # Errors
///
/// Returns an error if multihash construction fails
pub fn to_cid_v1(cbor_bytes: &[u8]) -> anyhow::Result<cid::Cid> {
    // Compute SHA2-256 hash
    let mut hasher = Sha256::new();
    hasher.update(cbor_bytes);
    let hash_digest = hasher.finalize();

    // Create multihash from digest using the wrap() API
    // The generic parameter <64> is the max digest size we support
    let multihash = multihash::Multihash::<64>::wrap(SHA2_256_CODE, &hash_digest)?;

    // Create CID v1 with CBOR codec
    let cid = cid::Cid::new_v1(CBOR_CODEC, multihash);

    Ok(cid)
}

/// Generates a CID v1 and returns it as a multibase-encoded string.
///
/// Uses base32 encoding (CID v1 default).
///
/// # Arguments
///
/// * `cbor_bytes` - The CBOR-encoded data to generate a CID for
///
/// # Returns
///
/// A base32-encoded CID v1 string (starting with 'b')
///
/// # Errors
///
/// Returns an error if CID generation fails
pub fn to_cid_v1_string(cbor_bytes: &[u8]) -> anyhow::Result<String> {
    let cid = to_cid_v1(cbor_bytes)?;
    Ok(cid.to_string())
}

/// Generates a CID v1 and returns it as raw bytes.
///
/// Binary format: `<version><multicodec><multihash>`
///
/// # Arguments
///
/// * `cbor_bytes` - The CBOR-encoded data to generate a CID for
///
/// # Returns
///
/// A 36-byte vector containing the binary CID v1
///
/// # Errors
///
/// Returns an error if CID generation fails
pub fn to_cid_v1_bytes(cbor_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cid = to_cid_v1(cbor_bytes)?;
    Ok(cid.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, CatalystSignedDocument, ContentType};

    /// SHA2-256 digest size in bytes.
    const SHA2_256_SIZE: usize = 32;

    /// Helper function to create a test CatalystSignedDocument
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

    /// Test CID v1 generation from a CatalystSignedDocument
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

    /// Test that binary format is exactly 36 bytes for a CatalystSignedDocument
    #[test]
    fn test_binary_format_size_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid_bytes = to_cid_v1_bytes(&cbor_bytes).expect("CID bytes should be valid");

        assert_eq!(
            cid_bytes.len(),
            36,
            "CID v1 binary format should be exactly 36 bytes"
        );
    }

    /// Test determinism: same document produces same CID
    #[test]
    fn test_determinism_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid1 = to_cid_v1(&cbor_bytes).expect("First CID should be valid");
        let cid2 = to_cid_v1(&cbor_bytes).expect("Second CID should be valid");

        assert_eq!(cid1, cid2, "Same document should produce identical CIDs");
    }

    /// Test string format starts with 'b' (base32) for a CatalystSignedDocument
    #[test]
    fn test_string_format_from_document() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid_string = to_cid_v1_string(&cbor_bytes).expect("CID string should be valid");

        assert!(
            cid_string.starts_with('b'),
            "CID v1 base32 string should start with 'b'"
        );
    }

    /// Test that multihash has correct SHA2-256 properties
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

    /// Test that CID string can be parsed back to a CID object
    #[test]
    fn test_cid_string_round_trip() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid_string = to_cid_v1_string(&cbor_bytes).expect("CID string should be valid");

        // Parse the string back to a CID
        let parsed_cid = cid::Cid::try_from(cid_string.as_str())
            .expect("CID string should be parseable");

        // Generate CID directly for comparison
        let original_cid = to_cid_v1(&cbor_bytes).expect("CID should be valid");

        assert_eq!(
            parsed_cid, original_cid,
            "Parsed CID should match original CID"
        );
    }

    /// Test that different documents produce different CID strings
    #[test]
    fn test_different_documents_different_cid_strings() {
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

        let cid_string1 = to_cid_v1_string(&doc1.to_bytes().expect("Should serialize"))
            .expect("CID string 1 should be valid");
        let cid_string2 = to_cid_v1_string(&doc2.to_bytes().expect("Should serialize"))
            .expect("CID string 2 should be valid");

        assert_ne!(
            cid_string1, cid_string2,
            "Different documents should produce different CID strings"
        );
    }

    /// Test CID string properties (base32 encoding characteristics)
    #[test]
    fn test_cid_string_properties() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid_string = to_cid_v1_string(&cbor_bytes).expect("CID string should be valid");

        // Base32 strings start with 'b'
        assert!(
            cid_string.starts_with('b'),
            "CID v1 base32 string should start with 'b'"
        );

        // Base32 encoding uses lowercase letters and digits 2-7
        assert!(
            cid_string.chars().skip(1).all(|c| c.is_ascii_lowercase() || ('2'..='7').contains(&c)),
            "CID v1 base32 string should only contain lowercase letters and digits 2-7"
        );

        // Should be non-empty and reasonably sized
        assert!(
            cid_string.len() > 10,
            "CID string should have reasonable length"
        );
    }

    /// Test CID string determinism (same document always produces same string)
    #[test]
    fn test_cid_string_determinism() {
        let doc = create_test_document();
        let cbor_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        let cid_string1 = to_cid_v1_string(&cbor_bytes).expect("First CID string should be valid");
        let cid_string2 = to_cid_v1_string(&cbor_bytes).expect("Second CID string should be valid");

        assert_eq!(
            cid_string1, cid_string2,
            "Same document should always produce the same CID string"
        );
    }

    /// Test full round-trip: document -> CID string -> CID -> bytes -> CID
    #[test]
    fn test_full_cid_round_trip() {
        let doc = create_test_document();
        let original_bytes = doc.to_bytes().expect("Should serialize to CBOR");

        // Convert to CID string
        let cid_string = to_cid_v1_string(&original_bytes).expect("CID string should be valid");

        // Parse string to CID
        let cid_from_string = cid::Cid::try_from(cid_string.as_str())
            .expect("Should parse CID from string");

        // Convert CID to bytes
        let cid_bytes = cid_from_string.to_bytes();

        // Parse bytes back to CID
        let cid_from_bytes = cid::Cid::try_from(cid_bytes.as_slice())
            .expect("Should parse CID from bytes");

        // Convert back to string
        let final_string = cid_from_bytes.to_string();

        assert_eq!(
            cid_string, final_string,
            "Full round-trip should preserve CID string"
        );
    }
}
