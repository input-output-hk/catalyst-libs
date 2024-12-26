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
#[derive(Clone, Debug, Default)]
pub struct Cip36 {
    pub cip36: Cip36Registration,
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
        decoded_metadata: &DecodedMetadata, slot: u64, txn: &MultiEraTx,
        raw_aux_data: &TransactionAuxData, catalyst_strict: bool, network: Network,
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
            Ok(metadata) => metadata,
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

        let cip36 = Cip36Registration {
            key_registration,
            registration_witness,
            is_catalyst_strict: catalyst_strict,
        };

        let validation = cip36.validate(network, &k61284, &mut validation_report);

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
// #[cfg(test)]
// mod tests {
//     use dashmap::DashMap;

//     use super::*;

//     fn create_empty_cip36(strict: bool) -> Cip36 {
//         Cip36 {
//             cip36: None,
//             voting_keys: vec![],
//             stake_pk: None,
//             payment_addr: vec![],
//             payable: false,
//             raw_nonce: 0,
//             nonce: 0,
//             purpose: 0,
//             signed: false,
//             strict_catalyst: strict,
//         }
//     }

//     #[test]
//     fn test_decode_purpose_1() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(true);
//         let mut decoder = Decoder::new(&[0x00]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_purpose(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.purpose, 0);
//         assert_eq!(rc, Some(0));
//     }

//     #[test]
//     fn test_decode_purpose_2() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(true);
//         let mut decoder = Decoder::new(&[0x19, 0x30, 0x39]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_purpose(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 1);
//         assert_eq!(cip36.purpose, 12345);
//         assert_eq!(rc, Some(12345));
//     }

//     #[test]
//     fn test_decode_purpose_3() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&[0x19, 0x30, 0x39]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_purpose(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.purpose, 12345);
//         assert_eq!(rc, Some(12345));
//     }

//     #[test]
//     fn test_decode_purpose_4() {
//         let bytes_cases: &[&[u8]] = &[
//             &[0x80],             // array(0)
//             &[0xA0],             // map(0)
//             &[0x21],             // negative(1)
//             &[0xF9, 0x3C, 0x00], // primitive(15360) - 1.0
//         ];

//         for bytes in bytes_cases {
//             let decoded_metadata = DecodedMetadata(DashMap::new());
//             let mut cip36 = create_empty_cip36(false);
//             let mut decoder = Decoder::new(bytes);
//             let mut report = ValidationReport::new();

//             let rc = cip36.decode_purpose(&mut decoder, &mut report, &decoded_metadata);

//             assert_eq!(report.len(), 1);
//             assert_eq!(cip36.purpose, 0);
//             assert_eq!(rc, None);
//         }
//     }

//     #[test]
//     // valid `nonce`, strict = false, raw_nonce > slot
//     fn test_decode_nonce_1() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&[0x1B, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_nonce(&mut decoder, &mut report, &decoded_metadata, 0);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.raw_nonce, u64::MAX);
//         assert_eq!(cip36.nonce, u64::MAX);
//         assert_eq!(rc, Some(u64::MAX));
//     }

//     #[test]
//     // valid `nonce`, strict = false, raw_nonce < slot
//     fn test_decode_nonce_2() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&[0x01]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_nonce(&mut decoder, &mut report, &decoded_metadata, 99);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.raw_nonce, 1);
//         assert_eq!(cip36.nonce, 1);
//         assert_eq!(rc, Some(1));
//     }

//     #[test]
//     // valid `nonce`, strict = true, raw_nonce > slot
//     fn test_decode_nonce_3() {
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(true);
//         let mut decoder = Decoder::new(&[0x10]);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_nonce(&mut decoder, &mut report, &decoded_metadata, 1);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.raw_nonce, 16);
//         assert_eq!(cip36.nonce, 1);
//         assert_eq!(rc, Some(1));
//     }

//     #[test]
//     fn test_decode_nonce_4() {
//         let bytes_cases: &[&[u8]] = &[
//             &[0x80],             // array(0)
//             &[0xA0],             // map(0)
//             &[0x21],             // negative(1)
//             &[0xF9, 0x3C, 0x00], // primitive(15360) - 1.0
//         ];

//         for bytes in bytes_cases {
//             let decoded_metadata = DecodedMetadata(DashMap::new());
//             let mut cip36 = create_empty_cip36(false);
//             let mut decoder = Decoder::new(bytes);
//             let mut report = ValidationReport::new();

//             let rc = cip36.decode_nonce(&mut decoder, &mut report, &decoded_metadata, 0);

//             assert_eq!(report.len(), 1);
//             assert_eq!(cip36.raw_nonce, 0);
//             assert_eq!(cip36.nonce, 0);
//             assert_eq!(rc, None);
//         }
//     }

//     #[test]
//     fn test_decode_payment_address_1() {
//         let hex_data = hex::decode(
//             // 0x004777561e7d9ec112ec307572faec1aff61ff0cfed68df4cd5c847f1872b617657881e30ad17c46e4010c9cb3ebb2440653a34d32219c83e9
//             "5839004777561E7D9EC112EC307572FAEC1AFF61FF0CFED68DF4CD5C847F1872B617657881E30AD17C46E4010C9CB3EBB2440653A34D32219C83E9"
//         ).expect("cannot decode hex");
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&hex_data);
//         let mut report = ValidationReport::new();
//         let multi_era_tx: *const MultiEraTx = std::ptr::null();
//         let multi_era_tx = unsafe { &*multi_era_tx };

