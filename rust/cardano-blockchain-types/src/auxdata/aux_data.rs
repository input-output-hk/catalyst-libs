//! Auxiliary Data Decoding

use minicbor::Decode;

use super::{
    metadatum::Metadata,
    metadatum_label::MetadatumLabel,
    metadatum_value::MetadatumValue,
    scripts::{MutableTransactionScriptsMap, ScriptArray, ScriptType, TransactionScripts},
};

/// Auxiliary Data (Metadata) for a single Transaction in a block
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TransactionAuxData {
    /// Metadata attached to a transaction
    metadata: Metadata,
    /// Scripts attached to a transaction
    #[allow(dead_code)]
    scripts: TransactionScripts,
}

impl Decode<'_, ()> for TransactionAuxData {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // Check what kind of aux data we have to deal with
        match d.datatype() {
            // Shelley: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L522
            Ok(minicbor::data::Type::Map) => {
                Ok(TransactionAuxData {
                    metadata: Metadata::decode(d, &mut ())?,
                    scripts: TransactionScripts::default(),
                })
            },
            // Shelley-MA: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L523
            Ok(minicbor::data::Type::Array) => Self::decode_shelley_ma_array(d),
            // Maybe Alonzo and beyond: https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L526
            Ok(minicbor::data::Type::Tag) => Self::decode_alonzo_plus_map(d),
            Ok(unexpected) => {
                let msg = format!(
                    "Error decoding Transaction Aux Data: Unexpected datatype {unexpected}"
                );
                Err(minicbor::decode::Error::message(&msg))
            },
            Err(error) => {
                let msg = format!("Error decoding Transaction Aux Data: {error}");
                Err(minicbor::decode::Error::message(msg))
            },
        }
    }
}

impl TransactionAuxData {
    /// Get metadata with the given label.
    #[must_use]
    pub fn metadata(&self, label: MetadatumLabel) -> Option<&MetadatumValue> {
        self.metadata.get(label)
    }

    /// Decode a Shelley-MA Auxiliary Data Array
    fn decode_shelley_ma_array(d: &mut minicbor::Decoder) -> Result<Self, minicbor::decode::Error> {
        match d.array() {
            Ok(Some(entries)) => {
                if entries != 2 {
                    let msg = format!(
                        "Error decoding Transaction Aux Data: Script Data Array Expected 2 entries, found {entries}."
                    );
                    return Err(minicbor::decode::Error::message(&msg));
                }
            },
            Ok(None) => {
                return Err(minicbor::decode::Error::message(
                    "Error decoding Transaction Aux Data: Indefinite Array found decoding Metadata. Invalid."));
            },
            Err(error) => {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding Transaction Aux Data: {error}."
                )));
            },
        };

        let metadata = Metadata::decode(d, &mut ())?;
        let script_array = ScriptArray::decode(d, &mut ScriptType::Native)?;

        let scripts = MutableTransactionScriptsMap::default();
        scripts.insert(ScriptType::Native, script_array);

        Ok(Self {
            metadata,
            scripts: scripts.into(),
        })
    }

    /// Decode an Alonzo Plus MAP
    fn decode_alonzo_plus_map(d: &mut minicbor::Decoder) -> Result<Self, minicbor::decode::Error> {
        match d.tag() {
            Ok(tag) => {
                if tag.as_u64() != 259 {
                    return Err(minicbor::decode::Error::message(format!(
                        "Invalid tag for Alonzo+ Aux Data. Expected 259, found {tag}."
                    )));
                }
            },
            Err(error) => {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding Transaction Alonzo+ Aux Data: {error}."
                )));
            },
        }

        let entries = match d.map() {
            Ok(Some(entries)) => entries,
            Ok(None) => {
                return Err(minicbor::decode::Error::message(
                    "Indefinite Map found decoding Alonzo+ Metadata. Invalid.",
                ))
            },
            Err(error) => {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding Transaction Alonzo+ Aux Data: {error}."
                )))
            },
        };

        // Make the default versions of the metadata and script types
        let mut metadata = Metadata::default();
        let scripts = MutableTransactionScriptsMap::default();

        // iterate the map
        for _ in 0..entries {
            let script_type = match d.u64() {
                Ok(key) => {
                    if let Ok(script_type) = ScriptType::try_from(key) {
                        script_type
                    } else {
                        // Only fails if its Metadata and not a script.
                        if metadata.is_empty() {
                            metadata = Metadata::decode(d, &mut ())?;
                            continue;
                        }
                        return Err(minicbor::decode::Error::message(
                            "Multiple Alonzo+ Metadata entries found. Invalid.",
                        ));
                    }
                },

                Err(error) => {
                    return Err(minicbor::decode::Error::message(format!(
                        "Error decoding Alonzo+ Metadata Aux Data Type Key: {error}"
                    )));
                },
            };

            let mut ctx = script_type;

            let script_array = ScriptArray::decode(d, &mut ctx)?;
            if scripts.insert(script_type, script_array).is_some() {
                return Err(minicbor::decode::Error::message(
                    "Multiple Alonzo+ Script entries of type {script_type} found. Invalid.",
                ));
            }
        }

        Ok(Self {
            metadata,
            scripts: scripts.into(),
        })
    }
}
