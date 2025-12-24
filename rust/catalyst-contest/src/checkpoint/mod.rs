//! Catalyst Ballot Checkpoint
//!
//! This serves as a checkpoint that collects new `contest-ballot-payload` documents that
//! have been observed by a bulletin board.
//!
//! It will be created periodically during the voting period to allow proofs of inclusion
//! to be firmly anchored and repeatably verifiable, and to allow voters or auditors to
//! confirm  a bulletin board acted honestly and included all valid ballots it detected.
//!
//! At another interval (which may be the same or different), a roll-up of the latest
//! checkpoint is submitted to a blockchain to provide an immutable anchor of the ballots
//! collected by a bulletin board up to that point in time.

mod drep_encryption_key;
mod rejection_reason;
mod rejections;
mod smt;
mod stage;
mod tally;

use cbork_utils::{decode_context::DecodeCtx, map::Map};
pub use drep_encryption_key::DrepEncryptionKey;
use minicbor::{Decode, Decoder, Encode, decode::Error as DecodeError};
pub use rejection_reason::RejectionReason;
pub use rejections::Rejections;
pub use smt::{entries::SmtEntries, root::SmtRoot};
pub use stage::BallotProcessingStage;
pub use tally::encrypted::EncryptedTally;

use crate::checkpoint::tally::Tally;
//use cbork_utils::decode_helper::decode_array_len;

/// Number of required fields in `CatalystBallotCheckpointPayload`.
const REQUIRED_FIELD_COUNT: u64 = 3;
/// Number of optional fields in `CatalystBallotCheckpointPayload`.
const OPTIONAL_FIELD_COUNT: u64 = 4;

/// Catalyst Ballot Checkpoint Payload
///
/// TH CDDL Schema:
/// ```
/// ; Catalyst Ballot Checkpoint Payload data object.
/// contest-ballot-checkpoint = {
///     "stage" : stage
///     "smt-root" : smt-root
///     "smt-entries" : smt-entries
///     ? "rejections" : rejections
///     ? "encrypted-tally" : encrypted-tally
///     ? "tally" : tally
///     ? "drep-encryption-key" : drep-encryption-key
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalystBallotCheckpointPayload {
    /// What stage in the ballot processing is represented by this checkpoint.
    stage: BallotProcessingStage,
    /// Blake3 256‑bit digest of the root of the Sparse Merkle Tree (SMT) containing all
    /// accepted ballot `document_ref`s up to and including this checkpoint.
    smt_root: SmtRoot,
    /// The total number of documents (leaves) in the SMT at this checkpoint.
    smt_entries: SmtEntries,
    /// Optional map of rejected contest ballots by reason.
    rejections: Option<Rejections>,
    /// Placeholder map of `document_ref => encrypted-tally-proposal-result`.
    encrypted_tally: Option<EncryptedTally>,
    /// Placeholder map of `document_ref => tally-proposal-result` for clear tally
    /// snapshots.
    tally: Option<Tally>,
    /// Placeholder for D-Rep Encryption Key to allow decryption where required for audit
    /// or published results.
    drep_encryption_key: Option<DrepEncryptionKey>,
}

impl Encode<()> for CatalystBallotCheckpointPayload {
    #[allow(clippy::arithmetic_side_effects)]
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let mut field_count = REQUIRED_FIELD_COUNT; // stage, smt-root, smt-entries are required
        if self.rejections.is_some() {
            field_count += 1;
        }
        if self.encrypted_tally.is_some() {
            field_count += 1;
        }
        if self.tally.is_some() {
            field_count += 1;
        }
        if self.drep_encryption_key.is_some() {
            field_count += 1;
        }

        e.map(field_count)?;

        // Encode in RFC 8949 canonical order (length-first, then lexicographic):
        // 1. "stage" (6 bytes: 0x65 + 5 chars)
        e.str("stage")?;
        self.stage.encode(e, ctx)?;