//         let rc = cip36.decode_payment_address(
//             &mut decoder,
//             &mut report,
//             &decoded_metadata,
//             multi_era_tx,
//             Network::Preprod,
//         );

//         assert_eq!(report.len(), 0);
//         assert!(cip36.payable);
//         assert_eq!(cip36.payment_addr.len(), 57);
//         assert_eq!(rc, Some(57));
//     }

//     #[test]
//     fn test_decode_stake_pub_1() {
//         let hex_data = hex::decode(
//             // 0xe3cd2404c84de65f96918f18d5b445bcb933a7cda18eeded7945dd191e432369
//             "5820E3CD2404C84DE65F96918F18D5B445BCB933A7CDA18EEDED7945DD191E432369",
//         )
//         .expect("cannot decode hex");
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&hex_data);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_stake_pub(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 0);
//         assert!(cip36.stake_pk.is_some());
//         assert_eq!(rc, Some(1));
//     }

//     #[test]
//     fn test_decode_stake_pub_2() {
//         let bytes_cases: &[Vec<u8>] = &[
//             vec![],
//             hex::decode(
//                 // 0xe3cd2404c84de65f96918f18d5b445bcb933a7cda18eeded7945dd19 (28 bytes)
//                 "581CE3CD2404C84DE65F96918F18D5B445BCB933A7CDA18EEDED7945DD19",
//             )
//             .expect("cannot decode hex"),
//         ];

//         for bytes in bytes_cases {
//             let decoded_metadata = DecodedMetadata(DashMap::new());
//             let mut cip36 = create_empty_cip36(false);
//             let mut decoder = Decoder::new(bytes);
//             let mut report = ValidationReport::new();

//             let rc = cip36.decode_stake_pub(&mut decoder, &mut report, &decoded_metadata);

//             assert_eq!(report.len(), 1);
//             assert_eq!(rc, None);
//         }
//     }

//     #[test]
//     // cip-36 version
//     fn test_decode_voting_key_1() {
//         let hex_data = hex::decode(
//             // [["0x0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0", 1]]
//             "818258200036EF3E1F0D3F5989E2D155EA54BDB2A72C4C456CCB959AF4C94868F473F5A001",
//         )
//         .expect("cannot decode hex");
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&hex_data);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_voting_key(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.cip36, Some(true));
//         assert_eq!(cip36.voting_keys.len(), 1);
//         assert_eq!(rc, Some(1));
//     }

//     #[test]
//     // cip-15 version
//     fn test_decode_voting_key_2() {
//         let hex_data = hex::decode(
//             // 0x0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0
//             "58200036EF3E1F0D3F5989E2D155EA54BDB2A72C4C456CCB959AF4C94868F473F5A0",
//         )
//         .expect("cannot decode hex");
//         let decoded_metadata = DecodedMetadata(DashMap::new());
//         let mut cip36 = create_empty_cip36(false);
//         let mut decoder = Decoder::new(&hex_data);
//         let mut report = ValidationReport::new();

//         let rc = cip36.decode_voting_key(&mut decoder, &mut report, &decoded_metadata);

//         assert_eq!(report.len(), 0);
//         assert_eq!(cip36.cip36, Some(false));
//         assert_eq!(cip36.voting_keys.len(), 1);
//         assert_eq!(rc, Some(1));
//     }

//     #[test]
//     fn test_decode_voting_key_3() {
//         let bytes_cases: &[Vec<u8>] = &[
//             vec![],
//             hex::decode(
//                 // [[]] (empty)
//                 "8180",
//             )
//             .expect("cannot decode hex"),
//             hex::decode(
//                 // [["0x0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0"]]
//                 // (without weight)
//                 "818158200036EF3E1F0D3F5989E2D155EA54BDB2A72C4C456CCB959AF4C94868F473F5A0",
//             )
//             .expect("cannot decode hex"),
//         ];

//         for bytes in bytes_cases {
//             let decoded_metadata = DecodedMetadata(DashMap::new());
//             let mut cip36 = create_empty_cip36(false);
//             let mut decoder = Decoder::new(bytes);
//             let mut report = ValidationReport::new();

//             let rc = cip36.decode_voting_key(&mut decoder, &mut report, &decoded_metadata);

//             assert_eq!(report.len(), 1);
//             assert_eq!(rc, None);
//         }
//     }
// }
