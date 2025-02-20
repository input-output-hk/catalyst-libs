//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod algorithm;
mod content_encoding;
mod content_type;
mod document_ref;
mod extra_fields;
mod section;
pub(crate) mod utils;

pub use algorithm::Algorithm;
use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{UuidV4, UuidV7},
};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
use coset::{cbor::Value, iana::CoapContentFormat};
pub use document_ref::DocumentRef;
pub use extra_fields::ExtraFields;
pub use section::Section;
use utils::{
    cose_protected_header_find, decode_document_field_from_protected_header, validate_option,
    CborUuidV4, CborUuidV7,
};

/// `content_encoding` field COSE key value
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";
/// `doc_type` field COSE key value
const TYPE_KEY: &str = "type";
/// `id` field COSE key value
const ID_KEY: &str = "id";
/// `ver` field COSE key value
const VER_KEY: &str = "ver";

/// Document Metadata.
///
/// These values are extracted from the COSE Sign protected header.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, Default)]
pub struct Metadata {
    /// Cryptographic Algorithm
    #[serde(deserialize_with = "validate_option")]
    alg: Option<Algorithm>,
    /// Document Type `UUIDv4`.
    #[serde(rename = "type", deserialize_with = "validate_option")]
    doc_type: Option<UuidV4>,
    /// Document ID `UUIDv7`.
    #[serde(deserialize_with = "validate_option")]
    id: Option<UuidV7>,
    /// Document Version `UUIDv7`.
    #[serde(deserialize_with = "validate_option")]
    ver: Option<UuidV7>,
    /// Document Payload Content Type.
    #[serde(rename = "content-type", deserialize_with = "validate_option")]
    content_type: Option<ContentType>,
    /// Document Payload Content Encoding.
    #[serde(rename = "content-encoding")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: ExtraFields,
}

impl Metadata {
    /// Return Document Cryptographic Algorithm
    ///
    /// # Errors
    /// - Missing 'alg' field.
    pub fn algorithm(&self) -> anyhow::Result<Algorithm> {
        self.alg.ok_or(anyhow::anyhow!("Missing 'alg' field"))
    }

    /// Return Document Type `UUIDv4`.
    ///
    /// # Errors
    /// - Missing 'type' field.
    pub fn doc_type(&self) -> anyhow::Result<UuidV4> {
        self.doc_type.ok_or(anyhow::anyhow!("Missing 'type' field"))
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'id' field.
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.id.ok_or(anyhow::anyhow!("Missing 'id' field"))
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'ver' field.
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.ver.ok_or(anyhow::anyhow!("Missing 'ver' field"))
    }

    /// Returns the Document Content Type, if any.
    ///
    /// # Errors
    /// - Missing 'content-type' field.
    pub fn content_type(&self) -> anyhow::Result<ContentType> {
        self.content_type
            .ok_or(anyhow::anyhow!("Missing 'content-type' field"))
    }

    /// Returns the Document Content Encoding, if any.
    #[must_use]
    pub fn content_encoding(&self) -> Option<ContentEncoding> {
        self.content_encoding
    }

    /// Return reference to additional metadata fields.
    #[must_use]
    pub fn extra(&self) -> &ExtraFields {
        &self.extra
    }

