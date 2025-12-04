//! A universal encrypted row proof.

use minicbor::{Decode, Decoder, Encode, Encoder, encode::Write};

/// A universal encrypted row proof.
///
/// The CDDL schema:
/// ```cddl
/// row-proof = [0, zkproof-elgamal-ristretto255-unit-vector-with-single-selection ]
///
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection = [ +zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item, zkproof-ed25519-scalar ]
///
/// zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item = ( zkproof-elgamal-announcement, ~elgamal-ristretto255-encrypted-choice, zkproof-ed25519-r-response )
///
/// zkproof-elgamal-announcement = ( zkproof-elgamal-group-element, zkproof-elgamal-group-element, zkproof-elgamal-group-element )
///
/// zkproof-elgamal-group-element = bytes .size 32
///
/// zkproof-ed25519-r-response = ( zkproof-ed25519-scalar, zkproof-ed25519-scalar, zkproof-ed25519-scalar )
///
/// zkproof-ed25519-scalar = bytes .size 32
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RowProof {
    /// A list of a single selection proofs.
    pub selections: Vec<()>,
    /// An individual Ed25519 scalar used in ZK proofs.
    pub scalar: (),
}

impl Decode<'_, ()> for RowProof {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        // TODO: FIXME:
        todo!()
    }
}

impl Encode<()> for RowProof {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: FIXME:
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let original = RowProof {};
        let mut buffer = Vec::new();
        original
            .encode(&mut Encoder::new(&mut buffer), &mut ())
            .unwrap();
        let decoded = RowProof::decode(&mut Decoder::new(&buffer), &mut ()).unwrap();
        assert_eq!(original, decoded);
    }
}
