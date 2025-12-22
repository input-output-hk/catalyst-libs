//! Number of entries in a Sparse Merkle Tree (SMT).

use minicbor::{Decode, Encode, decode::Error as DecodeError, encode::Error as EncodeError};

/// Number of entries in a Sparse Merkle Tree (SMT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtEntries(u64);

impl From<u64> for SmtEntries {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SmtEntries> for u64 {
    fn from(value: SmtEntries) -> Self {
        value.0
    }
}

impl Encode<()> for SmtEntries {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), EncodeError<W::Error>> {
        e.u64(self.0)?;
        Ok(())
    }
}

impl Decode<'_, ()> for SmtEntries {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, DecodeError> {
        let entries = d.u64()?;
        Ok(Self(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let test_values = [0u64, 1, 42, 1000, u64::MAX];

        for value in test_values {
            let original = SmtEntries::from(value);

            let mut buffer = Vec::new();
            original
                .encode(&mut minicbor::Encoder::new(&mut buffer), &mut ())
                .unwrap();
            let decoded =
                SmtEntries::decode(&mut minicbor::Decoder::new(&buffer), &mut ()).unwrap();
            assert_eq!(original, decoded);
        }
    }
}
