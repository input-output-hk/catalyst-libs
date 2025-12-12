//! Error types for Sparse Merkle Tree Module

use thiserror::Error;

/// Error type
#[derive(Debug, Error)]
pub enum Error {
    /// Inner implementation error
    #[error(transparent)]
    InnerTree(#[from] sparse_merkle_tree::error::Error),
    /// Invalid key prefix length
    #[error("invalid key prefix length")]
    InvalidKeyPrefixLength,
}
