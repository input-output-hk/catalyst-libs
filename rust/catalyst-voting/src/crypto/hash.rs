//! Different hash implementations.

pub use curve25519_dalek::digest;
use digest::{
    consts::{U32, U64},
    typenum::Unsigned,
    FixedOutput, HashMarker, Output, OutputSizeUser, Update,
};

/// Blake2b-512 hasher instance.
#[derive(Clone, Debug)]
#[must_use]
pub struct Blake2b512Hasher(blake2b_simd::State);

impl Blake2b512Hasher {
    /// Create a new `Blake2b256Hasher`.
    pub fn new() -> Self {
        Self(
            blake2b_simd::Params::new()
                .hash_length(Self::output_size())
                .to_state(),
        )
    }
}

// Implementation of the `digest::Digest` trait for `Blake2b256Hasher`.

impl Default for Blake2b512Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Update for Blake2b512Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

impl OutputSizeUser for Blake2b512Hasher {
    type OutputSize = U64;

    fn output_size() -> usize {
        Self::OutputSize::USIZE
    }
}

impl FixedOutput for Blake2b512Hasher {
    fn finalize_into(self, out: &mut Output<Self>) {
        let hash = self.0.finalize();
        out.copy_from_slice(hash.as_bytes());
    }
}

impl HashMarker for Blake2b512Hasher {}

/// Blake2b-256 hasher instance.
#[derive(Clone, Debug)]
#[must_use]
pub struct Blake2b256Hasher(blake2b_simd::State);

impl Blake2b256Hasher {
    /// Create a new `Blake2b256Hasher`.
    pub fn new() -> Self {
        Self(
            blake2b_simd::Params::new()
                .hash_length(Self::output_size())
                .to_state(),
        )
    }
}

// Implementation of the `digest::Digest` trait for `Blake2b256Hasher`.

impl Default for Blake2b256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Update for Blake2b256Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }
}

impl OutputSizeUser for Blake2b256Hasher {
    type OutputSize = U32;

    fn output_size() -> usize {
        Self::OutputSize::USIZE
    }
}

impl FixedOutput for Blake2b256Hasher {
    fn finalize_into(self, out: &mut Output<Self>) {
        let hash = self.0.finalize();
        out.copy_from_slice(hash.as_bytes());
    }
}

impl HashMarker for Blake2b256Hasher {}
