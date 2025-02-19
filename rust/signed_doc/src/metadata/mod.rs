//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod algorithm;
mod content_encoding;
mod content_type;
mod document_ref;
mod extra_fields;
mod section;

use algorithm::Algorithm;
use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, UuidV4, UuidV7},
};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
use coset::{iana::CoapContentFormat, CborSerializable};
pub use document_ref::DocumentRef;
pub use extra_fields::ExtraFields;
pub use section::Section;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    alg: Option<Algorithm>,
    /// Document Type `UUIDv4`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    doc_type: Option<UuidV4>,
    /// Document ID `UUIDv7`.
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<UuidV7>,
    /// Document Version `UUIDv7`.
    #[serde(skip_serializing_if = "Option::is_none")]
    ver: Option<UuidV7>,
    /// Document Payload Content Type.
    #[serde(rename = "content-type")]
    content_type: Option<ContentType>,
    /// Document Payload Content Encoding.
    #[serde(rename = "content-encoding", skip_serializing_if = "Option::is_none")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: ExtraFields,
}

impl Metadata {
    /// Return Document Cryptographic Algorithm
    ///
    /// # Errros
    /// - Missing 'alg' field.
    #[must_use]
    pub fn algorithm(&self) -> anyhow::Result<Algorithm> {
        self.alg.ok_or(anyhow::anyhow!("Missing 'alg' field"))
    }

    /// Return Document Type `UUIDv4`.
    ///
    /// # Errros
    /// - Missing 'type' field.
    #[must_use]
    pub fn doc_type(&self) -> anyhow::Result<UuidV4> {
        self.doc_type.ok_or(anyhow::anyhow!("Missing 'type' field"))
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errros
    /// - Missing 'id' field.
    #[must_use]
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.id.ok_or(anyhow::anyhow!("Missing 'id' field"))
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errros
    /// - Missing 'ver' field.
    #[must_use]
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.ver.ok_or(anyhow::anyhow!("Missing 'ver' field"))
    }

    /// Returns the Document Content Type, if any.
    ///
    /// # Errros
    /// - Missing 'content-type' field.
    #[must_use]
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
        protected: &coset::ProtectedHeader, error_report: &ProblemReport,
    ) -> Self {
        /// Context for error messages.
        const CONTEXT: &str = "COSE Protected Header to Metadata";

        let extra = ExtraFields::from_protected_header(protected, error_report);
        let mut metadata = Metadata {
            extra,
            ..Metadata::default()
        };

        if let Some(coset::RegisteredLabelWithPrivate::Assigned(alg)) = protected.header.alg {
            match Algorithm::try_from(alg) {
                Ok(alg) => metadata.alg = Some(alg),
                Err(e) => {
                    error_report.conversion_error(
                        "COSE protected header algorithm",
                        &format!("{alg:?}"),
                        &format!("Expected Algorithm: {e}"),
                        &format!("{CONTEXT}, Algorithm"),
                    );
                },
            }
        } else {
            error_report.missing_field("alg", "Missing alg field in COSE protected header");
        }

        if let Some(value) = protected.header.content_type.as_ref() {
            match ContentType::try_from(value) {
                Ok(ct) => metadata.content_type = Some(ct),
                Err(e) => {
                    error_report.conversion_error(
                        "COSE protected header content type",
                        &format!("{value:?}"),
                        &format!("Expected ContentType: {e}"),
                        &format!("{CONTEXT}, ContentType"),
                    );
                },
            }
        } else {
            error_report.missing_field(
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
                    error_report.conversion_error(
                        "COSE protected header content encoding",
                        &format!("{value:?}"),
                        &format!("Expected ContentEncoding: {e}"),
                        &format!("{CONTEXT}, ContentEncoding"),
                    );
                },
            }
        }

        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(TYPE_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => metadata.doc_type = Some(uuid),
                Err(e) => {
                    error_report.conversion_error(
                        "COSE protected header type",
                        &format!("{value:?}"),
                        &format!("Expected UUID: {e:?}"),
                        &format!("{CONTEXT}, decoding CBOR UUID for type"),
                    );
                },
            }
        } else {
            error_report.missing_field("type", "Missing type field in COSE protected header");
        }

        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => metadata.id = Some(uuid),
                Err(e) => {
                    error_report.conversion_error(
                        "COSE protected header ID",
                        &format!("{value:?}"),
                        &format!("Expected UUID: {e:?}"),
                        &format!("{CONTEXT}, decoding CBOR UUID for ID"),
                    );
                },
            }
        } else {
            error_report.missing_field("id", "Missing id field in COSE protected header");
        }

        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(VER_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => metadata.ver = Some(uuid),
                Err(e) => {
                    error_report.conversion_error(
                        "COSE protected header ver",
                        &format!("{value:?}"),
                        &format!("Expected UUID: {e:?}"),
                        &format!("{CONTEXT}, decoding CBOR UUID for version"),
                    );
                },
            }
        } else {
            error_report.missing_field("ver", "Missing ver field in COSE protected header");
        }

        if let Some(id) = metadata.id {
            if let Some(ver) = metadata.ver {
                if ver < id {
                    error_report.invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < id",
                        &format!(
                            "{CONTEXT}, Document Version {ver} cannot be smaller than Document ID {id}"
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
            .text_value(TYPE_KEY.to_string(), encode_cbor_uuid(meta.doc_type()?)?)
            .text_value(ID_KEY.to_string(), encode_cbor_uuid(meta.doc_id()?)?)
            .text_value(VER_KEY.to_string(), encode_cbor_uuid(meta.doc_ver()?)?);

        builder = meta.extra.fill_cose_header_fields(builder)?;

        Ok(builder.build())
    }
}

/// Find a value for a predicate in the protected header.
fn cose_protected_header_find(
    protected: &coset::ProtectedHeader, mut predicate: impl FnMut(&coset::Label) -> bool,
) -> Option<&coset::cbor::Value> {
    protected
        .header
        .rest
        .iter()
        .find(|(key, _)| predicate(key))
        .map(|(_, value)| value)
}

/// Encode `uuid::Uuid` type into `coset::cbor::Value`.
///
/// This is used to encode `UuidV4` and `UuidV7` types.
pub(crate) fn encode_cbor_uuid<T: minicbor::encode::Encode<CborContext>>(
    value: T,
) -> anyhow::Result<coset::cbor::Value> {
    let mut cbor_bytes = Vec::new();
    minicbor::encode_with(value, &mut cbor_bytes, &mut CborContext::Tagged)
        .map_err(|e| anyhow::anyhow!("Unable to encode CBOR value, err: {e}"))?;
    coset::cbor::Value::from_slice(&cbor_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid CBOR value, err: {e}"))
}

/// Decode `From<uuid::Uuid>` type from `coset::cbor::Value`.
///
/// This is used to decode `UuidV4` and `UuidV7` types.
pub(crate) fn decode_cbor_uuid<T: for<'a> minicbor::decode::Decode<'a, CborContext>>(
    value: coset::cbor::Value,
) -> anyhow::Result<T> {
    match value.to_vec() {
        Ok(cbor_value) => {
            minicbor::decode_with(&cbor_value, &mut CborContext::Tagged)
                .map_err(|e| anyhow::anyhow!("Invalid UUID, err: {e}"))
        },
        Err(e) => anyhow::bail!("Invalid CBOR value, err: {e}"),
    }
}
