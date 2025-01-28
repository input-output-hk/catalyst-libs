use catalyst_types::{
    define_hashes,
    hashes::{Blake2b224Hash, Blake2b256Hash},
};

define_hashes!(
    /// A transaction hash - Blake2b-256 hash of a transaction.
    (TransactionHash, Blake2b256Hash),
    /// A public key hash - raw Blake2b-224 hash of an Ed25519 public key (has no discriminator, just the hash).
    (PubKeyHash, Blake2b224Hash)
);
