//! ZK Unit Vector objects decoding implementation

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use super::{ResponseRandomness, UnitVectorProof};
use crate::crypto::{
    elgamal::Ciphertext,
    group::{GroupElement, Scalar},
    zk_unit_vector::randomness_announcements::Announcement,
};

/// A CBOR version of the `UnitVectorProof`.
const UNIT_VECTOR_PROOF_VERSION: u64 = 0;

/// A number of elements in the
/// `zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item` data type.
///
/// The CBOR CDDL schema:
/// ```cddl
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item = ( zkproof-elgamal-announcement, ~elgamal-ristretto255-encrypted-choice, zkproof-ed25519-r-response )
///
/// zkproof-elgamal-announcement = ( zkproof-elgamal-group-element, zkproof-elgamal-group-element, zkproof-elgamal-group-element )
///
/// zkproof-elgamal-group-element = bytes .size 32
///
/// elgamal-ristretto255-encrypted-choice = [
///     c1: elgamal-ristretto255-group-element
///     c2: elgamal-ristretto255-group-element
/// ]
///
/// zkproof-ed25519-r-response = ( zkproof-ed25519-scalar, zkproof-ed25519-scalar, zkproof-ed25519-scalar )
///
/// zkproof-ed25519-scalar = bytes .size 32
/// ```
///
/// Therefore, the total number consists of the following:
/// - 8 (zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item)
///      - 3 (zkproof-elgamal-announcement = x3 zkproof-elgamal-group-element)
///      - 2 (elgamal-ristretto255-encrypted-choice = x2
///        elgamal-ristretto255-group-element)
///      - 3 (zkproof-ed25519-r-response = x3 zkproof-ed25519-scalar)
const ITEM_ELEMENTS_LEN: u64 = 8;

/// A minimal length of the underlying CBOR array of the `UnitVectorProof` data type.
///
/// The CBOR CDDL schema:
/// ```cddl
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection = [ +zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item, zkproof-ed25519-scalar ]
/// ```
const MIN_PROOF_CBOR_ARRAY_LEN: u64 = ITEM_ELEMENTS_LEN + 1;

impl Encode<()> for Announcement {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.i.encode(e, ctx)?;
        self.b.encode(e, ctx)?;
        self.a.encode(e, ctx)
    }
}

impl Decode<'_, ()> for Announcement {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let i = GroupElement::decode(d, ctx)?;
        let b = GroupElement::decode(d, ctx)?;
        let a = GroupElement::decode(d, ctx)?;
        Ok(Self { i, b, a })
    }
}

impl Encode<()> for ResponseRandomness {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.z.encode(e, ctx)?;
        self.w.encode(e, ctx)?;
        self.v.encode(e, ctx)
    }
}

impl Decode<'_, ()> for ResponseRandomness {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let z = Scalar::decode(d, ctx)?;
        let w = Scalar::decode(d, ctx)?;
        let v = Scalar::decode(d, ctx)?;
        Ok(Self { z, w, v })
    }
}

impl Encode<()> for UnitVectorProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.0.len() != self.1.len() || self.0.len() != self.2.len() {
            return Err(minicbor::encode::Error::message(format!(
                "All UnitVectorProof parts must have the same length: announcements = {}, choice = {}, responses = {}",
                self.0.len(),
                self.1.len(),
                self.2.len()
            )));
        }

        e.array(2)?;
        UNIT_VECTOR_PROOF_VERSION.encode(e, ctx)?;

        e.array(self.0.len() as u64 * ITEM_ELEMENTS_LEN + 1)?;
        for ((announcement, choice), response) in
            self.0.iter().zip(self.1.iter()).zip(self.2.iter())
        {
            announcement.encode(e, ctx)?;
            choice.first().encode(e, ctx)?;
            choice.second().encode(e, ctx)?;
            response.encode(e, ctx)?;
        }
        self.3.encode(e, ctx)
    }
}

impl Decode<'_, ()> for UnitVectorProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "UnitVectorProof")?;
        if len != 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected UnitVectorProof array length {len}, expected 2"
            )));
        }

        let version = u64::decode(d, ctx)?;
        if version != UNIT_VECTOR_PROOF_VERSION {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected UnitVectorProof version value: {version}, expected {UNIT_VECTOR_PROOF_VERSION}"
            )));
        }

        let len = decode_array_len(d, "UnitVectorProof inner array")?;
        if len < MIN_PROOF_CBOR_ARRAY_LEN
            || !len.saturating_sub(1).is_multiple_of(ITEM_ELEMENTS_LEN)
        {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected rUnitVectorProof inner array length {len}, expected multiplier of {MIN_PROOF_CBOR_ARRAY_LEN}"
            )));
        }

        let elements = len.saturating_sub(1) / ITEM_ELEMENTS_LEN;
        let mut announcements = Vec::with_capacity(elements as usize);
        let mut choices = Vec::with_capacity(elements as usize);
        let mut responses = Vec::with_capacity(elements as usize);

        for _ in 0..elements {
            announcements.push(Announcement::decode(d, ctx)?);
            let first = GroupElement::decode(d, ctx)?;
            let second = GroupElement::decode(d, ctx)?;
            choices.push(Ciphertext::from_elements(first, second));
            responses.push(ResponseRandomness::decode(d, ctx)?);
        }
        let scalar = Scalar::decode(d, ctx)?;

        Ok(Self(announcements, choices, responses, scalar))
    }
}

#[cfg(test)]
#[allow(clippy::explicit_deref_methods)]
mod tests {
    use proptest::property_test;

    use super::*;

    #[property_test]
    fn response_randomness_cbor_roundtrip(original: ResponseRandomness) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ResponseRandomness::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[property_test]
    fn announcement_cbor_roundtrip(original: Announcement) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = Announcement::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[property_test]
    fn unit_vector_proof_cbor_roundtrip(original: UnitVectorProof) {
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = UnitVectorProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
