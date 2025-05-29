use std::collections::BTreeMap;

use super::EncodeError;

/// A map of CBOR encoded key-value pairs with **bytewise** lexicographic key ordering.
#[derive(Debug, Default)]
pub struct CborMap(BTreeMap<Vec<u8>, Vec<u8>>);

impl CborMap {
    /// Creates an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A number of entries in a map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is there no entries in the map.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Encodes a key-value pair to CBOR and then inserts it into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// CBOR-encoded value is returned.
    pub fn encode_and_insert<C, K: minicbor::Encode<C>, V: minicbor::Encode<C>>(
        &mut self, ctx: &mut C, key: K, v: V,
    ) -> Result<Option<Vec<u8>>, EncodeError> {
        let (encoded_key, encoded_v) = (
            minicbor::to_vec_with(key, ctx)?,
            minicbor::to_vec_with(v, ctx)?,
        );
        Ok(self.0.insert(encoded_key, encoded_v))
    }

    /// Iterate over CBOR-encoded key-value pairs.
    /// Items are returned in **bytewise** lexicographic key ordering.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.0
            .iter()
            .map(|(key_vec, value_vec)| (key_vec.as_slice(), value_vec.as_slice()))
    }
}
