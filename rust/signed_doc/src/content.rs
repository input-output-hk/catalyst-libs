//! Catalyst Signed Document Content Payload

/// Document Content bytes (COSE payload).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Content(Vec<u8>);

impl Content {
    /// Return content bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Return content byte size.
    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<u8>> for Content {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl minicbor::Encode<()> for Content {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.0.is_empty() {
            e.null()?;
        } else {
            e.bytes(self.0.as_slice())?;
        }
        Ok(())
    }
}

impl minicbor::Decode<'_, ()> for Content {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let p = d.position();
        d.null()
            .map(|()| Self(Vec::new()))
            // important to use `or_else` so it will lazy evaluated at the time when it is needed
            .or_else(|_| {
                d.set_position(p);
                d.bytes()
                    .map_err(|_| minicbor::decode::Error::message("fuck you"))
                    .map(Vec::from)
                    .map(Self)
            })
    }
}
