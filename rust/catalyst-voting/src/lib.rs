//! Voting primitives which are used among Catalyst ecosystem.

mod crypto;
pub mod txs;
pub mod vote_protocol;

pub use crypto::elgamal::{PublicKey, SecretKey};