        // 2. "tally" (6 bytes: 0x65 + 5 chars) - optional
        if let Some(ref tally) = self.tally {
            e.str("tally")?;
            tally.encode(e, ctx)?;
        }

        // 3. "smt-root" (9 bytes: 0x68 + 8 chars)
        e.str("smt-root")?;
        self.smt_root.encode(e, ctx)?;

        // 4. "rejections" (11 bytes: 0x6A + 10 chars) - optional
        if let Some(ref rejections) = self.rejections {
            e.str("rejections")?;
            rejections.encode(e, ctx)?;
        }

        // 5. "smt-entries" (12 bytes: 0x6B + 11 chars)
        e.str("smt-entries")?;
        self.smt_entries.encode(e, ctx)?;

        // 6. "encrypted-tally" (16 bytes: 0x6F + 15 chars) - optional
        if let Some(ref encrypted_tally) = self.encrypted_tally {
            e.str("encrypted-tally")?;
            encrypted_tally.encode(e, ctx)?;
        }

        // 7. "drep-encryption-key" (20 bytes: 0x73 + 19 chars) - optional
        if let Some(ref drep_encryption_key) = self.drep_encryption_key {
            e.str("drep-encryption-key")?;
            drep_encryption_key.encode(e, ctx)?;
        }

        Ok(())
    }
}

