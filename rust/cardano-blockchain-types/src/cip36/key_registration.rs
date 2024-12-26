//! CIP-36 Key Registration 61284.
//!
//! Catalyst registration data
//!
//! <https://cips.cardano.org/cip/CIP-36>
//! <https://github.com/cardano-foundation/CIPs/blob/master/CIP-0036/schema.cddl>

use std::collections::HashSet;

use ed25519_dalek::VerifyingKey;
use minicbor::{decode, Decode, Decoder};
use pallas::ledger::addresses::{Address, ShelleyAddress};
use strum::FromRepr;

use crate::utils::decode_helper::{decode_array_len, decode_bytes, decode_helper, decode_map_len};

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
#[derive(Clone, Default)]
pub struct Cip36KeyRegistration {
    /// Is this CIP36 or CIP15 format.
    #[allow(clippy::struct_field_names)]
    pub is_cip36: Option<bool>,
    /// Voting public keys (called Delegations in the CIP-36 Spec).
    /// Field 1 in the CIP-36 61284 Spec.
    pub voting_pks: Vec<VotingPubKey>,
    /// Stake public key to associate with the voting keys.
    /// Field 2 in the CIP-36 61284 Spec.
    pub stake_pk: VerifyingKey,
    /// Payment Address to associate with the voting keys.
    /// Field 3 in the CIP-36 61284 Spec.
    pub payment_addr: Option<ShelleyAddress>,
    /// Nonce (nonce that has been slot corrected).
    /// Field 4 in the CIP-36 61284 Spec.
    // FIXME: set this nonce
    pub nonce: u64,
    /// Registration Purpose (Always 0 for Catalyst).
    /// Field 5 in the CIP-36 61284 Spec.
    pub purpose: u64,
    /// Raw nonce (nonce that has not had slot correction applied).
    pub raw_nonce: u64,
    /// Is payment address payable? (not a script)
    pub is_payable: bool,
}

/// Voting public key containing the public key and weight.
#[derive(Clone, Debug)]
pub struct VotingPubKey {
    /// Voting public key.
    pub voting_pk: VerifyingKey,
    /// Voting key associated weight.
    pub weight: u32,
}

/// Enum of CIP36 registration (61284) with its associated unsigned integer key.
#[derive(FromRepr, Debug, PartialEq)]
#[repr(u16)]
pub enum Cip36KeyRegistrationKeys {
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
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP36 Key Registration")?;

        let mut cip36_key_registration = Cip36KeyRegistration::default();

        // Record of founded keys. Check for duplicate keys in the map
        let mut found_keys: HashSet<u16> = HashSet::new();

        for _ in 0..map_len {
            let key: u16 = decode_helper(d, "key in CIP36 Key Registration", ctx)?;

            if let Some(key) = Cip36KeyRegistrationKeys::from_repr(key) {
                match key {
                    Cip36KeyRegistrationKeys::VotingKey => {
                        if !found_keys.insert(key as u16) {
                            return Err(decode::Error::message(
                                "Duplicate key in CIP36 Key Registration voting key",
                            ));
                        }
                        let (is_cip36, voting_keys) = decode_voting_key(d)?;
                        cip36_key_registration.is_cip36 = Some(is_cip36);
                        cip36_key_registration.voting_pks = voting_keys;
                    },
                    Cip36KeyRegistrationKeys::StakePk => {
                        if !found_keys.insert(key as u16) {
                            return Err(decode::Error::message(
                                "Duplicate key in CIP36 Key Registration stake public key",
                            ));
                        }
                        let stake_pk = decode_stake_pk(d)?;
                        cip36_key_registration.stake_pk = stake_pk;
                    },
                    Cip36KeyRegistrationKeys::PaymentAddr => {
                        if !found_keys.insert(key as u16) {
                            return Err(decode::Error::message(
                                "Duplicate key in CIP36 Key Registration payment address",
                            ));
                        }
                        let shelley_addr = decode_payment_addr(d)?;
                        cip36_key_registration.payment_addr = Some(shelley_addr.clone());
                        cip36_key_registration.is_payable = !shelley_addr.payment().is_script();
                    },
                    Cip36KeyRegistrationKeys::Nonce => {
                        if !found_keys.insert(key as u16) {
                            return Err(decode::Error::message(
                                "Duplicate key in CIP36 Key Registration nonce",
                            ));
                        }
                        let raw_nonce = decode_nonce(d)?;
                        cip36_key_registration.raw_nonce = raw_nonce;
                    },
                    Cip36KeyRegistrationKeys::Purpose => {
                        if !found_keys.insert(key as u16) {
                            return Err(decode::Error::message(
                                "Duplicate key in CIP36 Key Registration purpose",
                            ));
                        }
                        let purpose = decode_purpose(d)?;
                        cip36_key_registration.purpose = purpose;
                    },
                }
            }
        }
        Ok(cip36_key_registration)
    }
}

