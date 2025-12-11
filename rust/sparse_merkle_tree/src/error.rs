use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InnerTree(sparse_merkle_tree::error::Error),
}
