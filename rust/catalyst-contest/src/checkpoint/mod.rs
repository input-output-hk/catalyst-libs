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

pub use drep_encryption_key::DrepEncryptionKey;
use minicbor::{Decode, Encode, decode::Error as DecodeError};
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

/// Field name.
const REJECTIONS_NAME: &str = "rejections";
/// Field name.
const ENCRYPTED_TALLY_NAME: &str = "encrypted-tally";
/// Field name.
const TALLY_NAME: &str = "tally";
/// Field name.
const DREP_KEY_NAME: &str = "drep-encryption-key";

/// Error for unknown payload field names.
#[derive(Debug)]
struct UnexpectedPayloadField(String);

impl From<UnexpectedPayloadField> for DecodeError {
    fn from(value: UnexpectedPayloadField) -> Self {
        DecodeError::message(format!(
            "Unexpected field in CatalystBallotCheckpointPayload: {}",
            value.0,
        ))
    }
}

/// Catalyst Ballot Checkpoint Payload
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

        e.str("stage")?;
        self.stage.encode(e, ctx)?;

        e.str("smt-root")?;
        self.smt_root.encode(e, ctx)?;

        e.str("smt-entries")?;
        self.smt_entries.encode(e, ctx)?;

        if let Some(ref rejections) = self.rejections {
            e.str("rejections")?;
            rejections.encode(e, ctx)?;
        }

        if let Some(ref encrypted_tally) = self.encrypted_tally {
            e.str("encrypted-tally")?;
            encrypted_tally.encode(e, ctx)?;
        }

        if let Some(ref tally) = self.tally {
            e.str("tally")?;
            tally.encode(e, ctx)?;
        }

        if let Some(ref drep_encryption_key) = self.drep_encryption_key {
            e.str("drep-encryption-key")?;
            drep_encryption_key.encode(e, ctx)?;
        }

        Ok(())
    }
}

impl Decode<'_, ()> for CatalystBallotCheckpointPayload {
    #[allow(clippy::arithmetic_side_effects)]
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        const MAX_FIELDS: u64 = REQUIRED_FIELD_COUNT + OPTIONAL_FIELD_COUNT;
        let Some(map_len) = d.map()? else {
            return Err(DecodeError::message(
                "CatalystBallotCheckpointPayload must be a defined-size map",
            ));
        };

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
        // Required fields
        let key = d.str()?;
        if key != "stage" {
            return Err(DecodeError::message("Expected 'stage', got {key}"));
        }
        let stage = BallotProcessingStage::decode(d, ctx)?;

        let key = d.str()?;
        if key != "smt-root" {
            return Err(DecodeError::message("Expected 'smt-root', got {key}"));
        }
        let smt_root = SmtRoot::decode(d, ctx)?;

        let key = d.str()?;
        if key != "smt-entries" {
            return Err(DecodeError::message("Expected 'smt-entries', got {key}"));
        }
        let smt_entries = SmtEntries::decode(d, ctx)?;

        // Optional fields
        let mut rejections: Option<Rejections> = None;
        let mut encrypted_tally: Option<EncryptedTally> = None;
        let mut tally: Option<Tally> = None;
        let mut drep_encryption_key: Option<DrepEncryptionKey> = None;

        let mut optional_fields = vec![
            REJECTIONS_NAME,
            ENCRYPTED_TALLY_NAME,
            TALLY_NAME,
            DREP_KEY_NAME,
        ];

        let mut remaining_opt_items = map_len - REQUIRED_FIELD_COUNT;

        if remaining_opt_items > 0 {
            let mut key = d.str()?;
            // Return error if field name is not expected
            if !&optional_fields.contains(&key) {
                return Err(UnexpectedPayloadField(key.to_string()).into());
            }

            // Rejections field
            let field_name = optional_fields.remove(0);
            if key == field_name {
                rejections = Some(Rejections::decode(d, ctx)?);
                remaining_opt_items -= 1;
                if remaining_opt_items > 0 {
                    key = d.str()?;
                }
            } else if !&optional_fields.contains(&key) {
                return Err(UnexpectedPayloadField(key.to_string()).into());
            }

            // Encrypted Tally field
            if remaining_opt_items > 0 {
                let field_name = optional_fields.remove(0);
                if key == field_name {
                    encrypted_tally = Some(EncryptedTally::decode(d, ctx)?);
                    remaining_opt_items -= 1;
                    if remaining_opt_items > 0 {
                        key = d.str()?;
                    }
                } else if !&optional_fields.contains(&key) {
                    return Err(UnexpectedPayloadField(key.to_string()).into());
                }
            }

            // Tally field
            if remaining_opt_items > 0 {
                let field_name = optional_fields.remove(0);
                if key == field_name {
                    tally = Some(Tally::decode(d, ctx)?);
                    remaining_opt_items -= 1;
                    if remaining_opt_items > 0 {
                        key = d.str()?;
                    }
                } else if !&optional_fields.contains(&key) {
                    return Err(UnexpectedPayloadField(key.to_string()).into());
                }
            }

            // D-Rep Encryption Key field
            if remaining_opt_items > 0 {
                let field_name = optional_fields.remove(0);
                if key != field_name {
                    return Err(UnexpectedPayloadField(key.to_string()).into());
                }
                drep_encryption_key = Some(DrepEncryptionKey::decode(d, ctx)?);
            }
        }

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
