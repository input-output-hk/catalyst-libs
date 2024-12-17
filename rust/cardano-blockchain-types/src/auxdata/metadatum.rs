//! Raw metadata

use std::sync::Arc;

use dashmap::DashMap;
use minicbor::Decode;

use super::{metadatum_label::MetadatumLabel, metadatum_value::MetadatumValue};
use crate::conversion::from_saturating;

/// Transaction Metadata
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L519>
#[derive(Clone, Debug)]
pub struct Metadata(Arc<MetadataInner>);

#[derive(Clone, Debug)]
/// Transaction Metadata - Inner
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L519>
pub struct MetadataInner {
    /// Sequence the metadatum labels appear in the metadata.
    seq: Vec<MetadatumLabel>,
    /// K/V of metadata items.
    map: dashmap::ReadOnlyView<MetadatumLabel, MetadatumValue>,
}

impl Default for MetadataInner {
    fn default() -> Self {
        Self {
            seq: Vec::new(),
            map: DashMap::default().into_read_only(),
        }
    }
}

impl Metadata {
    /// Does the metadata contain the label?
    #[must_use]
    pub fn contains(&self, label: MetadatumLabel) -> bool {
        self.0.map.contains_key(&label)
    }

    /// Get the requested labels value
    #[must_use]
    pub fn get(&self, label: MetadatumLabel) -> Option<&MetadatumValue> {
        self.0.map.get(&label)
    }

    /// Are there any entries
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.seq.len() == 0
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata(Arc::new(MetadataInner::default()))
    }
}

impl Decode<'_, ()> for Metadata {
    fn decode(
        d: &mut minicbor::Decoder<'_>, ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let (entries, mut sequence, metadata) = match d.map() {
            Ok(Some(entries)) => {
                (
                    entries,
                    Vec::with_capacity(from_saturating(entries)),
                    DashMap::with_capacity(from_saturating(entries)),
                )
            },
            Ok(None) => {
                // Sadly... Indefinite Maps are allowed in Cardano CBOR Encoding.
                (u64::MAX, Vec::new(), DashMap::new())
            },
            Err(error) => {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding Metadata map: {error}"
                )));
            },
        };

        for _ in 0..entries {
            let label = MetadatumLabel::decode(d, ctx)?;
            let value = MetadatumValue::decode(d, ctx)?;

            sequence.push(label);
            let _unused = metadata.insert(label, value);

            // Look for End Sentinel IF its an indefinite MAP
            //   (which we know because entries is u64::MAX).
            if entries == u64::MAX {
                match d.datatype() {
                    Ok(minicbor::data::Type::Break) => {
                        // Skip over the break token.
                        let _unused = d.skip();
                        break;
                    },
                    Ok(_) => (), // Not break, so do next loop, should be the next key.
                    Err(error) => {
                        return Err(minicbor::decode::Error::message(format!(
                            "Error decoding indefinite Metadata map end sentinel: {error}"
                        )));
                    },
                }
            }
        }

        // Reduce metadata map and seq to smallest size.
        sequence.shrink_to_fit();
        metadata.shrink_to_fit();

        // Make map immutable
        let metadata = metadata.into_read_only();

        Ok(Self(Arc::new(MetadataInner {
            seq: sequence,
            map: metadata,
        })))
    }
}
