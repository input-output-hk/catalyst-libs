//! Smart Contract types

use std::sync::Arc;

use anyhow::anyhow;
use dashmap::DashMap;

/// Raw, Script
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub struct Script(Arc<Vec<u8>>);

impl minicbor::Decode<'_, ScriptType> for Script {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut ScriptType,
    ) -> Result<Self, minicbor::decode::Error> {
        let script_type = *ctx;

        if script_type == ScriptType::Native {
            // Native Scripts are actually CBOR arrays, so capture their data as bytes for
            // later processing.
            // See: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L542-L560
            let value_start = d.position();
            if let Err(error) = d.skip() {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding native script value:  {error}"
                )));
            }
            let value_end = d.position();
            let Some(value_slice) = d.input().get(value_start..value_end) else {
                return Err(minicbor::decode::Error::message(
                    "Invalid metadata value found. Unable to extract native script slice.",
                ));
            };
            Ok(Self(Arc::new(value_slice.to_vec())))
        } else {
            // Plutus is encoded as a bytes string.  Extract the script contents.
            // See: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L450-L452
            let script = match d.bytes() {
                Ok(script) => script,
                Err(error) => {
                    return Err(minicbor::decode::Error::message(format!(
                        "Error decoding plutus script data: {error}"
                    )))
                },
            };
            Ok(Self(Arc::new(script.to_vec())))
        }
    }
}

/// Array of Scripts
#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct ScriptArray(Arc<Vec<Script>>);

impl minicbor::Decode<'_, ScriptType> for ScriptArray {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut ScriptType,
    ) -> Result<Self, minicbor::decode::Error> {
        let mut scripts: Vec<Script> = Vec::new();

        // Scripts are encoded as arrays of scripts
        // See: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L527-L530
        // And: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L524
        let entries = match d.array() {
            Ok(Some(entries)) => entries,
            Ok(None) => {
                return Err(minicbor::decode::Error::message(
                    "Indefinite Script Array found decoding Metadata. Invalid.",
                ));
            },
            Err(error) => {
                let msg = Arc::new(format!("Error decoding script array: {error}"));
                return Err(minicbor::decode::Error::message(&msg));
            },
        };

        for _entry in 0..entries {
            scripts.push(Script::decode(d, ctx)?);
        }

        Ok(Self(Arc::new(scripts)))
    }
}

/// What type of smart contract is this list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::Display, Hash)]
pub enum ScriptType {
    /// Native smart contracts
    Native,
    /// Plutus smart contracts (with version number 1-x)
    Plutus(u64),
}

impl TryFrom<u64> for ScriptType {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Err(anyhow!("Invalid script type: {value}")),
            1 => Ok(Self::Native),
            _ => Ok(Self::Plutus(value.saturating_sub(1))),
        }
    }
}

/// Scripts attached to a transaction
#[derive(Clone, Debug)]
#[allow(dead_code, clippy::module_name_repetitions)]
pub struct TransactionScripts(Arc<dashmap::ReadOnlyView<ScriptType, ScriptArray>>);

impl Default for TransactionScripts {
    fn default() -> Self {
        Self(Arc::new(DashMap::default().into_read_only()))
    }
}

/// A Mutable version of the `TransactionScripts` because we need to build it iteratively
/// and in different ways.
pub(crate) type MutableTransactionScriptsMap = DashMap<ScriptType, ScriptArray>;

impl From<MutableTransactionScriptsMap> for TransactionScripts {
    fn from(scripts: MutableTransactionScriptsMap) -> Self {
        Self(Arc::new(scripts.into_read_only()))
    }
}
