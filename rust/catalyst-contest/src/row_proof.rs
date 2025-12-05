//! A universal encrypted row proof.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::ElgamalRistretto255Choice;

/// A length of the underlying CBOR array of the `ProofScalar` type.
const SCALAR_PROOF_LEN: u64 = 32;

/// A length of the underlying CBOR array of the `ProofAnnouncementElement` type.
const PROOF_ANNOUNCEMENT_ELEMENT_LEN: u64 = 32;

/// A minimal length (number of elements) of the
/// `zkproof-elgamal-ristretto255-unit-vector-with-single-selection` array.
///
///
/// The number of elements consists of the following:
/// - 7 (zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item)
///      - 3 (zkproof-elgamal-announcement = x3 zkproof-elgamal-group-element)
///      - 1 (elgamal-ristretto255-encrypted-choice)
///      - 3 (zkproof-ed25519-r-response = x3 zkproof-ed25519-scalar)
/// - 1 (zkproof-ed25519-scalar)
const MIN_SELECTION_LEN: u64 = 8;

/// A universal encrypted row proof.
///
/// The CDDL schema:
/// ```cddl
/// row-proof = [0, zkproof-elgamal-ristretto255-unit-vector-with-single-selection ]
///
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection = [ +zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item, zkproof-ed25519-scalar ]
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RowProof {
    /// A list of a single selection proofs.
    pub selections: Vec<SingleSelectionProof>,
    /// An individual Ed25519 scalar used in ZK proofs.
    pub scalar: ProofScalar,
}

/// A proof that the row is a unit vector with a single selection.
///
/// The CDDL schema:
/// ```cddl
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item = ( zkproof-elgamal-announcement, ~elgamal-ristretto255-encrypted-choice, zkproof-ed25519-r-response )
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SingleSelectionProof {
    /// A ZK proof announcement values for Elgamal.
    pub announcement: ProofAnnouncement,
    /// An elgamal encrypted ciphertext.
    pub choice: ElgamalRistretto255Choice,
    /// A ZK proof response values for Ed25519.
    pub response: ProofResponse,
}

/// An individual Ed25519 scalar used in ZK proofs.
///
/// The CDDL schema:
/// ```cddl
/// zkproof-ed25519-scalar = bytes .size 32
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProofScalar(pub [u8; SCALAR_PROOF_LEN as usize]);

/// A ZK proof announcement values for Elgamal.
///
/// The CDDL schema:
/// ```cddl
/// zkproof-elgamal-announcement = ( zkproof-elgamal-group-element, zkproof-elgamal-group-element, zkproof-elgamal-group-element )
///
/// zkproof-elgamal-group-element = bytes .size 32
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProofAnnouncement(
    pub ProofAnnouncementElement,
    pub ProofAnnouncementElement,
    pub ProofAnnouncementElement,
);

/// An individual Elgamal group element used in ZK proofs.
///
/// The CDDL schema:
/// ```cddl
/// zkproof-elgamal-group-element = bytes .size 32
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProofAnnouncementElement(pub [u8; PROOF_ANNOUNCEMENT_ELEMENT_LEN as usize]);

/// A ZK proof response values for Ed25519.
///
/// The CDDL schema:
///
/// ```cddl
/// zkproof-ed25519-r-response = ( zkproof-ed25519-scalar, zkproof-ed25519-scalar, zkproof-ed25519-scalar )
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProofResponse(pub ProofScalar, pub ProofScalar, pub ProofScalar);

impl Decode<'_, ()> for RowProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let len = decode_array_len(d, "row proof")?;
        if len != 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected row proof array length {len}, expected 2"
            )));
        }
        let version = u64::decode(d, ctx)?;
        if version != 0 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected row proof version value: {version}, expected 0"
            )));
        }

        let len = decode_array_len(d, "row proof single selection")?;
        if len < MIN_SELECTION_LEN || !len.is_multiple_of(MIN_SELECTION_LEN) {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected row proof single selection array length {len}, expected multiplier of {MIN_SELECTION_LEN}"
            )));
        }

        let mut selections = Vec::with_capacity(len as usize - 1);
        for _ in 0..len / MIN_SELECTION_LEN {
            selections.push(SingleSelectionProof::decode(d, ctx)?);
        }
        let scalar = ProofScalar::decode(d, ctx)?;

        Ok(Self { selections, scalar })
    }
}

impl Encode<()> for RowProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;
        0.encode(e, ctx)?;

        e.array(MIN_SELECTION_LEN * self.selections.len() as u64 + 1)?;
        for selection in &self.selections {
            selection.encode(e, ctx)?;
        }
        self.scalar.encode(e, ctx)
    }
}

impl Decode<'_, ()> for SingleSelectionProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let announcement = ProofAnnouncement::decode(d, ctx)?;
        let choice = ElgamalRistretto255Choice::decode(d, ctx)?;
        let response = ProofResponse::decode(d, ctx)?;

        Ok(Self {
            announcement,
            choice,
            response,
        })
    }
}

impl Encode<()> for SingleSelectionProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.announcement.encode(e, ctx)?;
        self.choice.encode(e, ctx)?;
        self.response.encode(e, ctx)
    }
}

impl Decode<'_, ()> for ProofScalar {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        <[u8; SCALAR_PROOF_LEN as usize]>::decode(d, ctx).map(Self)
    }
}

impl Encode<()> for ProofScalar {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for ProofAnnouncement {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self(
            ProofAnnouncementElement::decode(d, ctx)?,
            ProofAnnouncementElement::decode(d, ctx)?,
            ProofAnnouncementElement::decode(d, ctx)?,
        ))
    }
}

impl Encode<()> for ProofAnnouncement {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)?;
        self.1.encode(e, ctx)?;
        self.2.encode(e, ctx)
    }
}

impl Decode<'_, ()> for ProofResponse {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        Ok(Self(
            ProofScalar::decode(d, ctx)?,
            ProofScalar::decode(d, ctx)?,
            ProofScalar::decode(d, ctx)?,
        ))
    }
}

impl Encode<()> for ProofResponse {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)?;
        self.1.encode(e, ctx)?;
        self.2.encode(e, ctx)
    }
}

impl Decode<'_, ()> for ProofAnnouncementElement {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        <[u8; PROOF_ANNOUNCEMENT_ELEMENT_LEN as usize]>::decode(d, ctx).map(Self)
    }
}

impl Encode<()> for ProofAnnouncementElement {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_proof_roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = RowProof {
            selections: vec![SingleSelectionProof {
                announcement: ProofAnnouncement(
                    ProofAnnouncementElement(bytes),
                    ProofAnnouncementElement(bytes),
                    ProofAnnouncementElement(bytes),
                ),
                choice: ElgamalRistretto255Choice {
                    c1: bytes,
                    c2: bytes,
                },
                response: ProofResponse(ProofScalar(bytes), ProofScalar(bytes), ProofScalar(bytes)),
            }],
            scalar: ProofScalar(bytes),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = RowProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn single_selection_proof_roundtrip() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let original = SingleSelectionProof {
            announcement: ProofAnnouncement(
                ProofAnnouncementElement(bytes),
                ProofAnnouncementElement(bytes),
                ProofAnnouncementElement(bytes),
            ),
            choice: ElgamalRistretto255Choice {
                c1: bytes,
                c2: bytes,
            },
            response: ProofResponse(ProofScalar(bytes), ProofScalar(bytes), ProofScalar(bytes)),
        };
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = SingleSelectionProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn proof_scalar_roundtrip() {
        let original = ProofScalar([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = ProofScalar::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
