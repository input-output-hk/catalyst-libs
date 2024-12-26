//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

use std::sync::Arc;

use cardano_blockchain_types::{MetadatumLabel, TransactionAuxData};
use minicbor::{Decode, Decoder};
use pallas::ledger::traverse::MultiEraTx;
use rbac_registration::cardano::cip509::{Cip509 as RbacRegCip509, Cip509Validation};

use super::{DecodedMetadata, DecodedMetadataItem, DecodedMetadataValues, ValidationReport};

/// CIP509 metadatum.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509 {
    /// CIP509 data.
    pub cip509: RbacRegCip509,
    /// Validation value, not a part of CIP509, justs storing validity of the data.
    pub validation: Cip509Validation,
}

impl Cip509 {
    /// Decode and validate CIP509 Metadata
    ///
    /// # Returns
    ///
    /// Nothing.  IF CIP509 Metadata is found it will be updated in `decoded_metadata`.
    pub(crate) fn decode_and_validate(
        decoded_metadata: &DecodedMetadata, txn: &MultiEraTx, raw_aux_data: &TransactionAuxData,
    ) {
        // Get the CIP509 metadata if possible
        let Some(k509) = raw_aux_data.metadata(MetadatumLabel::CIP509_RBAC) else {
            return;
        };

        let mut validation_report = ValidationReport::new();
        let mut decoder = Decoder::new(k509.as_ref());

        let cip509 = match RbacRegCip509::decode(&mut decoder, &mut ()) {
            Ok(metadata) => metadata,
            Err(e) => {
                Cip509::default().decoding_failed(
                    &format!("Failed to decode CIP509 metadata: {e}"),
                    &mut validation_report,
                    decoded_metadata,
                );
                return;
            },
        };

        // Validate the decoded metadata
        let validation = cip509.validate(txn, &mut validation_report);

        // Create a Cip509 struct and insert it into decoded_metadata
        decoded_metadata.0.insert(
            MetadatumLabel::CIP509_RBAC,
            Arc::new(DecodedMetadataItem {
                value: DecodedMetadataValues::Cip509(Arc::new(Cip509 { cip509, validation })),
                report: validation_report.clone(),
            }),
        );
    }

    /// Decoding of the CIP509 metadata failed, and can not continue.
    fn decoding_failed(
        &self, reason: &str, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata,
    ) {
        validation_report.push(reason.into());
        decoded_metadata.0.insert(
            MetadatumLabel::CIP509_RBAC,
            Arc::new(DecodedMetadataItem {
                value: DecodedMetadataValues::Cip509(Arc::new(self.clone()).clone()),
                report: validation_report.clone(),
            }),
        );
    }
}
