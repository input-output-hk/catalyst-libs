//! Cardano Improvement Proposal 509 (CIP-509) metadata module.
//! Doc Reference: <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
//! CDDL Reference: <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>

// cspell: words pkix

pub mod rbac;
pub(crate) mod utils;
pub(crate) mod validation;
pub mod x509_chunks;

use minicbor::{
    decode::{self},
    Decode, Decoder,
};
use pallas::{crypto::hash::Hash, ledger::traverse::MultiEraTx};
use strum_macros::FromRepr;
use validation::{
    validate_aux, validate_payment_key, validate_role_singing_key, validate_stake_public_key,
    validate_txn_inputs_hash,
};
use x509_chunks::X509Chunks;

use super::transaction::witness::TxWitness;
use crate::utils::{
    decode_helper::{decode_bytes, decode_helper, decode_map_len},
    general::{decode_utf8, decremented_index},
    hashing::{blake2b_128, blake2b_256},
};

/// CIP509 label.
pub const LABEL: u64 = 509;

/// CIP509.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cip509 {
    /// `UUIDv4` Purpose .
    pub purpose: UuidV4, // (bytes .size 16)
    /// Transaction inputs hash.
    pub txn_inputs_hash: TxInputHash, // bytes .size 16
    /// Optional previous transaction ID.
    pub prv_tx_id: Option<Hash<32>>, // bytes .size 32
    /// x509 chunks.
    pub x509_chunks: X509Chunks, // chunk_type => [ + x509_chunk ]
    /// Validation signature.
    pub validation_signature: Vec<u8>, // bytes size (1..64)
}

/// `UUIDv4` representing in 16 bytes.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct UuidV4([u8; 16]);

impl From<[u8; 16]> for UuidV4 {
    fn from(bytes: [u8; 16]) -> Self {
        UuidV4(bytes)
    }
}

impl TryFrom<Vec<u8>> for UuidV4 {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(&vec);
            Ok(UuidV4(array))
        } else {
            Err("Input Vec must be exactly 16 bytes")
        }
    }
}

/// Transaction input hash representing in 16 bytes.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TxInputHash([u8; 16]);

impl From<[u8; 16]> for TxInputHash {
    fn from(bytes: [u8; 16]) -> Self {
        TxInputHash(bytes)
    }
}

impl TryFrom<Vec<u8>> for TxInputHash {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(&vec);
            Ok(TxInputHash(array))
        } else {
            Err("Input Vec must be exactly 16 bytes")
        }
    }
}

/// Enum of CIP509 metadatum with its associated unsigned integer value.
#[allow(clippy::module_name_repetitions)]
#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub(crate) enum Cip509IntIdentifier {
    /// Purpose.
    Purpose = 0,
    /// Transaction inputs hash.
    TxInputsHash = 1,
    /// Previous transaction ID.
    PreviousTxId = 2,
    /// Validation signature.
    ValidationSignature = 99,
}

impl Decode<'_, ()> for Cip509 {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP509")?;
        let mut cip509_metadatum = Cip509::default();
        for _ in 0..map_len {
            // Use probe to peak
            let key = d.probe().u8()?;
            if let Some(key) = Cip509IntIdentifier::from_repr(key) {
                // Consuming the int
                let _: u8 = decode_helper(d, "CIP509", ctx)?;
                match key {
                    Cip509IntIdentifier::Purpose => {
                        cip509_metadatum.purpose =
                            UuidV4::try_from(decode_bytes(d, "CIP509 purpose")?).map_err(|_| {
                                decode::Error::message("Invalid data size of Purpose")
                            })?;
                    },
                    Cip509IntIdentifier::TxInputsHash => {
                        cip509_metadatum.txn_inputs_hash =
                            TxInputHash::try_from(decode_bytes(d, "CIP509 txn inputs hash")?)
                                .map_err(|_| {
                                    decode::Error::message("Invalid data size of TxInputsHash")
                                })?;
                    },
                    Cip509IntIdentifier::PreviousTxId => {
                        let prv_tx_hash: [u8; 32] = decode_bytes(d, "CIP509 previous tx ID")?
                            .try_into()
                            .map_err(|_| {
                                decode::Error::message("Invalid data size of PreviousTxId")
                            })?;
                        cip509_metadatum.prv_tx_id = Some(Hash::from(prv_tx_hash));
                    },
                    Cip509IntIdentifier::ValidationSignature => {
                        let validation_signature = decode_bytes(d, "CIP509 validation signature")?;
                        if validation_signature.is_empty() || validation_signature.len() > 64 {
                            return Err(decode::Error::message(
                                "Invalid data size of ValidationSignature",
                            ));
                        }
                        cip509_metadatum.validation_signature = validation_signature;
                    },
                }
            } else {
                // Handle the x509 chunks 10 11 12
                let x509_chunks = X509Chunks::decode(d, ctx)?;
                cip509_metadatum.x509_chunks = x509_chunks;
            }
        }
        Ok(cip509_metadatum)
    }
}

