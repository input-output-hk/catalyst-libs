//! Catalyst Signed Document Content Payload

/// Document Content bytes (COSE payload).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Content(Option<Vec<u8>>);

impl Content {
    /// Return content bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.0.as_deref().unwrap_or(&[])
    }
}

impl From<Vec<u8>> for Content {
    fn from(value: Vec<u8>) -> Self {
        Self(Some(value))
    }
}

impl minicbor::Encode<()> for Content {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match &self.0 {
            Some(bytes) => e.bytes(bytes)?,
            None => e.null()?,
        };
        Ok(())
    }

    fn is_nil(&self) -> bool {
        self.0.is_none()
    }
}

impl minicbor::Decode<'_, ()> for Content {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let p = d.position();
        d.null()
            .map(|()| Self(None))
            // important to use `or_else` so it will lazy evaluated at the time when it is needed
            .or_else(|_| {
                d.set_position(p);
                d.bytes().map(Vec::from).map(Some).map(Self)
            })
    }
}
