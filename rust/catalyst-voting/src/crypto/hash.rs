//! Blake2b-256 hash implementation.

#![allow(dead_code)]

/// Blake2b-256 hasher instance.
pub(crate) struct Blake2b256Hasher(blake2b_simd::State);

impl Blake2b256Hasher {
    /// Create a new `Blake2b256Hasher`.
    pub(crate) fn new() -> Self {
        Self(
            blake2b_simd::Params::new()
                .hash_length(Blake2b256::HASH_SIZE)
                .to_state(),
        )
    }

    /// Incrementally add bytes to the hasher.
    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    ///  Finalize the state and return a `Hash`.
    pub(crate) fn finalize(self) -> Blake2b256 {
        let hash = self.0.finalize();
        Blake2b256::from_bytes_unchecked(hash.as_bytes())
    }
}

/// Blake2b-256 hash instance.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Blake2b256([u8; Self::HASH_SIZE]);

impl Blake2b256 {
    /// Blake2b-256 hash size.
    const HASH_SIZE: usize = 32;

    /// Create a new `Blake2b256` from bytes.
    /// It does not validate the size of the bytes, so all checks should be done by
    /// the caller.
    fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        let mut hash_bytes = [0; Self::HASH_SIZE];
        hash_bytes.copy_from_slice(bytes);

        Self(hash_bytes)
    }

    /// Calculate a new `Blake2b256` from bytes.
    pub(crate) fn hash(bytes: &[u8]) -> Self {
        let hash = blake2b_simd::Params::new()
            .hash_length(Self::HASH_SIZE)
            .hash(bytes);

        Self::from_bytes_unchecked(hash.as_bytes())
    }

    /// Return the hash bytes.
    pub(crate) fn to_bytes(&self) -> [u8; Self::HASH_SIZE] {
        self.0
    }
}
