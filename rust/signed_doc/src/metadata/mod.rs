//! Catalyst Signed Document Metadata.
use std::fmt::{Display, Formatter};

mod content_encoding;
mod content_type;
mod document_ref;
mod extra_fields;
mod section;
pub(crate) mod utils;

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
    cose_protected_header_find, decode_document_field_from_protected_header, CborUuidV4, CborUuidV7,
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
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Metadata(InnerMetadata);

/// An actual representation of all metadata fields.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, Default)]
pub(crate) struct InnerMetadata {
    /// Document Type `UUIDv4`.
    #[serde(rename = "type")]
    doc_type: Option<UuidV4>,
    /// Document ID `UUIDv7`.
    id: Option<UuidV7>,
    /// Document Version `UUIDv7`.
    ver: Option<UuidV7>,
    /// Document Payload Content Type.
    #[serde(rename = "content-type")]
    content_type: Option<ContentType>,
    /// Document Payload Content Encoding.
    #[serde(rename = "content-encoding")]
    content_encoding: Option<ContentEncoding>,
    /// Additional Metadata Fields.
    #[serde(flatten)]
    extra: ExtraFields,
}

impl Metadata {
    /// Return Document Type `UUIDv4`.
    ///
    /// # Errors
    /// - Missing 'type' field.
    pub fn doc_type(&self) -> anyhow::Result<UuidV4> {
        self.0
            .doc_type
            .ok_or(anyhow::anyhow!("Missing 'type' field"))
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'id' field.
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.0.id.ok_or(anyhow::anyhow!("Missing 'id' field"))
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'ver' field.
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.0.ver.ok_or(anyhow::anyhow!("Missing 'ver' field"))
    }

    /// Returns the Document Content Type, if any.
    ///
    /// # Errors
    /// - Missing 'content-type' field.
    pub fn content_type(&self) -> anyhow::Result<ContentType> {
        self.0
            .content_type
            .ok_or(anyhow::anyhow!("Missing 'content-type' field"))
    }

    /// Returns the Document Content Encoding, if any.
    #[must_use]
    pub fn content_encoding(&self) -> Option<ContentEncoding> {
        self.0.content_encoding
    }

    /// Return reference to additional metadata fields.
    #[must_use]
    pub fn extra(&self) -> &ExtraFields {
        &self.0.extra
    }

    /// Build `Metadata` object from the metadata fields, doing all necessary validation.
    pub(crate) fn from_metadata_fields(metadata: InnerMetadata, report: &ProblemReport) -> Self {
        if metadata.doc_type.is_none() {
            report.missing_field("type", "Missing type field in COSE protected header");
        }
        if metadata.id.is_none() {
            report.missing_field("id", "Missing id field in COSE protected header");
        }
        if metadata.ver.is_none() {
            report.missing_field("ver", "Missing ver field in COSE protected header");
        }

        if metadata.content_type.is_none() {
            report.missing_field(
                "content type",
                "Missing content_type field in COSE protected header",
            );
        }

        Self(metadata)
    }

    /// Converting COSE Protected Header to Metadata.
    pub(crate) fn from_protected_header(
        protected: &coset::ProtectedHeader, report: &ProblemReport,
    ) -> Self {
        let metadata = InnerMetadata::from_protected_header(protected, report);
        Self::from_metadata_fields(metadata, report)
    }
}

impl InnerMetadata {
    /// Converting COSE Protected Header to Metadata fields, collecting decoding report
    /// issues.
    pub(crate) fn from_protected_header(
        protected: &coset::ProtectedHeader, report: &ProblemReport,
    ) -> Self {
        /// Context for problem report messages during decoding from COSE protected
        /// header.
        const COSE_DECODING_CONTEXT: &str = "COSE Protected Header to Metadata";

        let extra = ExtraFields::from_protected_header(protected, report);
        let mut metadata = Self {
            extra,
            ..Self::default()
        };

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

        metadata.id = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            ID_KEY,
            COSE_DECODING_CONTEXT,
            report,
        )
        .map(|v| v.0);

        metadata.ver = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            VER_KEY,
            COSE_DECODING_CONTEXT,
            report,
        )
        .map(|v| v.0);

        metadata
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  type: {:?},", self.0.doc_type)?;
        writeln!(f, "  id: {:?},", self.0.id)?;
        writeln!(f, "  ver: {:?},", self.0.ver)?;
        writeln!(f, "  content_type: {:?}", self.0.content_type)?;
        writeln!(f, "  content_encoding: {:?}", self.0.content_encoding)?;
        writeln!(f, "  additional_fields: {:?},", self.0.extra)?;
        writeln!(f, "}}")
    }
}

impl TryFrom<&Metadata> for coset::Header {
    type Error = anyhow::Error;

    fn try_from(meta: &Metadata) -> Result<Self, Self::Error> {
        let mut builder = coset::HeaderBuilder::new()
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

        builder = meta.0.extra.fill_cose_header_fields(builder)?;

        Ok(builder.build())
    }
}
