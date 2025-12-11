use sparse_merkle_tree::H256;

use crate::hasher::Hasher;

#[derive(Default, Debug, Clone, PartialEq)]
pub(super) struct ValueWrapper(pub(super) Vec<u8>);

impl sparse_merkle_tree::traits::Value for ValueWrapper {
    fn to_h256(&self) -> H256 {
        Hasher::hash(&self.0)
    }

    fn zero() -> Self {
        ValueWrapper(Vec::new())
    }
}

pub trait Value {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}
