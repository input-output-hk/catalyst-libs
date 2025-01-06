//! CIP-36 Key Registration 61284.
//!
//! Catalyst registration data
//!
//! <https://cips.cardano.org/cip/CIP-36>
//! <https://github.com/cardano-foundation/CIPs/blob/master/CIP-0036/schema.cddl>

use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::{decode_array_len, decode_bytes, decode_helper, decode_map_len};
use ed25519_dalek::VerifyingKey;
use minicbor::{decode, Decode, Decoder};
use pallas::ledger::addresses::{Address, ShelleyAddress};
use strum::FromRepr;

use super::voting_pk::VotingPubKey;

/// CIP-36 key registration - 61284
///
///
/// ```cddl
/// key_registration = {
///     1 : [+delegation] / legacy_key_registration,
///     2 : $stake_credential,
///     3 : $payment_address,
///     4 : $nonce,
///     ? 5 : $voting_purpose .default 0
//   }
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
pub(crate) struct Cip36KeyRegistration {
    /// Is this CIP36 or CIP15 format.
    /// None if not either CIP36 or CIP15.
    pub is_cip36: Option<bool>,
    /// Voting public keys (called Delegations in the CIP-36 Spec).
    /// Field 1 in the CIP-36 61284 Spec.
    pub voting_pks: Vec<VotingPubKey>,
    /// Stake public key to associate with the voting keys.
    /// Field 2 in the CIP-36 61284 Spec.
    /// None if it is not set.
    pub stake_pk: Option<VerifyingKey>,
    /// Payment Address to associate with the voting keys.
    /// Field 3 in the CIP-36 61284 Spec.
    /// None if it is not set.
    pub payment_addr: Option<ShelleyAddress>,
    /// Nonce (nonce that has been slot corrected).
    /// Field 4 in the CIP-36 61284 Spec.
    /// None if it is not set.
    pub nonce: Option<u64>,
    /// Registration Purpose (Always 0 for Catalyst).
    /// Field 5 in the CIP-36 61284 Spec.
    /// Default to 0.
    pub purpose: u64,
    /// Raw nonce (nonce that has not had slot correction applied).
    /// None if it is not set.
    pub raw_nonce: Option<u64>,
    /// Is payment address payable? (not a script)
    /// None if it is not set.
    pub is_payable: Option<bool>,
}

/// Enum of CIP36 registration (61284) with its associated unsigned integer key.
#[derive(FromRepr, Debug, PartialEq)]
#[repr(u16)]
enum Cip36KeyRegistrationKeys {
    /// Voting key.
    VotingKey = 1,
    /// Stake public key.
    StakePk = 2,
    /// Payment address.
    PaymentAddr = 3,
    /// Nonce.
    Nonce = 4,
    /// Purpose.
    Purpose = 5,
}

