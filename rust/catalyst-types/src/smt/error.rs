//! Error types for Sparse Merkle Tree Module

use thiserror::Error;

/// Error type
#[derive(Debug, Error)]
pub enum Error {
    /// Internal sparse-merkle-tree error.
    #[error(transparent)]
    InnerTree(sparse_merkle_tree::error::Error),
}
