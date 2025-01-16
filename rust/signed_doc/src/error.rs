//! Catalyst Signed Document errors.

/// Catalyst Signed Document error.
#[derive(thiserror::Error, Debug)]
#[error("Catalyst Signed Document Error: {0:?}")]
pub struct Error(pub(crate) List);

/// List of errors.
#[derive(Debug)]
pub(crate) struct List(pub(crate) Vec<anyhow::Error>);

impl From<Vec<anyhow::Error>> for List {
    fn from(e: Vec<anyhow::Error>) -> Self {
        Self(e)
    }
}

impl From<Vec<anyhow::Error>> for Error {
    fn from(e: Vec<anyhow::Error>) -> Self {
        Self(e.into())
    }
}

impl Error {
    /// List of errors.
    pub fn errors(&self) -> &Vec<anyhow::Error> {
        &self.0 .0
    }
}
