//! Decoder and Validator for CIP36 Metadata

use std::sync::Arc;

use cardano_blockchain_types::{
    Cip36 as Cip36Registration, Cip36KeyRegistration, Cip36RegistrationWitness, Cip36Validation,
    MetadatumLabel, Network, TransactionAuxData,
};
use minicbor::{Decode, Decoder};
use pallas::ledger::traverse::MultiEraTx;

use super::{DecodedMetadata, DecodedMetadataItem, DecodedMetadataValues, ValidationReport};

/// CIP 36 Registration Data.
#[derive(Clone, Default, Debug)]
pub struct Cip36 {
    /// CIP36 data.
    pub cip36: Cip36Registration,
    /// Validation value, not a part of CIP36, just storing validity of the data.
    pub validation: Cip36Validation,
}

impl Cip36 {
    /// Decode and validate CIP36/15 Metadata
    ///
    /// CIP15 is a subset of CIP36.
    ///
    /// See:
    /// * <https://cips.cardano.org/cip/CIP-36>
    /// * <https://github.com/cardano-foundation/CIPs/tree/master/CIP-0036>
    ///
    /// # Parameters
    /// * `decoded_metadata` - Decoded Metadata - Will be updated only if CIP36 Metadata
    ///   is found.
    /// * `slot` - Current Slot
    /// * `txn` - Transaction data was attached to and to be validated/decoded against.
    ///   Not used for CIP36 Metadata.
    /// * `raw_aux_data` - Raw Auxiliary Data for the transaction.
    /// * `catalyst_strict` - Strict Catalyst Validation - otherwise Catalyst Specific
    ///   rules/workarounds are not applied.
    /// * `network` - Network Chain
    ///
    /// # Returns
    ///
    /// Nothing.  IF CIP36 Metadata is found it will be updated in `decoded_metadata`.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn decode_and_validate(
        decoded_metadata: &DecodedMetadata, slot: u64, _txn: &MultiEraTx,
        raw_aux_data: &TransactionAuxData, is_catalyst_strict: bool, network: Network,
    ) {
        let Some(k61284) = raw_aux_data.metadata(MetadatumLabel::CIP036_REGISTRATION) else {
            return;
        };
        let Some(k61285) = raw_aux_data.metadata(MetadatumLabel::CIP036_WITNESS) else {
            return;
        };

        let mut validation_report = ValidationReport::new();
        let mut key_registration = Decoder::new(k61284.as_ref());
        let mut registration_witness = Decoder::new(k61285.as_ref());

        let key_registration = match Cip36KeyRegistration::decode(&mut key_registration, &mut ()) {
            Ok(mut metadata) => {
                // FIXME: Don't like it here
                let nonce = if is_catalyst_strict && metadata.raw_nonce > slot {
                    slot
                } else {
                    metadata.raw_nonce
                };

                metadata.nonce = nonce;
                metadata
            },
            Err(e) => {
                Cip36::default().decoding_failed(
                    &format!("Failed to decode CIP36 Key Registration metadata: {e}"),
                    &mut validation_report,
                    decoded_metadata,
                    MetadatumLabel::CIP036_REGISTRATION,
                );
                return;
            },
        };

        let registration_witness =
            match Cip36RegistrationWitness::decode(&mut registration_witness, &mut ()) {
                Ok(metadata) => metadata,
                Err(e) => {
                    Cip36::default().decoding_failed(
                        &format!("Failed to decode CIP36 Registration Witness metadata: {e}"),
                        &mut validation_report,
                        decoded_metadata,
                        MetadatumLabel::CIP036_WITNESS,
                    );
                    return;
                },
            };

        let cip36 =
            Cip36Registration::new(key_registration, registration_witness, is_catalyst_strict);

        let validation = cip36.validate(network, k61284, &mut validation_report);

        // Create a Cip509 struct and insert it into decoded_metadata
        decoded_metadata.0.insert(
            MetadatumLabel::CIP036_REGISTRATION,
            Arc::new(DecodedMetadataItem {
                value: DecodedMetadataValues::Cip36(Arc::new(Cip36 { cip36, validation })),
                report: validation_report.clone(),
            }),
        );
    }

    /// Decoding of the CIP36 metadata failed, and can not continue.
    fn decoding_failed(
        &self, reason: &str, validation_report: &mut ValidationReport,
        decoded_metadata: &DecodedMetadata, label: MetadatumLabel,
    ) {
        validation_report.push(reason.into());
        decoded_metadata.0.insert(
            label,
            Arc::new(DecodedMetadataItem {
                value: DecodedMetadataValues::Cip36(Arc::new(self.clone()).clone()),
                report: validation_report.clone(),
            }),
        );
    }
}