impl Decode<'_, ()> for Cip36KeyRegistration {
    #[allow(clippy::too_many_lines)]
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP36 Key Registration")?;

        let mut cip36_key_registration = Cip36KeyRegistration::default();

        // Record of founded keys. Check for duplicate keys in the map
        let mut found_keys: Vec<Cip36KeyRegistrationKeys> = Vec::new();

        // Record of errors found during decoding
        let err_report = ProblemReport::new("CIP36 Key Registration Decoding");

        for index in 0..map_len {
            let key: u16 = decode_helper(d, "key in CIP36 Key Registration", ctx)?;

            if let Some(key) = Cip36KeyRegistrationKeys::from_repr(key) {
                match key {
                    Cip36KeyRegistrationKeys::VotingKey => {
                        if found_keys.contains(&key) {
                            err_report.duplicate_field(
                                "Key 1",
                                format!("Redundant key found in item {} in RBAC map", index + 1)
                                    .as_str(),
                                "CIP36 Key Registration voting key",
                            );
                            continue;
                        }
                        if let Some((is_cip36, voting_keys)) = decode_voting_key(d, &err_report)? {
                            cip36_key_registration.is_cip36 = is_cip36;
                            cip36_key_registration.voting_pks = voting_keys;
                        }
                    },
                    Cip36KeyRegistrationKeys::StakePk => {
                        if found_keys.contains(&key) {
                            err_report.duplicate_field(
                                "Key 2",
                                format!("Redundant key found in item {} in RBAC map", index + 1)
                                    .as_str(),
                                "CIP36 Key Registration stake public key",
                            );
                            continue;
                        }
                        if let Some(stake_pk) = decode_stake_pk(d, &err_report)? {
                            cip36_key_registration.stake_pk = Some(stake_pk);
                        }
                    },
                    Cip36KeyRegistrationKeys::PaymentAddr => {
                        if found_keys.contains(&key) {
                            err_report.duplicate_field(
                                "Key 3",
                                format!("Redundant key found in item {} in RBAC map", index + 1)
                                    .as_str(),
                                "CIP36 Key Registration payment address",
                            );
                            continue;
                        }
                        if let Some(shelley_addr) = decode_payment_addr(d, &err_report)? {
                            cip36_key_registration.payment_addr = Some(shelley_addr.clone());
                            cip36_key_registration.is_payable =
                                Some(!shelley_addr.payment().is_script());
                        }
                    },
                    Cip36KeyRegistrationKeys::Nonce => {
                        if found_keys.contains(&key) {
                            err_report.duplicate_field(
                                "Key 4",
                                format!("Redundant key found in item {} in RBAC map", index + 1)
                                    .as_str(),
                                "CIP36 Key Registration nonce",
                            );
                            continue;
                        }
                        cip36_key_registration.nonce = Some(decode_nonce(d)?);
                    },
                    Cip36KeyRegistrationKeys::Purpose => {
                        if found_keys.contains(&key) {
                            err_report.duplicate_field(
                                "Key 5",
                                format!("Redundant key found in item {} in RBAC map", index + 1)
                                    .as_str(),
                                "CIP36 Key Registration purpose",
                            );
                            continue;
                        }
                        cip36_key_registration.purpose = decode_purpose(d)?;
                    },
                }
                // Update the founded keys.
                found_keys.push(key);
            }
        }

        if !found_keys.contains(&Cip36KeyRegistrationKeys::VotingKey) {
            err_report.missing_field(
                "Voting Key",
                "Missing required key in CIP36 Key Registration",
            );
        }

        if !found_keys.contains(&Cip36KeyRegistrationKeys::StakePk) {
            err_report.missing_field(
                "Stake Public Key",
                "Missing required key in CIP36 Key Registration",
            );
        }

        if !found_keys.contains(&Cip36KeyRegistrationKeys::PaymentAddr) {
            err_report.missing_field(
                "Payment Address",
                "Missing required key in CIP36 Key Registration",
            );
        }

        if !found_keys.contains(&Cip36KeyRegistrationKeys::Nonce) {
            err_report.missing_field("Nonce", "Missing required key in CIP36 Key Registration");
        }

        if err_report.problematic() {
            return Err(decode::Error::message(
                serde_json::to_string(&err_report)
                    .unwrap_or_else(|_| "Failed to serialize ProblemReport".to_string()),
            ));
        }

        Ok(cip36_key_registration)
    }
}

