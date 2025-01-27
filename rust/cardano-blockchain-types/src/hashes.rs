//! A bunch of specific hashes.

// cspell: attrss

use std::str::FromStr;

use catalyst_types::hashes::{Blake2b224Hash, Blake2b256Hash, Blake2bHashError};
use pallas_crypto::hash::Hash;

macro_rules! define_hashes {
    ($($(#[$($attrss:tt)*])* ($name:ident, $inner:ty)),+) => {
        $(
            $(#[$($attrss)*])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name($inner);

            impl $name {
                /// Creates a new instance from the given bytes by hashing them.
                #[must_use]
                pub fn new(input_bytes: &[u8]) -> Self {
                    Self(<$inner>::new(input_bytes))
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
                type Error = Blake2bHashError;

                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    Ok(Self(<$inner>::try_from(value)?))
                }
            }

            impl TryFrom<Vec<u8>> for $name {
                type Error = Blake2bHashError;

                fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                    value.as_slice().try_into()
                }
            }

            impl FromStr for $name {
                type Err = Blake2bHashError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let hash: $inner = s.parse().map_err(Blake2bHashError::from)?;
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

            // TODO: Uncomment when the changes to Blake2bHash are merged.
            // impl SerializeValue for $name {
            //     fn serialize<'b>(
            //         &self, typ: &ColumnType, writer: CellWriter<'b>,
            //     ) -> Result<WrittenCellProof<'b>, SerializationError> {
            //         self.0.serialize(typ, writer)
            //     }
            // }
            //
            // impl<'frame, 'metadata> DeserializeValue<'frame, 'metadata> for $name
            // {
            //     fn type_check(typ: &ColumnType) -> Result<(), TypeCheckError> {
            //         <$inner>::type_check(typ)
            //     }
            //
            //     fn deserialize(
            //         typ: &'metadata ColumnType<'metadata>, v: Option<FrameSlice<'frame>>,
            //     ) -> Result<Self, DeserializationError> {
            //         let hash = <$inner>::deserialize(typ, v)?;
            //         Ok(Self(hash))
            //     }
            // }
        )+
    };
}

define_hashes!(
    /// A transaction hash - Blake2b-256 hash of a transaction.
    (TransactionHash, Blake2b256Hash),
    /// A public key hash - raw Blake2b-224 hash of an Ed25519 public key (has no discriminator, just the hash).
    (PubKeyHash, Blake2b224Hash)
);

impl From<Hash<32>> for TransactionHash {
    fn from(hash: Hash<32>) -> Self {
        Self(Blake2b256Hash::from(hash))
    }
}

impl From<Hash<28>> for PubKeyHash {
    fn from(hash: Hash<28>) -> Self {
        Self(Blake2b224Hash::from(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There is little reason to check the conversion itself, it is mostly a demonstration
    // that the methods defined by the macro are working.
    #[test]
    fn roundtrip() {
        let hash = TransactionHash::new(&[]);

        let v = Vec::from(hash);
        let from_slice = TransactionHash::try_from(v.as_slice()).unwrap();
        assert_eq!(hash, from_slice);

        let from_vec = TransactionHash::try_from(v).unwrap();
        assert_eq!(hash, from_vec);
    }
}
