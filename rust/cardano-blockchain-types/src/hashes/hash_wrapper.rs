//! A macro for defining a new type wrappers for the given hash types.

/// Defines a new type wrapper for the given hash types.
///
/// # Examples
///
/// ```
/// use cardano_blockchain_types::define_hashes;
/// use cardano_blockchain_types::hashes::Blake2b128Hash;
///
/// define_hashes!(
///     /// You can document the declared types...
///     (SomeHash, Blake2b128Hash),
///     // ...or not.
///     (AnotherHash, Blake2b128Hash),
/// );
///
/// let hash = SomeHash::new(&[1, 2, 3]);
/// println!("{hash:?}");
/// ```
#[macro_export]
macro_rules! define_hashes {
    ($($(#[$docs:meta])* ($name:ident, $inner:ty)),+ $(,)?) => {
        $(
            $(#[$docs])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name($inner);

            impl $name {
                /// Creates a new instance from the given bytes by hashing them.
                #[must_use]
                pub fn new(input_bytes: &[u8]) -> Self {
                    Self(<$inner>::new(input_bytes))
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(&format!("0x{}", self.0))
                }
            }

            impl From<$name> for Vec<u8> {
                fn from(value: $name) -> Self {
                    value.0.into()
                }
            }

            impl From<$inner> for $name {
                fn from(value: $inner) -> Self {
                    Self(value)
                }
            }

            impl TryFrom<&[u8]> for $name {
                type Error = $crate::hashes::Blake2bHashError;

                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    Ok(Self(<$inner>::try_from(value)?))
                }
            }

            impl TryFrom<Vec<u8>> for $name {
                type Error = $crate::hashes::Blake2bHashError;

                fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                    value.as_slice().try_into()
                }
            }

            impl std::str::FromStr for $name {
                type Err = $crate::hashes::Blake2bHashError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let hash: $inner = s.parse().map_err($crate::hashes::Blake2bHashError::from)?;
                    Ok(Self(hash))
                }
            }

            impl<C> minicbor::Encode<C> for $name {
                fn encode<W: minicbor::encode::Write>(
                    &self, e: &mut minicbor::Encoder<W>, ctx: &mut C,
                ) -> Result<(), minicbor::encode::Error<W::Error>> {
                    self.0.encode(e, ctx)
                }
            }

            impl<'a, C> minicbor::Decode<'a, C> for $name {
                fn decode(
                    d: &mut minicbor::Decoder<'a>, ctx: &mut C,
                ) -> Result<Self, minicbor::decode::Error> {
                    let hash = <$inner>::decode(d, ctx)?;
                    Ok(Self(hash))
                }
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use crate::hashes::Blake2b128Hash;

    // Define one type without a trailing comma.
    define_hashes!((H1, Blake2b128Hash));
    // Define one type with a trailing comma and a doc-comment.
    define_hashes!(
        /// Some documentation.
        (H2, Blake2b128Hash),
    );
    // Define multiple types at once.
    define_hashes!(
        /// Documentation.
        (H3, Blake2b128Hash),
        // No documentation.
        (H4, Blake2b128Hash),
        /// More documentation.
        (H5, Blake2b128Hash),
    );

    // There is little reason to check the conversion itself, it is mostly a demonstration
    // that the methods defined by the macro are working.
    #[test]
    fn hash_wrapper() {
        let hash = H1::new(&[1, 2, 3, 4, 5]);

        let v = Vec::from(hash);
        let from_slice = H1::try_from(v.as_slice()).unwrap();
        assert_eq!(hash, from_slice);

        let from_vec = H1::try_from(v).unwrap();
        assert_eq!(hash, from_vec);
    }

    // The display implementation is used to get user-friendly representation and must be
    // equal to `hex::encode(<underlying bytes>)`.
    #[test]
    fn display() {
        let hash = H1::new(&[1, 2, 3, 4, 5]);
        let display = format!("{hash}");
        let expected = "0x2a6ad53c3c6986406e1d6c7cfd06b69a";
        assert_eq!(expected, display);
    }
}
