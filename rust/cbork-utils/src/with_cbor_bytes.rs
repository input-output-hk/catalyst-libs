//! A helper struct which preserves original CBOR bytes of the decoded object

use std::{convert::Infallible, ops::Deref};

/// A helper immutable data structure, which holds original CBOR bytes of the object with
/// the object themselves.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WithCborBytes<T> {
    /// original CBOR bytes of the `obj`
    cbor_bytes: Vec<u8>,
    /// underlying `T` instance
    obj: T,
}

impl<T> Deref for WithCborBytes<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T> WithCborBytes<T> {
    /// Creates a new instance of the `WithCborBytes` from the provided `obj` by encoding
    /// it and storing resulted `cbor_bytes`.
    ///
    /// # Errors
    ///  - Infallible
    pub fn new<C>(
        obj: T,
        ctx: &mut C,
    ) -> Result<Self, minicbor::encode::Error<Infallible>>
    where
        T: minicbor::Encode<C>,
    {
        let cbor_bytes = minicbor::to_vec_with(&obj, ctx)?;
        Ok(Self { cbor_bytes, obj })
    }

    /// Return inner `obj` instance
    pub fn inner(self) -> T {
        self.obj
    }
}

impl<T> minicbor::Encode<()> for WithCborBytes<T> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.writer_mut()
            .write_all(&self.cbor_bytes)
            .map_err(minicbor::encode::Error::write)?;
        Ok(())
    }
}

impl<'a, C, T: minicbor::Decode<'a, C>> minicbor::Decode<'a, C> for WithCborBytes<T> {
    fn decode(
        d: &mut minicbor::Decoder<'a>,
        ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let start = d.position();
        let obj = d.decode_with(ctx)?;
        let end = d.position();
        let cbor_bytes = d
            .input()
            .get(start..end)
            .ok_or(minicbor::decode::Error::end_of_input())?
            .to_vec();
        Ok(Self { cbor_bytes, obj })
    }
}
