//! CBOR decode context which could used as an argument for the `minicbor::Decode` logic

/// CBOR `minicbor::Decode` context struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeCtx {
    /// Decode a CBOR object applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    Deterministic,
    /// Decode a CBOR object **NOT** applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    NonDeterministic,
}

impl DecodeCtx {
    /// Depends on the set `DecodeCtx` variant applies the provided deterministic validation 
    pub(crate) fn try_check<E>(&self, f: impl FnOnce() -> Result<(), E>) -> Result<(), E> {
        match self {
            Self::Deterministic => f(),
            Self::NonDeterministic => Ok(()),
        }
    }
}