impl Cip509 {
    /// Basic validation for CIP509
    /// The validation include the following:
    /// * Hashing the transaction inputs within the transaction should match the
    ///   txn-inputs-hash
    /// * Auxiliary data hash within the transaction should match the hash of the
    ///   auxiliary data itself.
    /// * Public key validation for role 0 where public key extracted from x509 and c509
    ///   subject alternative name should match one of the witness in witness set within
    ///   the transaction.
    /// * Payment key reference validation for role 0 where the reference should be either
    ///     1. Negative index reference - reference to transaction output in transaction:
    ///        should match some of the key within witness set.
    ///     2. Positive index reference - reference to the transaction input in
    ///        transaction: only check whether the index exist within the transaction
    ///        inputs.
    /// * Role signing key validation for role 0 where the signing keys should only be the certificates
    ///
    ///  See:
    /// * <https://github.com/input-output-hk/catalyst-CIPs/tree/x509-envelope-metadata/CIP-XXXX>
    /// * <https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/x509-envelope.cddl>
    ///
    /// Note: This CIP509 is still under development and is subject to change.
    ///
    /// # Parameters
    /// * `txn` - Transaction data was attached to and to be validated/decoded against.
    /// * `txn_idx` - Transaction Index
    /// * `validation_report` - Validation report to store the validation result.
    pub fn validate(
        &self, txn: &MultiEraTx, txn_idx: usize, validation_report: &mut Vec<String>,
    ) -> bool {
        let tx_input_validate =
            validate_txn_inputs_hash(self, txn, validation_report).unwrap_or(false);
        let aux_validate = validate_aux(txn, validation_report).unwrap_or(false);
        let mut stake_key_validate = true;
        let mut payment_key_validate = true;
        let mut signing_key = true;
        // Validate the role 0
        if let Some(role_set) = &self.x509_chunks.0.role_set {
            // Validate only role 0
            for role in role_set {
                if role.role_number == 0 {
                    stake_key_validate =
                        validate_stake_public_key(self, txn, txn_idx, validation_report)
                            .unwrap_or(false);
                    payment_key_validate =
                        validate_payment_key(txn, txn_idx, role, validation_report)
                            .unwrap_or(false);
                    signing_key = validate_role_singing_key(role, validation_report);
                }
            }
        }
        tx_input_validate
            && aux_validate
            && stake_key_validate
            && payment_key_validate
            && signing_key
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode_cip509() {
        // This data is from conway_1.block
        let cip_509 = "a50050ca7a1457ef9f4c7f9c747f8c4a4cfa6c0150226d126819472b7afad7d0b8c7b89aa20258204d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c20b9458401b03060066006fd5b67002167882eac0b5f2b11da40788a39bfa0324c494f7003a6b4c1c4bac378e322cb280230a4002f5b2754e863806f7e524afc99996aa28584032f02b600cbf04c6a09e05100880a09ee59b6627dc78d68175469b8c5b1fac141a6da5c6c2ea446597b6f0b6efea00a04ac0c1756455589908a5e089ba604a1258405917d6ee2b2535959d806c00eb2958929ababb40d681b5245751538e915d3d90f561ddcaa9aaa9cd78a30882a22a99c742c4f7610b43750a0d6651e8640a8d4c58402167427cfa933d6430c026640888210cd0c4e93e7015100300dcaef47b9c155ea4ccb27773c27f5d6a44fbf98065a14e5f0eca530e57082a971cbf22fa9065585840ae72e2a061eb558d3fd7727e87a8f07b5faf0d3cedf8d99ab6e0c845f5dd3ce78d31d7365c523b5a4dfe5d35bfafaefb2f60dd7473cbe8d6aa6bf557b1fbdf775840bf96bcd3ffdbfc7d20b65be7f5c7dba1cf635e3b449bb644bdfc73e0e49a5db73edddc7ed331220ba732f62f3aee8503f5c6f9bd5f7fedb37dc6580196052e50584027fdd7e8bfe9146561ad1ebc79ecef0ee1df7081cf9cd1fd929569ef3d55972d5b7ff882ce2213f789fc08787164f14aa86d55e98e332b220a07fa464aaa7c335840ce4bcfb268ed577f72e87fdea4442107bf2da93fe05121d5befa7ae5aecc5f3f9c732e82108003166380198c0146b0e214114a31d7c62b0ec18afd5834034c2b58402b2c515b350d8980a16932071b6d8d125ea1eb53dc28a8aee1787a1670b9e8c4c8cb00c726f3515a39ca1689f870295752820a64721e05e1a234710583416316584031d80291ac9a2b66a04cba844b85f9928a4a04a9928b2805124a25b3aaa4422e45e5d422a9b88a028ba4a5123ac244b8b472164b86085ac21357c3aae7696be25840f1104878009b03813d9e6c53255722402090206058a009d2b808aff772fb712d75f1dea09507fd83838e045dd9ce3eb59e4554f5ed02b8aeb60700f4b39dd9fe584064e1d5a137de0fa4c6cccfd71f831bee372756d72990b357a44e2f9eaf3854db65379db466cfcb55517ba71550acade564f4b7efd1fd95fa57228cee6fa9ae3458405ce1ae79b77f7cd5bdecfcb800fbdb7eaf720eae5995176d94a07c326c71aaf5e6f8439e577edb2d1ed64959324b5a7476e9159bf37bdf226edb747787b79b9e5840bc6ab5b84714eefa4a8c2df4aba37a36757d8b39dd79ec41b4a2f3ee96eabdc0e1f65b37264bdbfdf79eebbc820a7deab4e39f7e1cbf6610402fd8fb55fbef3d584038226e4d37c42970c830184b2e1c5026eadb9677ae8f6d300975ca6ceec5c8920382e827c1f636f7dd9f8d492737f4520a944bfeebba5ca2d5efa80ad453a43f584004c357ecccfc4dab75ce560b0300db9092ced52625d0c8df6fc89da9a45b6dc9c2461f21e6ee7b7afd877fbd8c1a1fa7ff38fa506e14749ebb68e24571c6220c584004208c284d628c2148b252f91b8b50014b080b040554095b52ca862bb974218222d412112ae5d2584c54584ae157f22b183cb4ba9c5fc42ba6894ad074ffe0875840c69ee921211d0ce4cd0f89b7e708163b3ab9286fe26a8c68ed85930cabc5dbfed7f9681c535dbdbfeb56f7a2b32d1f43de1dbcc934676edefacb3df7c1210067584064a1b8d94448b7f22a77dc736edb12f7c2c52b2eb8d4a80b78147d89f9a3a0659c03e10bbb336e391b3961f1afbfa08af3de2a817fceddea0cb57f438b0f8947581e9782ee92e890df65636d835d2d465cc5521c0ec05470e002800015eecf5818635840e0427f23196c17cf13f030595335343030c11d914bc7a84b56af7040930af4110fd4ca29b0bc0e83789adb8668ea2ef28c1dd10dc1fd35ea6ae8c06ee769540d";
        let binding = hex::decode(cip_509).unwrap();
        let mut decoder = Decoder::new(binding.as_slice());
        let decoded_cip509 = Cip509::decode(&mut decoder, &mut ()).unwrap();

        let purpose: [u8; 16] = hex::decode("ca7a1457ef9f4c7f9c747f8c4a4cfa6c")
            .unwrap()
            .try_into()
            .unwrap();
        let txn_inputs_hash: [u8; 16] = hex::decode("226d126819472b7afad7d0b8c7b89aa2")
            .unwrap()
            .try_into()
            .unwrap();
        let prv_tx_id: [u8; 32] =
            hex::decode("4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2")
                .unwrap()
                .try_into()
                .unwrap();
        let validation_signature = hex::decode("e0427f23196c17cf13f030595335343030c11d914bc7a84b56af7040930af4110fd4ca29b0bc0e83789adb8668ea2ef28c1dd10dc1fd35ea6ae8c06ee769540d").unwrap();

        assert_eq!(decoded_cip509.purpose, UuidV4(purpose));
        assert_eq!(decoded_cip509.txn_inputs_hash, TxInputHash(txn_inputs_hash));
        assert_eq!(decoded_cip509.prv_tx_id, Some(prv_tx_id.into()));
        assert_eq!(decoded_cip509.validation_signature, validation_signature);
    }
}
