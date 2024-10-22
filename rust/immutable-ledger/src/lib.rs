//! Block Serialization
//!
//! Facilitates block serialization and validation for immutable ledger
//!
//! Spec: https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/immutable_ledger/ledger/
//!

/// Block validation logic
pub mod validate;

/// Block encoding decoding
pub mod serialize;