    /// Converting COSE Protected Header to Metadata.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_protected_header(
        protected: &coset::ProtectedHeader, report: &ProblemReport,
    ) -> Self {
        /// Context for problem report messages during decoding from COSE protected
        /// header.
        const COSE_DECODING_CONTEXT: &str = "COSE Protected Header to Metadata";

        let extra = ExtraFields::from_protected_header(protected, report);
        let mut metadata = Metadata {
            extra,
            ..Metadata::default()
        };

        if let Some(coset::RegisteredLabelWithPrivate::Assigned(alg)) = protected.header.alg {
            match Algorithm::try_from(alg) {
                Ok(alg) => metadata.alg = Some(alg),
                Err(e) => {
                    report.conversion_error(
                        "COSE protected header algorithm",
                        &format!("{alg:?}"),
                        &format!("Expected Algorithm: {e}"),
                        &format!("{COSE_DECODING_CONTEXT}, Algorithm"),
                    );
                },
            }
        } else {
            report.missing_field("alg", "Missing alg field in COSE protected header");
        }

        if let Some(value) = protected.header.content_type.as_ref() {
            match ContentType::try_from(value) {
                Ok(ct) => metadata.content_type = Some(ct),
                Err(e) => {
                    report.conversion_error(
                        "COSE protected header content type",
                        &format!("{value:?}"),
                        &format!("Expected ContentType: {e}"),
                        &format!("{COSE_DECODING_CONTEXT}, ContentType"),
                    );
                },
            }
        } else {
            report.missing_field(
                "content type",
                "Missing content_type field in COSE protected header",
            );
        }

        if let Some(value) = cose_protected_header_find(
            protected,
            |key| matches!(key, coset::Label::Text(label) if label.eq_ignore_ascii_case(CONTENT_ENCODING_KEY)),
        ) {
            match ContentEncoding::try_from(value) {
                Ok(ce) => metadata.content_encoding = Some(ce),
                Err(e) => {
                    report.conversion_error(
                        "COSE protected header content encoding",
                        &format!("{value:?}"),
                        &format!("Expected ContentEncoding: {e}"),
                        &format!("{COSE_DECODING_CONTEXT}, ContentEncoding"),
                    );
                },
            }
        }

        metadata.doc_type = decode_document_field_from_protected_header::<CborUuidV4>(
            protected,
            TYPE_KEY,
            COSE_DECODING_CONTEXT,
            report,
        )
        .map(|v| v.0);
        if metadata.doc_type.is_none() {
            report.missing_field("type", "Missing type field in COSE protected header");
        }

        metadata.id = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            ID_KEY,
            COSE_DECODING_CONTEXT,
            report,
        )
        .map(|v| v.0);
        if metadata.id.is_none() {
            report.missing_field("id", "Missing id field in COSE protected header");
        }

        metadata.ver = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            VER_KEY,
            COSE_DECODING_CONTEXT,
            report,
        )
        .map(|v| v.0);
        if metadata.ver.is_none() {
            report.missing_field("ver", "Missing ver field in COSE protected header");
        }

        if let Some(id) = metadata.id {
            if let Some(ver) = metadata.ver {
                if ver < id {
                    report.invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < id",
                        &format!(
                            "{COSE_DECODING_CONTEXT}, Document Version {ver} cannot be smaller than Document ID {id}"
                        ),
                    );
                }
            }
        }

        metadata
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  type: {:?},", self.doc_type)?;
        writeln!(f, "  id: {:?},", self.id)?;
        writeln!(f, "  ver: {:?},", self.ver)?;
        writeln!(f, "  alg: {:?},", self.alg)?;
        writeln!(f, "  content_type: {:?}", self.content_type)?;
        writeln!(f, "  content_encoding: {:?}", self.content_encoding)?;
        writeln!(f, "  additional_fields: {:?},", self.extra)?;
        writeln!(f, "}}")
    }
}

impl TryFrom<&Metadata> for coset::Header {
    type Error = anyhow::Error;

    fn try_from(meta: &Metadata) -> Result<Self, Self::Error> {
        let mut builder = coset::HeaderBuilder::new()
            .algorithm(
                meta.alg
                    .ok_or(anyhow::anyhow!("missing `alg` field"))?
                    .into(),
            )
            .content_format(CoapContentFormat::from(meta.content_type()?));

        if let Some(content_encoding) = meta.content_encoding() {
            builder = builder.text_value(
                CONTENT_ENCODING_KEY.to_string(),
                format!("{content_encoding}").into(),
            );
        }

        builder = builder
            .text_value(
                TYPE_KEY.to_string(),
                Value::try_from(CborUuidV4(meta.doc_type()?))?,
            )
            .text_value(
                ID_KEY.to_string(),
                Value::try_from(CborUuidV7(meta.doc_id()?))?,
            )
            .text_value(
                VER_KEY.to_string(),
                Value::try_from(CborUuidV7(meta.doc_ver()?))?,
            );

        builder = meta.extra.fill_cose_header_fields(builder)?;

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_serde_test() {
        let alg = Algorithm::EdDSA;
        let uuid_v7 = UuidV7::new();
        let uuid_v4 = UuidV4::new();
        let content_type = ContentType::Json;

        let valid = serde_json::json!({
            "alg": alg.to_string(),
            "content-type": content_type.to_string(),
            "type": uuid_v4.to_string(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),

        });
        assert!(serde_json::from_value::<Metadata>(valid).is_ok());

        let missing_alg = serde_json::json!({
            "content-type": content_type.to_string(),
            "type": uuid_v4.to_string(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),

        });
        assert!(serde_json::from_value::<Metadata>(missing_alg).is_err());

        let missing_content_type = serde_json::json!({
            "alg": alg.to_string(),
            "type": uuid_v4.to_string(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
        });
        assert!(serde_json::from_value::<Metadata>(missing_content_type).is_err());

        let missing_type = serde_json::json!({
            "alg": alg.to_string(),
            "content-type": content_type.to_string(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),

        });
        assert!(serde_json::from_value::<Metadata>(missing_type).is_err());

        let missing_id = serde_json::json!({
            "alg": alg.to_string(),
            "content-type": content_type.to_string(),
            "type": uuid_v4.to_string(),
            "ver": uuid_v7.to_string(),

        });
        assert!(serde_json::from_value::<Metadata>(missing_id).is_err());

        let missing_ver = serde_json::json!({
            "alg": alg.to_string(),
            "content-type": content_type.to_string(),
            "type": uuid_v4.to_string(),
            "id": uuid_v7.to_string(),
        });
        assert!(serde_json::from_value::<Metadata>(missing_ver).is_err());
    }
}
