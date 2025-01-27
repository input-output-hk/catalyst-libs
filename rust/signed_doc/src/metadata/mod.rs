//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod algorithm;
mod content_encoding;
mod content_type;
mod document_id;
mod document_ref;
mod document_type;
mod document_version;
mod extra_fields;

use algorithm::Algorithm;
use anyhow::{anyhow, bail};
use catalyst_types::problem_report::ProblemReport;
pub use catalyst_types::uuid::{CborContext, UuidV4, UuidV7};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
use coset::{iana::CoapContentFormat, CborSerializable};
pub use document_id::DocumentId;
pub use document_ref::DocumentRef;
pub use document_type::DocumentType;
pub use document_version::DocumentVersion;
pub use extra_fields::ExtraFields;

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
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct Metadata {
    /// Cryptographic Algorithm
    #[serde(default = "Algorithm::default")]
    alg: Algorithm,
    /// Document Type `UUIDv4`.
    #[serde(rename = "type")]
    doc_type: DocumentType,
    /// Document ID `UUIDv7`.
    id: DocumentId,
    /// Document Version `UUIDv7`.
    ver: DocumentVersion,
    /// Document Payload Content Type.
    #[serde(rename = "content-type")]
    content_type: ContentType,
    /// Document Payload Content Encoding.
    #[serde(rename = "content-encoding", skip_serializing_if = "Option::is_none")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: ExtraFields,
}

impl Metadata {
    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> UuidV4 {
        self.doc_type.into()
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> UuidV7 {
        self.id.into()
    }

    /// Return Document Version `UUIDv7`.
    #[must_use]
    pub fn doc_ver(&self) -> UuidV7 {
        self.ver.into()
    }

    /// Returns the Document Content Type, if any.
    #[must_use]
    pub fn content_type(&self) -> ContentType {
        self.content_type
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
    ) -> anyhow::Result<Self> {
        /// Context for error messages.
        const CONTEXT: &str = "COSE Protected Header to Metadata";

        let mut algorithm = Algorithm::default();
        if let Some(coset::RegisteredLabelWithPrivate::Assigned(alg)) = protected.header.alg {
            match Algorithm::try_from(alg) {
                Ok(alg) => algorithm = alg,
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

        let mut content_type = None;
        if let Some(value) = protected.header.content_type.as_ref() {
            match ContentType::try_from(value) {
                Ok(ct) => content_type = Some(ct),
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

        let mut content_encoding = None;
        if let Some(value) = cose_protected_header_find(
            protected,
            |key| matches!(key, coset::Label::Text(label) if label.eq_ignore_ascii_case(CONTENT_ENCODING_KEY)),
        ) {
            match ContentEncoding::try_from(value) {
                Ok(ce) => content_encoding = Some(ce),
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

        let mut doc_type: Option<UuidV4> = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(TYPE_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => doc_type = Some(uuid),
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

        let mut id: Option<UuidV7> = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => id = Some(uuid),
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

        let mut ver: Option<UuidV7> = None;
        if let Some(value) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(VER_KEY.to_string())
        }) {
            match decode_cbor_uuid(value.clone()) {
                Ok(uuid) => ver = Some(uuid),
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

        let extra = ExtraFields::from_protected_header(protected, error_report).map_or_else(
            |e| {
                error_report.conversion_error(
                    "COSE protected header",
                    &format!("{protected:?}"),
                    &format!("Expected ExtraField: {e}"),
                    &format!("{CONTEXT}, ExtraFields"),
                );
                None
            },
            Some,
        );

        match (content_type, content_encoding, id, doc_type, ver, extra) {
            (
                Some(content_type),
                content_encoding,
                Some(id),
                Some(doc_type),
                Some(ver),
                Some(extra),
            ) => {
                if ver < id {
                    error_report.invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < id",
                        &format!("{CONTEXT}, Document Version {ver} cannot be smaller than Document ID {id}"),
                    );

                    bail!("Failed to convert COSE Protected Header to Metadata: document version is smaller than document ID");
                }

                Ok(Self {
                    doc_type: doc_type.into(),
                    id: id.into(),
                    ver: ver.into(),
                    alg: algorithm,
                    content_encoding,
                    content_type,
                    extra,
                })
            },
            _ => bail!("Failed to convert COSE Protected Header to Metadata"),
        }
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  type: {},", self.doc_type)?;
        writeln!(f, "  id: {},", self.id)?;
        writeln!(f, "  ver: {},", self.ver)?;
        writeln!(f, "  alg: {:?},", self.alg)?;
        writeln!(f, "  content_type: {}", self.content_type)?;
        writeln!(f, "  content_encoding: {:?}", self.content_encoding)?;
        writeln!(f, "  additional_fields: {:?},", self.extra)?;
        writeln!(f, "}}")
    }
}

impl TryFrom<&Metadata> for coset::Header {
    type Error = anyhow::Error;

    fn try_from(meta: &Metadata) -> Result<Self, Self::Error> {
        let mut builder = coset::HeaderBuilder::new()
            .algorithm(meta.alg.into())
            .content_format(CoapContentFormat::from(meta.content_type()));

        if let Some(content_encoding) = meta.content_encoding() {
            builder = builder.text_value(
                CONTENT_ENCODING_KEY.to_string(),
                format!("{content_encoding}").into(),
            );
        }

        builder = builder
            .text_value(TYPE_KEY.to_string(), meta.doc_type.try_into()?)
            .text_value(ID_KEY.to_string(), meta.id.try_into()?)
            .text_value(VER_KEY.to_string(), meta.ver.try_into()?);

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
pub(crate) fn decode_cbor_uuid<
    T: for<'a> minicbor::decode::Decode<'a, CborContext> + TryFrom<uuid::Uuid>,
>(
    value: coset::cbor::Value,
) -> anyhow::Result<T> {
    match value.to_vec() {
        Ok(cbor_value) => {
            minicbor::decode_with(&cbor_value, &mut CborContext::Tagged)
                .map_err(|e| anyhow!("Invalid UUID, err: {e}"))
        },
        Err(e) => anyhow::bail!("Invalid CBOR value, err: {e}"),
    }
}
