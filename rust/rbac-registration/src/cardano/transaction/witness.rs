//! Transaction Witness
use std::fmt::{Display, Formatter};

use anyhow::bail;
use dashmap::DashMap;
use pallas::{codec::utils::Bytes, ledger::traverse::MultiEraTx};

use crate::utils::hashing::blake2b_244;

/// `WitnessMap` type of `DashMap` with
/// key as [u8; 28] = (`blake2b_244` hash of the public key)
/// value as `(Bytes, Vec<u8>) = (public key, tx index within the block)`
pub(crate) type WitnessMap = DashMap<[u8; 28], (Bytes, Vec<u16>)>;

#[derive(Debug)]
/// `TxWitness` struct to store the witness data.
pub(crate) struct TxWitness(WitnessMap);

impl TxWitness {
    /// Create a new `TxWitness` from a list of `MultiEraTx`.
    pub(crate) fn new(txs: &[MultiEraTx]) -> anyhow::Result<Self> {
        let map: WitnessMap = DashMap::new();
        for (i, tx) in txs.iter().enumerate() {
            match tx {
                MultiEraTx::AlonzoCompatible(tx, _) => {
                    let witness_set = &tx.transaction_witness_set;
                    if let Some(vkey_witness_set) = witness_set.vkeywitness.clone() {
                        for vkey_witness in vkey_witness_set {
                            let vkey_hash = blake2b_244(&vkey_witness.vkey)?;
                            let tx_num = u16::try_from(i)?;
                            map.entry(vkey_hash)
                                .and_modify(|entry: &mut (_, Vec<u16>)| entry.1.push(tx_num))
                                .or_insert((vkey_witness.vkey.clone(), vec![tx_num]));
                        }
                    };
                },
                MultiEraTx::Babbage(tx) => {
                    let witness_set = &tx.transaction_witness_set;
                    if let Some(vkey_witness_set) = witness_set.vkeywitness.clone() {
                        for vkey_witness in vkey_witness_set {
                            let vkey_hash = blake2b_244(&vkey_witness.vkey)?;
                            let tx_num = u16::try_from(i)?;
                            map.entry(vkey_hash)
                                .and_modify(|entry: &mut (_, Vec<u16>)| entry.1.push(tx_num))
                                .or_insert((vkey_witness.vkey.clone(), vec![tx_num]));
                        }
                    }
                },
                MultiEraTx::Conway(tx) => {
                    let witness_set = &tx.transaction_witness_set;
                    if let Some(vkey_witness_set) = &witness_set.vkeywitness.clone() {
                        for vkey_witness in vkey_witness_set {
                            let vkey_hash = blake2b_244(&vkey_witness.vkey)?;
                            let tx_num = u16::try_from(i)?;
                            map.entry(vkey_hash)
                                .and_modify(|entry: &mut (_, Vec<u16>)| entry.1.push(tx_num))
                                .or_insert((vkey_witness.vkey.clone(), vec![tx_num]));
                        }
                    }
                },
                _ => {
                    bail!("Unsupported transaction type");
                },
            };
        }
        Ok(Self(map))
    }

    /// Check whether the public key hash is in the given transaction number.
    pub(crate) fn check_witness_in_tx(&self, vkey_hash: &[u8; 28], tx_num: u16) -> bool {
        self.0
            .get(vkey_hash)
            .map_or(false, |entry| entry.1.contains(&tx_num))
    }

    /// Get the actual address from the given public key hash.
    #[allow(dead_code)]
    pub(crate) fn get_witness_pk_addr(&self, vkey_hash: &[u8; 28]) -> Option<Bytes> {
        self.0.get(vkey_hash).map(|entry| entry.0.clone())
    }
}

impl Display for TxWitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for data in &self.0 {
            let vkey_hash = hex::encode(data.key());
            let vkey: Vec<u8> = data.0.clone().into();
            let vkey_encoded = hex::encode(&vkey);
            writeln!(
                f,
                "Key Hash: {}, PublicKey: {}, Tx: {:?}",
                vkey_hash, vkey_encoded, data.1
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    fn conway() -> Vec<u8> {
        hex::decode(include_str!("../../test_data/cardano/conway_1.block"))
            .expect("Failed to decode hex block.")
    }

    #[test]
    fn tx_witness() {
        let conway = conway();
        let conway_block = pallas::ledger::traverse::MultiEraBlock::decode(&conway)
            .expect("Failed to decode MultiEraBlock");
        let txs_conway = conway_block.txs();
        let tx_witness_conway = TxWitness::new(&txs_conway).expect("Failed to create TxWitness");
        let vkey1_hash: [u8; 28] =
            hex::decode("bd95d582888acda57a20256bb03e4c4abb6bdf09a47d788605412c53")
                .expect("Failed to decode vkey1_hash")
                .try_into()
                .expect("Invalid length of vkey1_hash");
        assert!(tx_witness_conway.get_witness_pk_addr(&vkey1_hash).is_some());
        assert!(tx_witness_conway.check_witness_in_tx(&vkey1_hash, 0));
    }
}
