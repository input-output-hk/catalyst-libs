//! Utilities for [CIP-134] (Cardano URIs - Address Representation).
//!
//! [CIP-134]: https://github.com/cardano-foundation/CIPs/tree/master/CIP-0134

pub use self::{uri::Cip0134Uri, uri_list::Cip0134UriList};

mod uri;
mod uri_list;
