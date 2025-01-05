//! Document Payload Content Type.

/// Payload Content Type.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged, rename_all_fields = "lowercase")]
pub enum ContentType {
    /// 'application/cbor'
    Cbor,
    /// 'application/json'
    Json,
}

impl TryFrom<&coset::ContentType> for ContentType {
    type Error = anyhow::Error;

    fn try_from(value: &coset::ContentType) -> Result<Self, Self::Error> {
        use coset::iana::CoapContentFormat as Format;
        match value {
            coset::ContentType::Assigned(Format::Json) => Ok(ContentType::Json),
            coset::ContentType::Assigned(Format::Cbor) => Ok(ContentType::Cbor),
            _ => anyhow::bail!("Unsupported Content Type {value:?}"),
        }
    }
}
