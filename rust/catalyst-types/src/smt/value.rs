//! Items related to a value stored in the Sparse Merkle Tree

use sparse_merkle_tree::H256;

use crate::smt::hasher::Hasher;

/// A value wrapper that implements `sparse_merkle_tree::traits::Value`
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

/// Each type that needs to be stored in SMT must be convertible to and from bytes
pub trait Value {
    /// Converts the object to bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Converts bytes to the object
    fn from_bytes(bytes: &[u8]) -> Self;
}