/// Helper function for decoding the voting key.
///
/// # Returns
///
/// - A tuple containing
///     - A boolean value, true if it is CIP36 format, false if it is CIP15, None if not
///       either CIP36 or CIP15.
///     - A vector of `VotingPubKey`, if the `voting_pk` vector cannot be converted to
///       verifying key, assign `voting_pk` to None.
/// - None if there is invalid data.
/// - Error if decoding failed.
#[allow(clippy::type_complexity)]
fn decode_voting_key(
    d: &mut Decoder, err_report: &ProblemReport,
) -> Result<Option<(Option<bool>, Vec<VotingPubKey>)>, decode::Error> {
    let mut voting_keys = Vec::new();
    #[allow(unused_assignments)]
    let mut is_cip36 = None;

    match d.datatype() {
        Ok(dt) => {
            match dt {
                // CIP15 type registration (single voting key).
                // ```cddl
                //      legacy_key_registration = $cip36_vote_pub_key
                //      $cip36_vote_pub_key /= bytes .size 32
                // ```
                minicbor::data::Type::Bytes => {
                    is_cip36 = Some(false);
                    let pub_key =
                        decode_bytes(d, "CIP36 Key Registration voting key, single voting key")?;
                    let vk = voting_pk_vec_to_verifying_key(
                        &pub_key,
                        err_report,
                        "CIP36 Key Registration voting key, single voting key",
                    );
                    // Since there is 1 voting key, all the weight goes to this key = 1.
                    voting_keys.push(VotingPubKey::new(vk, 1));
                },
                // CIP36 type registration (multiple voting keys).
                // ```cddl
                //      [+delegation]
                //      delegation = [$cip36_vote_pub_key, $weight]
                //      $cip36_vote_pub_key /= bytes .size 32
                // ```
                minicbor::data::Type::Array => {
                    is_cip36 = Some(true);
                    let len = decode_array_len(
                        d,
                        "CIP36 Key Registration voting key, multiple voting keys",
                    )?;

                    for _ in 0..len {
                        let len =
                            decode_array_len(d, "CIP36 Key Registration voting key, delegations")?;
                        // This fixed array should be a length of 2 (voting key, weight).
                        if len != 2 {
                            return Err(decode::Error::message(format!("Invalid length for CIP36 Key Registration voting key delegations, expected 2, got {len}")));
                        }

                        // The first entry.
                        let pub_key = decode_bytes(d, "CIP36 Key Registration voting key, delegation array first entry (voting public key)")?;
                        // The second entry.
                        let weight: u32 = decode_helper(d, "CIP36 Key Registration voting key, delegation array second entry (weight)", &mut (),)?;

                        let vk = voting_pk_vec_to_verifying_key(
                            &pub_key,
                            err_report,
                            "CIP36 Key Registration voting key, multiple voting keys",
                        );
                        voting_keys.push(VotingPubKey::new(vk, weight));
                    }
                },

                _ => {
                    return Err(decode::Error::message("Invalid datatype for CIP36 Key Registration voting key, should be either Array or Bytes"));
                },
            }
        },
        Err(e) => {
            return Err(decode::Error::message(format!(
                "Decoding voting key, invalid data type: {e}"
            )));
        },
    }
    Ok(Some((is_cip36, voting_keys)))
}

/// Helper function for converting `&[u8]` to `VerifyingKey`.
fn voting_pk_vec_to_verifying_key(
    pub_key: &[u8], err_report: &ProblemReport, context: &str,
) -> Option<VerifyingKey> {
    let bytes = pub_key
        .try_into()
        .map_err(|_| {
            err_report.invalid_value(
                "Verifying key length",
                format!("{}", pub_key.len()).as_str(),
                "Invalid length, must be length 32",
                context,
            );
        })
        .ok()?;
    VerifyingKey::from_bytes(bytes)
        .map_err(|e| {
            err_report.other(
                format!("Failed to convert to VerifyingKey: {e:?}").as_str(),
                context,
            );
        })
        .ok()
}

/// Helper function for decoding the stake public key.
///
/// ```cddl
///     2 : $stake_credential,
///     $stake_credential /= $staking_pub_key
///     $staking_pub_key /= bytes .size 32
/// ```
///
/// # Returns
///
/// - The stake public key as a `VerifyingKey`.
/// - None if cannot converted `Vec<u8>` to `VerifyingKey`.
/// - Error if decoding failed.
fn decode_stake_pk(
    d: &mut Decoder, err_report: &ProblemReport,
) -> Result<Option<VerifyingKey>, decode::Error> {
    let pub_key = decode_bytes(d, "CIP36 Key Registration stake public key")?;
    Ok(voting_pk_vec_to_verifying_key(
        &pub_key,
        err_report,
        "CIP36 Key Registration stake public key",
    ))
}

/// Helper function for decoding the payment address.
///
/// ```cddl
///   3 : $payment_address,
///   $payment_address /= bytes
/// ```
///
/// # Returns
///
/// - The payment address as a `ShelleyAddress`.
/// - None if cannot converted `Vec<u8>` to `ShelleyAddress`.
/// - Error if decoding failed.
fn decode_payment_addr(
    d: &mut Decoder, err_report: &ProblemReport,
) -> Result<Option<ShelleyAddress>, decode::Error> {
    let raw_addr = decode_bytes(d, "CIP36 Key Registration payment address")?;
    // Cannot convert raw address to Address type
    let address = match Address::from_bytes(&raw_addr) {
        Ok(addr) => addr,
        Err(e) => {
            err_report.other(
                format!("Cannot convert to type Address: {e}").as_str(),
                "CIP36 Key Registration payment address",
            );
            // Can't process any further
            return Ok(None);
        },
    };

    if let Address::Shelley(addr) = address {
        Ok(Some(addr.clone()))
    } else {
        err_report.invalid_value(
            "Shelley Address",
            format!("{address}").as_str(),
            "Expected Shelley address",
            "CIP36 Key Registration payment address",
        );
        Ok(None)
    }
}

