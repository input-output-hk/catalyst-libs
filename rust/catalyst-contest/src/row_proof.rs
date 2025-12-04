//! A universal encrypted row proof.

use cbork_utils::decode_helper::decode_array_len;
use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

use crate::ElgamalRistretto255Choice;

const SCALAR_PROOF_LEN: u64 = 32;

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
pub struct ProofAnnouncement {}

/// A ZK proof response values for Ed25519.
///
/// The CDDL schema:
///
/// ```cddl
/// zkproof-ed25519-r-response = ( zkproof-ed25519-scalar, zkproof-ed25519-scalar, zkproof-ed25519-scalar )
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProofResponse {}

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
        if len < 2 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected row proof single selection array length {len}, expected ata least 2"
            )));
        }

        let mut selections = Vec::with_capacity(len as usize - 1);
        for _ in 0..len - 1 {
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

        e.array(self.selections.len() as u64 + 1)?;
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
        let len = decode_array_len(d, "row proof")?;
        if len != 3 {
            return Err(minicbor::decode::Error::message(format!(
                "Unexpected row proof array length {len}, expected 3"
            )));
        }

        let announcement

        /*
        pub announcement: ProofAnnouncement,
    /// An elgamal encrypted ciphertext.
    pub choice: ElgamalRistretto255Choice,
    /// A ZK proof response values for Ed25519.
    pub response: ProofResponse,
         */
        // TODO: FIXME:
        todo!()
    }
}

impl Encode<()> for SingleSelectionProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: FIXME:
        todo!()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_proof_roundtrip() {
        let original = RowProof {};
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = RowProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn single_selection_proof_roundtrip() {
        let original = SingleSelectionProof {};
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
