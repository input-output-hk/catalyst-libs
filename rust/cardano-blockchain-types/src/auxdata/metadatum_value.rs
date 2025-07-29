//! Metadatum value

use std::sync::Arc;

use minicbor::Decode;

/// Metadatum CBOR Encoded value
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L511>
#[derive(Clone, Debug)]
pub struct MetadatumValue(Arc<Vec<u8>>);

impl Decode<'_, ()> for MetadatumValue {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // Get the start of the raw CBOR value we are going to extract.
        let value_start = d.position();
        if let Err(error) = d.skip() {
            return Err(minicbor::decode::Error::message(format!(
                "Error decoding Metadatum value: {error}"
            )));
        }
        // Get the end of the raw value
        let value_end = d.position();
        let Some(value_slice) = d.input().get(value_start..value_end) else {
            return Err(minicbor::decode::Error::message(
                "Error decoding Metadatum value: Unable to extract raw value slice.",
            ));
        };

        // Intentionally copy the data into a vec, so that we don't have any self-reference
        // issues.
        Ok(Self(Arc::new(value_slice.to_vec())))
    }
}

impl AsRef<[u8]> for MetadatumValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