/// Helper function for decoding raw nonce.
///
/// ```cddl
///     4 : $nonce,
///     $nonce /= uint
/// ```
///
/// # Returns
///
/// - Raw nonce.
/// - Error if decoding failed.
fn decode_nonce(d: &mut Decoder) -> Result<u64, decode::Error> {
    decode_helper(d, "CIP36 Key Registration nonce", &mut ())
}

/// Helper function for decoding the purpose.
///
/// ```cddl
///    5 : $voting_purpose .default 0
///    $voting_purpose /= uint
/// ```
///
/// # Returns
///
/// - The purpose.
/// - Error if decoding failed.
fn decode_purpose(d: &mut Decoder) -> Result<u64, decode::Error> {
    decode_helper(d, "CIP36 Key Registration purpose", &mut ())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode_payment_address() {
        let hex_data = hex::decode(
            // 0x004777561e7d9ec112ec307572faec1aff61ff0cfed68df4cd5c847f1872b617657881e30ad17c46e4010c9cb3ebb2440653a34d32219c83e9
            "5839004777561E7D9EC112EC307572FAEC1AFF61FF0CFED68DF4CD5C847F1872B617657881E30AD17C46E4010C9CB3EBB2440653A34D32219C83E9"
        ).expect("cannot decode hex");
        let mut decoder = Decoder::new(&hex_data);
        let err_report = ProblemReport::new("CIP36 Key Registration Decoding");
        let address =
            decode_payment_addr(&mut decoder, &err_report).expect("cannot decode payment address");
        assert!(!err_report.problematic());
        assert_eq!(address.unwrap().to_vec().len(), 57);
    }

    #[test]
    fn test_decode_stake_pk() {
        let hex_data = hex::decode(
            // 0xe3cd2404c84de65f96918f18d5b445bcb933a7cda18eeded7945dd191e432369
            "5820E3CD2404C84DE65F96918F18D5B445BCB933A7CDA18EEDED7945DD191E432369",
        )
        .expect("cannot decode hex");
        let mut decoder = Decoder::new(&hex_data);
        let err_report = ProblemReport::new("CIP36 Key Registration Decoding");
        let stake_pk = decode_stake_pk(&mut decoder, &err_report).expect("cannot decode stake pk");
        assert!(!err_report.problematic());
        assert!(stake_pk.is_some());
    }

    #[test]
    // cip-36 version
    fn test_decode_voting_key_cip36() {
        let hex_data = hex::decode(
            // [["0x0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0", 1]]
            "818258200036EF3E1F0D3F5989E2D155EA54BDB2A72C4C456CCB959AF4C94868F473F5A001",
        )
        .expect("cannot decode hex");
        let mut decoder = Decoder::new(&hex_data);
        let err_report = ProblemReport::new("CIP36 Key Registration Decoding");
        let (is_cip36, voting_pk) = decode_voting_key(&mut decoder, &err_report)
            .expect("cannot decode voting key")
            .unwrap();
        assert!(!err_report.problematic());
        assert!(is_cip36.unwrap());
        assert_eq!(voting_pk.len(), 1);
    }

    #[test]
    // cip-15 version
    fn test_decode_voting_key_2() {
        let hex_data = hex::decode(
            // 0x0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0
            "58200036EF3E1F0D3F5989E2D155EA54BDB2A72C4C456CCB959AF4C94868F473F5A0",
        )
        .expect("cannot decode hex");
        let mut decoder = Decoder::new(&hex_data);
        let err_report = ProblemReport::new("CIP36 Key Registration Decoding");
        let (is_cip36, voting_pk) = decode_voting_key(&mut decoder, &err_report)
            .expect("cannot decode voting key")
            .unwrap();
        assert!(!err_report.problematic());
        assert!(!is_cip36.unwrap());
        assert_eq!(voting_pk.len(), 1);
    }

    #[test]
    fn test_decode_nonce() {
        let hex_data = hex::decode("1A014905D1").expect("cannot decode hex");
        let mut decoder = Decoder::new(&hex_data);
        let nonce = decode_nonce(&mut decoder).expect("cannot decode nonce");
        assert_eq!(nonce, 21_562_833);
    }
}
