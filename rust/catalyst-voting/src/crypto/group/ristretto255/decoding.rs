//! ristretto255 objects decoding implementation

use anyhow::anyhow;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar as IScalar};
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use super::{GroupElement, Scalar};

impl Scalar {
    /// `Scalar` bytes size
    pub const BYTES_SIZE: usize = 32;

    /// Attempt to construct a `Scalar` from a canonical byte representation.
    ///
    /// # Errors
    ///   - Cannot decode scalar.
    pub fn from_bytes(bytes: [u8; Self::BYTES_SIZE]) -> anyhow::Result<Scalar> {
        Into::<Option<_>>::into(IScalar::from_canonical_bytes(bytes))
            .map(Scalar)
            .ok_or(anyhow!("Cannot decode scalar."))
    }

    /// Convert this `Scalar` to its underlying sequence of bytes.
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.to_bytes()
    }
}

impl GroupElement {
    /// `Scalar` bytes size
    pub const BYTES_SIZE: usize = 32;

    /// Attempt to construct a `Scalar` from a compressed value byte representation.
    ///
    /// # Errors
    ///   - Cannot decode group element.
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        Ok(GroupElement(
            CompressedRistretto::from_slice(bytes)?
                .decompress()
                .ok_or(anyhow!("Cannot decode group element."))?,
        ))
    }

    /// Convert this `GroupElement` to its underlying sequence of bytes.
    /// Always encode the compressed value.
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        self.0.compress().to_bytes()
    }
}

impl Encode<()> for Scalar {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.to_bytes().encode(e, ctx)
    }
}

impl Decode<'_, ()> for Scalar {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let bytes = <[u8; Scalar::BYTES_SIZE]>::decode(d, ctx)?;
        Self::from_bytes(bytes).map_err(minicbor::decode::Error::message)
    }
}

impl Encode<()> for GroupElement {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.to_bytes().encode(e, ctx)
    }
}

impl Decode<'_, ()> for GroupElement {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let compressed = <[u8; GroupElement::BYTES_SIZE]>::decode(d, ctx)?;
        Self::from_bytes(&compressed).map_err(minicbor::decode::Error::message)
    }
}

impl serde::Serialize for GroupElement {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = hex::encode(self.to_bytes());
        serializer.serialize_str(&hex)
    }
}

impl<'de> serde::Deserialize<'de> for GroupElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let hex = String::deserialize(deserializer)?;
        let bytes = hex::decode(hex).map_err(serde::de::Error::custom)?;
        let array = <[u8; GroupElement::BYTES_SIZE]>::try_from(bytes.as_slice())
            .map_err(serde::de::Error::custom)?;
        Self::from_bytes(&array).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn scalar_to_bytes_from_bytes_test(e1: Scalar) {
        let bytes = e1.to_bytes();
        let e2 = Scalar::from_bytes(bytes).unwrap();
        assert_eq!(e1, e2);
    }

    #[proptest]
    fn group_element_to_bytes_from_bytes_test(ge1: GroupElement) {
        let bytes = ge1.to_bytes();
        let ge2 = GroupElement::from_bytes(&bytes).unwrap();
        assert_eq!(ge1, ge2);
    }

    #[proptest]
    fn scalar_cbor_roundtrip(original: Scalar) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Scalar::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[proptest]
    fn group_element_cbor_roundtrip(original: GroupElement) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = GroupElement::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[proptest]
    fn group_element_json_roundtrip(original: GroupElement) {
        let json = serde_json::to_string(&original).unwrap();
        let decoded = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }
}
