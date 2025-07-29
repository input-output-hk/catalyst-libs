//! Transaction Witness
use std::fmt::{Display, Formatter};

use catalyst_types::{conversion::vkey_from_bytes, hashes::Blake2b224Hash};
use dashmap::{DashMap, DashSet};
use ed25519_dalek::VerifyingKey;
use pallas::ledger::traverse::MultiEraTx;

use crate::TxnIndex;

/// Hash of a witness verifying public key
pub type VKeyHash = Blake2b224Hash;

/// `WitnessMap` type of `DashMap` with
/// key as [u8; 28] = (`blake2b_244` hash of the public key)
/// value as `(Bytes, Vec<u8>) = (public key, tx index within the block)`
type WitnessMap = DashMap<VKeyHash, (VerifyingKey, DashSet<TxnIndex>)>;

#[derive(Debug)]
/// `TxnWitness` struct to store the witness data.
pub struct TxnWitness(WitnessMap);

impl TxnWitness {
    /// Create a new `TxnWitness` from a list of `MultiEraTx`.
    ///
    /// # Errors
    ///
    /// If the witness map does not contain a valid ED25519 public key,
    /// or unsupported transaction era.
    pub fn new(txs: &[MultiEraTx]) -> anyhow::Result<Self> {
        /// Update the temporary map with the witnesses.
        fn update_map(
            map: &WitnessMap,
            vkey_witness_set: Option<&Vec<pallas::ledger::primitives::conway::VKeyWitness>>,
            i: usize,
        ) -> anyhow::Result<()> {
            if let Some(vkey_witness_set) = vkey_witness_set {
                for vkey_witness in vkey_witness_set {
                    let vkey = vkey_from_bytes(&vkey_witness.vkey)?;
                    let vkey_hash = VKeyHash::new(vkey.as_ref());
                    let tx_num = i.into();
                    if let Some(entry) = map.get(&vkey_hash) {
                        entry.1.insert(tx_num);
                    } else {
                        let new_set = DashSet::new();
                        new_set.insert(tx_num);
                        map.insert(vkey_hash, (vkey, new_set));
                    }
                }
            }
            Ok(())
        }

        let map: WitnessMap = DashMap::new();
        for (i, tx) in txs.iter().enumerate() {
            match tx {
                MultiEraTx::AlonzoCompatible(tx, _) => {
                    let witness_set = &tx.transaction_witness_set;
                    update_map(&map, witness_set.vkeywitness.as_ref(), i)?;
                },
                MultiEraTx::Babbage(tx) => {
                    let witness_set = &tx.transaction_witness_set;
                    update_map(&map, witness_set.vkeywitness.as_ref(), i)?;
                },
                MultiEraTx::Conway(tx) => {
                    let witness_set = &tx.transaction_witness_set;
                    if let Some(non_empty_set) = witness_set.vkeywitness.clone() {
                        update_map(&map, Some(non_empty_set.to_vec()).as_ref(), i)?;
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!("Unsupported transaction Era"));
                },
            }
        }
        Ok(Self(map))
    }

    /// Check whether the public key hash is in the given transaction number.
    #[must_use]
    pub fn check_witness_in_tx(&self, vkey_hash: &VKeyHash, tx_num: TxnIndex) -> bool {
        self.0
            .get(vkey_hash)
            .is_some_and(|entry| entry.1.contains(&tx_num))
    }

    /// Get the actual verifying key from the given public key hash.
    #[must_use]
    pub fn get_witness_vkey(&self, vkey_hash: &VKeyHash) -> Option<VerifyingKey> {
        self.0.get(vkey_hash).map(|entry| entry.0)
    }
}

impl Display for TxnWitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for data in &self.0 {
            let vkey_hash = data.key();
            let txn = &data.value().1;
            let vkey = hex::encode(data.value().0.as_bytes());
            writeln!(
                f,
                "Key Hash: 0x{vkey_hash}, PublicKey: 0x{vkey}, Tx: {txn:?}"
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;
    use crate::multi_era_block_data::tests::{alonzo_block, babbage_block};

    #[test]
    fn tx_witness() {
        let alonzo = alonzo_block();
        let alonzo_block = pallas::ledger::traverse::MultiEraBlock::decode(&alonzo)
            .expect("Failed to decode MultiEraBlock");
        let txs_alonzo = alonzo_block.txs();
        let tx_witness_alonzo = TxnWitness::new(&txs_alonzo).expect("Failed to create TxnWitness");
        let vkey1_hash =
            VKeyHash::from_str("6082eb618d161a704207a0b3a9609e820111570d94d1e711b005386c")
                .expect("Failed to decode vkey1_hash");
        println!("{tx_witness_alonzo}");
        assert!(tx_witness_alonzo.get_witness_vkey(&vkey1_hash).is_some());
        assert!(tx_witness_alonzo.check_witness_in_tx(&vkey1_hash, 0.into()));

        let babbage = babbage_block();
        let babbage_block = pallas::ledger::traverse::MultiEraBlock::decode(&babbage)
            .expect("Failed to decode MultiEraBlock");
        let txs_babbage = babbage_block.txs();
        let tx_witness_babbage =
            TxnWitness::new(&txs_babbage).expect("Failed to create TxnWitness");
        let vkey2_hash =
            VKeyHash::from_str("ba4ab50bdecca85162f3b8114739bc5ba3aaa6490e2b1d15ad0f9c66")
                .expect("Failed to decode vkey2_hash");

        println!("{tx_witness_babbage}");
        assert!(tx_witness_babbage.get_witness_vkey(&vkey2_hash).is_some());
        assert!(tx_witness_babbage.check_witness_in_tx(&vkey2_hash, 0.into()));
    }
}