impl Decode<'_, ()> for CatalystBallotCheckpointPayload {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        const MAX_FIELDS: u64 = REQUIRED_FIELD_COUNT + OPTIONAL_FIELD_COUNT;

        let entries = Map::decode(d, &mut DecodeCtx::Deterministic)?;
        let map_len = entries.len() as u64;

        if map_len < REQUIRED_FIELD_COUNT {
            return Err(DecodeError::message(format!(
                "CatalystBallotCheckpointPayload must have {REQUIRED_FIELD_COUNT} required fields, got {map_len}.",
            )));
        }

        if map_len > MAX_FIELDS {
            return Err(DecodeError::message(format!(
                "CatalystBallotCheckpointPayload must have at most {MAX_FIELDS} fields, got {map_len}.",
            )));
        }

        // Initialize all fields as None/missing
        let mut stage: Option<BallotProcessingStage> = None;
        let mut smt_root: Option<SmtRoot> = None;
        let mut smt_entries: Option<SmtEntries> = None;
        let mut rejections: Option<Rejections> = None;
        let mut encrypted_tally: Option<EncryptedTally> = None;
        let mut tally: Option<Tally> = None;
        let mut drep_encryption_key: Option<DrepEncryptionKey> = None;

        // Iterate through all entries and decode based on key name
        for entry in entries.as_slice() {
            let key = Decoder::new(&entry.key_bytes).str()?;
            let mut value_decoder = Decoder::new(&entry.value);

            match key {
                "stage" => {
                    if stage.is_some() {
                        return Err(DecodeError::message("Duplicate 'stage' field"));
                    }
                    stage = Some(BallotProcessingStage::decode(&mut value_decoder, ctx)?);
                },
                "smt-root" => {
                    if smt_root.is_some() {
                        return Err(DecodeError::message("Duplicate 'smt-root' field"));
                    }
                    smt_root = Some(SmtRoot::decode(&mut value_decoder, ctx)?);
                },
                "smt-entries" => {
                    if smt_entries.is_some() {
                        return Err(DecodeError::message("Duplicate 'smt-entries' field"));
                    }
                    smt_entries = Some(SmtEntries::decode(&mut value_decoder, ctx)?);
                },
                "rejections" => {
                    if rejections.is_some() {
                        return Err(DecodeError::message("Duplicate 'rejections' field"));
                    }
                    rejections = Some(Rejections::decode(&mut value_decoder, ctx)?);
                },
                "encrypted-tally" => {
                    if encrypted_tally.is_some() {
                        return Err(DecodeError::message("Duplicate 'encrypted-tally' field"));
                    }
                    encrypted_tally = Some(EncryptedTally::decode(&mut value_decoder, ctx)?);
                },
                "tally" => {
                    if tally.is_some() {
                        return Err(DecodeError::message("Duplicate 'tally' field"));
                    }
                    tally = Some(Tally::decode(&mut value_decoder, ctx)?);
                },
                "drep-encryption-key" => {
                    if drep_encryption_key.is_some() {
                        return Err(DecodeError::message(
                            "Duplicate 'drep-encryption-key' field",
                        ));
                    }
                    drep_encryption_key = Some(DrepEncryptionKey::decode(&mut value_decoder, ctx)?);
                },
                _ => {
                    return Err(DecodeError::message(format!(
                        "Unexpected field in CatalystBallotCheckpointPayload: {key}",
                    )));
                },
            }
        }

        // Verify required fields are present
        let stage = stage.ok_or_else(|| DecodeError::message("Missing required field 'stage'"))?;
        let smt_root =
            smt_root.ok_or_else(|| DecodeError::message("Missing required field 'smt-root'"))?;
        let smt_entries = smt_entries
            .ok_or_else(|| DecodeError::message("Missing required field 'smt-entries'"))?;

        Ok(CatalystBallotCheckpointPayload {
            stage,
            smt_root,
            smt_entries,
            rejections,
            encrypted_tally,
            tally,
            drep_encryption_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use catalyst_signed_doc::tests_utils::create_dummy_doc_ref;

    use super::*;

    fn create_test_smt_root() -> SmtRoot {
        SmtRoot(vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ])
    }

    fn create_test_rejections() -> Rejections {
        let doc_ref1 = create_dummy_doc_ref();
        let doc_ref2 = create_dummy_doc_ref();

        let mut rejections_map = std::collections::HashMap::new();
        rejections_map.insert(RejectionReason::AlreadyVoted, vec![doc_ref1.clone()].into());
        rejections_map.insert(
            RejectionReason::ObsoleteVote,
            vec![doc_ref2.clone(), doc_ref1.clone()].into(),
        );

        Rejections(rejections_map)
    }

    fn create_test_encrypted_tally() -> EncryptedTally {
        // Use Default trait to create an empty encrypted tally
        EncryptedTally::default()
    }

    fn create_test_tally() -> Tally {
        // Use Default trait to create an empty tally
        Tally::default()
    }

    #[test]
    fn roundtrip_required_fields_only() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::BulletinBoard,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(42u64),
            rejections: None,
            encrypted_tally: None,
            tally: None,
            drep_encryption_key: None,
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_with_rejections_and_drep_key() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::Tally,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(100u64),
            rejections: Some(create_test_rejections()),
            encrypted_tally: None,
            tally: None,
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_with_tally_and_drep_key() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::Audit,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(500u64),
            rejections: None,
            encrypted_tally: None,
            tally: Some(create_test_tally()),
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_with_rejections_and_encrypted_tally() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::BulletinBoard,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(1000u64),
            rejections: Some(create_test_rejections()),
            encrypted_tally: Some(create_test_encrypted_tally()),
            tally: None,
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_with_drep_key() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::Tally,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(250u64),
            rejections: None,
            encrypted_tally: None,
            tally: None,
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_all_fields() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::Audit,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(9999u64),
            rejections: Some(create_test_rejections()),
            encrypted_tally: Some(create_test_encrypted_tally()),
            tally: Some(create_test_tally()),
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_with_multiple_optional_fields() {
        let original = CatalystBallotCheckpointPayload {
            stage: BallotProcessingStage::BulletinBoard,
            smt_root: create_test_smt_root(),
            smt_entries: SmtEntries::from(777u64),
            rejections: Some(create_test_rejections()),
            encrypted_tally: Some(create_test_encrypted_tally()),
            tally: None,
            drep_encryption_key: Some(DrepEncryptionKey),
        };

        let mut buffer = Vec::new();
        original
            .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded =
            CatalystBallotCheckpointPayload::decode(&mut minicbor::Decoder::new(&buffer), &mut ())
                .unwrap();
        assert_eq!(original, decoded);
    }
}