/// Helper function for decoding the voting key.
///
/// # Returns
///
/// A tuple containing a boolean value, true if it is CIP36 format, false if it is CIP15
/// format and a vector of voting public keys.
fn decode_voting_key(d: &mut Decoder) -> Result<(bool, Vec<VotingPubKey>), decode::Error> {
    let mut voting_keys = Vec::new();
    let mut is_cip36 = false;

    match d.datatype()? {
        // CIP15 type registration (single voting key).
        // ```cddl
        //      legacy_key_registration = $cip36_vote_pub_key
        //      $cip36_vote_pub_key /= bytes .size 32
        // ```
        minicbor::data::Type::Bytes => {
            let pub_key = decode_bytes(d, "CIP36 Key Registration voting key, single voting key")?;
            let vk = voting_pk_vec_to_verifying_key(&pub_key).map_err(|e| {
                decode::Error::message(format!(
                    "CIP36 Key Registration voting key, singe voting key, {e}"
                ))
            })?;
            // Since there is 1 voting key, all the weight goes to this key = 1.
            voting_keys.push(VotingPubKey {
                voting_pk: vk,
                weight: 1,
            });
        },
        // CIP36 type registration (multiple voting keys).
        // ```cddl
        //      [+delegation]
        //      delegation = [$cip36_vote_pub_key, $weight]
        //      $cip36_vote_pub_key /= bytes .size 32
        // ```
        minicbor::data::Type::Array => {
            is_cip36 = true;
            let len =
                decode_array_len(d, "CIP36 Key Registration voting key, multiple voting keys")?;
            for _ in 0..len {
                let len = decode_array_len(d, "CIP36 Key Registration voting key, delegations")?;
                // This fixed array should be a length of 2 (voting key, weight).
                if len != 2 {
                    return Err(decode::Error::message(format!(
                        "Invalid length for CIP36 Key Registration voting key delegations, expected 2, got {len}"
                    )));
                }
                // The first entry.
                let pub_key = decode_bytes(
                    d,
                    "CIP36 Key Registration voting key, delegation array first entry (voting public key)",
                )?;
                // The second entry.
                let weight: u32 = decode_helper(
                    d,
                    "CIP36 Key Registration voting key, delegation array second entry (weight)",
                    &mut (),
                )?;

                let vk = voting_pk_vec_to_verifying_key(&pub_key).map_err(|e| {
                    decode::Error::message(format!(
                        "CIP36 Key Registration voting key, multiple voting keys, {e}"
                    ))
                })?;

                voting_keys.push(VotingPubKey {
                    voting_pk: vk,
                    weight,
                });
            }
        },
        _ => {
            return Err(decode::Error::message(
                "Invalid datatype for CIP36 Key Registration voting key",
            ))
        },
    }
    Ok((is_cip36, voting_keys))
}

/// Helper function for converting `&[u8]` to `VerifyingKey`.
fn voting_pk_vec_to_verifying_key(pub_key: &[u8]) -> anyhow::Result<VerifyingKey> {
    VerifyingKey::from_bytes(
        pub_key
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid verifying key length"))?,
    )
    .map_err(|_| anyhow::anyhow!("Failed to convert to VerifyingKey"))
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
/// The stake public key as a `VerifyingKey`.
fn decode_stake_pk(d: &mut Decoder) -> Result<VerifyingKey, decode::Error> {
    let pub_key = decode_bytes(d, "CIP36 Key Registration stake public key")?;
    voting_pk_vec_to_verifying_key(&pub_key).map_err(|e| {
        decode::Error::message(format!("CIP36 Key Registration stake public key, {e}"))
    })
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
/// The payment address as a `ShelleyAddress`.
fn decode_payment_addr(d: &mut Decoder) -> Result<ShelleyAddress, decode::Error> {
    let raw_addr = decode_bytes(d, "CIP36 Key Registration payment address")?;
    let address = Address::from_bytes(&raw_addr).map_err(|e| {
        decode::Error::message(format!("CIP36 Key Registration payment address, {e}"))
    })?;
    if let Address::Shelley(addr) = address {
        Ok(addr.clone())
    } else {
        Err(decode::Error::message(format!(
            "Invalid CIP36 Key Registration payment address, expected Shelley address, got {address}"
        )))
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
/// Raw nonce.
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
/// The purpose.
fn decode_purpose(d: &mut Decoder) -> Result<u64, decode::Error> {
    decode_helper(d, "CIP36 Key Registration purpose", &mut ())
}
