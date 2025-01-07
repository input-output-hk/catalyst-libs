//! Conversion functions

use displaydoc::Display;
use thiserror::Error;

/// Errors that can occur when converting bytes to an Ed25519 verifying key.
#[derive(Display, Debug, Error)]
pub enum VKeyFromBytesError {
    /// Invalid byte length: expected {expected} bytes, got {actual}
    InvalidLength {
        /// The expected number of bytes (must be 32).
        expected: usize,
        /// The actual number of bytes in the provided input.
        actual: usize,
    },
    /// Failed to parse Ed25519 public key: {source}
    ParseError {
        /// The underlying error from `ed25519_dalek`.
        #[from]
        source: ed25519_dalek::SignatureError,
    },
}

/// Convert an `<T>` to `<R>` (saturate if out of range).
/// Note can convert any int to float, or f32 to f64 as well.
/// can not convert from float to int, or f64 to f32.
pub fn from_saturating<
    R: Copy + num_traits::identities::Zero + num_traits::Bounded,
    T: Copy
        + TryInto<R>
        + std::ops::Sub<Output = T>
        + std::cmp::PartialOrd<T>
        + num_traits::identities::Zero,
>(
    value: T,
) -> R {
    match value.try_into() {
        Ok(value) => value,
        Err(_) => {
            // If we couldn't convert, its out of range for the destination type.
            if value > T::zero() {
                // If the number is positive, its out of range in the positive direction.
                R::max_value()
            } else {
                // Otherwise its out of range in the negative direction.
                R::min_value()
            }
        },
    }
}

/// Try and convert a byte array into an Ed25519 verifying key.
///
/// # Errors
///
/// Fails if the bytes are not a valid ED25519 Public Key
pub fn vkey_from_bytes(bytes: &[u8]) -> Result<ed25519_dalek::VerifyingKey, VKeyFromBytesError> {
    if bytes.len() != ed25519_dalek::PUBLIC_KEY_LENGTH {
        return Err(VKeyFromBytesError::InvalidLength {
            expected: ed25519_dalek::PUBLIC_KEY_LENGTH,
            actual: bytes.len(),
        });
    }

    let mut ed25519 = [0u8; ed25519_dalek::PUBLIC_KEY_LENGTH];
    ed25519.copy_from_slice(bytes); // Can't panic because we already validated its size.

    ed25519_dalek::VerifyingKey::from_bytes(&ed25519)
        .map_err(|source| VKeyFromBytesError::ParseError { source })
}
