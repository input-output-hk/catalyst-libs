//! Document Payload Chain.

use std::hash::Hash;

/// Document type - `Chain`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Chain;
