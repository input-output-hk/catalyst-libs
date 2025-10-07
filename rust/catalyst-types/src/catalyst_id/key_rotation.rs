//! COSE Signature Protected Header `kid` Role0 Key Version.

use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

use displaydoc::Display;
use thiserror::Error;

/// Errors from parsing the `KeyRotation`
#[derive(Display, Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum KeyRotationError {
    /// Invalid Role Key Rotation
    InvalidRole(#[from] ParseIntError),
}

/// Rotation count of the Role Key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyRotation(u16);

impl KeyRotation {
    /// Default Key Rotation
    pub const DEFAULT: KeyRotation = KeyRotation(0);

    /// Is the `KeyRotation` the default value
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::DEFAULT
    }

    /// Get the key by the rotation value from the provided keys slice, if present.
    pub fn get_key<'a, T>(
        &self,
        keys: &'a [T],
    ) -> Option<&'a T> {
        keys.get(self.0 as usize)
    }

    /// Get the latest rotation value calculated from the provided keys slice
    /// (`keys.len() - 1`).
    ///
    /// An empty slice will be saturated to `0` rotation value.
    pub fn from_latest_rotation<T>(keys: &[T]) -> Self {
        // we allow the cast here as per spec we cannot overflow `u16` with a keys count.
        #[allow(clippy::cast_possible_truncation)]
        let rotation = keys.len() as u16;
        let rotation = rotation.saturating_sub(1);

        Self(rotation)
    }
}

impl Default for KeyRotation {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<u16> for KeyRotation {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl FromStr for KeyRotation {
    type Err = KeyRotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u16>()?))
    }
}

impl Display for KeyRotation {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
