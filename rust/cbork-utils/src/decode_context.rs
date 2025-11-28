//! CBOR decode context which could used as an argument for the `minicbor::Decode` logic

/// a type alias for the deterministic error handler function
pub type DeterministicErrorHandler =
    Box<dyn FnMut(minicbor::decode::Error) -> Result<(), minicbor::decode::Error>>;

/// CBOR `minicbor::Decode` context struct.
pub enum DecodeCtx {
    /// Decode a CBOR object applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    Deterministic,
    /// Decode a CBOR object applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    /// Additional apply `RFC 8949 Section 4.2.3` for array entries, so it become
    /// deterministically sorted.
    ArrayDeterministic,
    /// Decode a CBOR object **NOT** applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    ///
    /// Optionally it could carry an deterministic decoding error handler, so if provided
    /// deterministic decoding rule is applied and the error message passed to the
    /// handler function
    NonDeterministic(Option<DeterministicErrorHandler>),
}

impl DecodeCtx {
    /// Returns `DecodeCtx::NonDeterministic` variant
    /// Decode a CBOR object **NOT** applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    #[must_use]
    pub fn non_deterministic() -> Self {
        Self::NonDeterministic(None)
    }

    /// Returns `DecodeCtx::NonDeterministic` variant
    /// Decode a CBOR object **NOT** applying deterministic decoding rules (RFC 8949
    /// Section 4.2).
    ///
    /// When deterministic decoding rule is applied, the error message passed to
    /// the provided `handler`
    #[must_use]
    pub fn non_deterministic_with_handler(
        handler: impl FnMut(minicbor::decode::Error) -> Result<(), minicbor::decode::Error> + 'static
    ) -> Self {
        Self::NonDeterministic(Some(Box::new(handler)))
    }

    /// Depends on the set `DecodeCtx` variant applies the provided deterministic
    /// validation
    pub(crate) fn try_check(
        &mut self,
        f: impl FnOnce() -> Result<(), minicbor::decode::Error>,
    ) -> Result<(), minicbor::decode::Error> {
        match self {
            Self::Deterministic | Self::ArrayDeterministic => f(),
            Self::NonDeterministic(None) => Ok(()),
            Self::NonDeterministic(Some(h)) => {
                if let Err(err) = f() {
                    h(err)
                } else {
                    Ok(())
                }
            },
        }
    }
}
