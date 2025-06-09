//! Catalyst Signed Document Metadata.
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

mod content_encoding;
mod content_type;
pub(crate) mod doc_type;
mod document_ref;
mod section;
mod supported_field;
pub(crate) mod utils;

use catalyst_types::{problem_report::ProblemReport, uuid::UuidV7};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
use coset::{cbor::Value, iana::CoapContentFormat, CborSerializable};
pub use doc_type::DocType;
pub use document_ref::DocumentRef;
use minicbor::{Decode, Decoder};
pub use section::Section;
use strum::IntoDiscriminant;
use utils::{cose_protected_header_find, decode_document_field_from_protected_header, CborUuidV7};

use crate::{
    decode_context::DecodeContext,
    metadata::supported_field::{SupportedField, SupportedLabel},
};

/// `content_encoding` field COSE key value
const CONTENT_ENCODING_KEY: &str = "Content-Encoding";
/// `doc_type` field COSE key value
const TYPE_KEY: &str = "type";
/// `id` field COSE key value
const ID_KEY: &str = "id";
/// `ver` field COSE key value
const VER_KEY: &str = "ver";

/// `ref` field COSE key value
const REF_KEY: &str = "ref";
/// `template` field COSE key value
const TEMPLATE_KEY: &str = "template";
/// `reply` field COSE key value
const REPLY_KEY: &str = "reply";
/// `section` field COSE key value
const SECTION_KEY: &str = "section";
/// `collabs` field COSE key value
const COLLABS_KEY: &str = "collabs";
/// `parameters` field COSE key value
const PARAMETERS_KEY: &str = "parameters";
/// `brand_id` field COSE key value (alias of the `parameters` field)
const BRAND_ID_KEY: &str = "brand_id";
/// `campaign_id` field COSE key value (alias of the `parameters` field)
const CAMPAIGN_ID_KEY: &str = "campaign_id";
/// `category_id` field COSE key value (alias of the `parameters` field)
const CATEGORY_ID_KEY: &str = "category_id";

/// Document Metadata.
///
/// These values are extracted from the COSE Sign protected header.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Metadata(BTreeMap<SupportedLabel, SupportedField>);

/// An actual representation of all metadata fields.
// TODO: this is maintained as an implementation of `serde` and `coset` for `Metadata`
//       and should be removed in case `serde` and `coset` are deprecated completely.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, Default)]
pub(crate) struct InnerMetadata {
    /// Document Type, list of `UUIDv4`.
    #[serde(rename = "type")]
    doc_type: Option<DocType>,
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
    /// Reference to the latest document.
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    doc_ref: Option<DocumentRef>,
    /// Reference to the document template.
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<DocumentRef>,
    /// Reference to the document reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<DocumentRef>,
    /// Reference to the document section.
    #[serde(skip_serializing_if = "Option::is_none")]
    section: Option<Section>,
    /// Reference to the document collaborators. Collaborator type is TBD.
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    collabs: Vec<String>,
    /// Reference to the parameters document.
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<DocumentRef>,
}

impl InnerMetadata {
    /// Converts into an iterator over present fields fields.
    fn into_iter(self) -> impl Iterator<Item = SupportedField> {
        [
            self.doc_type.map(SupportedField::Type),
            self.id.map(SupportedField::Id),
            self.ver.map(SupportedField::Ver),
            self.content_type.map(SupportedField::ContentType),
            self.content_encoding.map(SupportedField::ContentEncoding),
            self.doc_ref.map(SupportedField::Ref),
            self.template.map(SupportedField::Template),
            self.reply.map(SupportedField::Reply),
            self.section.map(SupportedField::Section),
            (!self.collabs.is_empty()).then_some(SupportedField::Collabs(self.collabs)),
            self.parameters.map(SupportedField::Parameters),
        ]
        .into_iter()
        .flatten()
    }
}

impl Metadata {
    /// Return Document Type `DocType` - a list of `UUIDv4`.
    ///
    /// # Errors
    /// - Missing 'type' field.
    pub fn doc_type(&self) -> anyhow::Result<&DocType> {
        self.0
            .get(&SupportedLabel::Type)
            .and_then(SupportedField::try_as_type_ref)
            .ok_or(anyhow::anyhow!("Missing 'type' field"))
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'id' field.
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.0
            .get(&SupportedLabel::Id)
            .and_then(SupportedField::try_as_id_ref)
            .copied()
            .ok_or(anyhow::anyhow!("Missing 'id' field"))
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errors
    /// - Missing 'ver' field.
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.0
            .get(&SupportedLabel::Ver)
            .and_then(SupportedField::try_as_ver_ref)
            .copied()
            .ok_or(anyhow::anyhow!("Missing 'ver' field"))
    }

    /// Returns the Document Content Type, if any.
    ///
    /// # Errors
    /// - Missing 'content-type' field.
    pub fn content_type(&self) -> anyhow::Result<ContentType> {
        self.0
            .get(&SupportedLabel::ContentType)
            .and_then(SupportedField::try_as_content_type_ref)
            .copied()
            .ok_or(anyhow::anyhow!("Missing 'content-type' field"))
    }

    /// Returns the Document Content Encoding, if any.
    #[must_use]
    pub fn content_encoding(&self) -> Option<ContentEncoding> {
        self.0
            .get(&SupportedLabel::ContentEncoding)
            .and_then(SupportedField::try_as_content_encoding_ref)
            .copied()
    }

    /// Return `ref` field.
    #[must_use]
    pub fn doc_ref(&self) -> Option<DocumentRef> {
        self.0
            .get(&SupportedLabel::Ref)
            .and_then(SupportedField::try_as_ref_ref)
            .copied()
    }

    /// Return `template` field.
    #[must_use]
    pub fn template(&self) -> Option<DocumentRef> {
        self.0
            .get(&SupportedLabel::Template)
            .and_then(SupportedField::try_as_template_ref)
            .copied()
    }

    /// Return `reply` field.
    #[must_use]
    pub fn reply(&self) -> Option<DocumentRef> {
        self.0
            .get(&SupportedLabel::Reply)
            .and_then(SupportedField::try_as_reply_ref)
            .copied()
    }

    /// Return `section` field.
    #[must_use]
    pub fn section(&self) -> Option<&Section> {
        self.0
            .get(&SupportedLabel::Section)
            .and_then(SupportedField::try_as_section_ref)
    }

    /// Return `collabs` field.
    #[must_use]
    pub fn collabs(&self) -> &[String] {
        self.0
            .get(&SupportedLabel::Collabs)
            .and_then(SupportedField::try_as_collabs_ref)
            .map_or(&[], Vec::as_slice)
    }

    /// Return `parameters` field.
    #[must_use]
    pub fn parameters(&self) -> Option<DocumentRef> {
        self.0
            .get(&SupportedLabel::Parameters)
            .and_then(SupportedField::try_as_parameters_ref)
            .copied()
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

        Self(
            metadata
                .into_iter()
                .map(|field| (field.discriminant(), field))
                .collect(),
        )
    }

    /// Converting COSE Protected Header to Metadata.
    pub(crate) fn from_protected_header(
        protected: &coset::ProtectedHeader, context: &mut DecodeContext,
    ) -> Self {
        let metadata = InnerMetadata::from_protected_header(protected, context);
        Self::from_metadata_fields(metadata, context.report)
    }
}

impl InnerMetadata {
    /// Converting COSE Protected Header to Metadata fields, collecting decoding report
    /// issues.
    #[allow(
        clippy::too_many_lines,
        reason = "This is a compilation of `coset` decoding and should be replaced once migrated to `minicbor`."
    )]
    pub(crate) fn from_protected_header(
        protected: &coset::ProtectedHeader, context: &mut DecodeContext,
    ) -> Self {
        /// Context for problem report messages during decoding from COSE protected
        /// header.
        const COSE_DECODING_CONTEXT: &str = "COSE Protected Header to Metadata";

        let mut metadata = Self::default();

        if let Some(value) = protected.header.content_type.as_ref() {
            match ContentType::try_from(value) {
                Ok(ct) => metadata.content_type = Some(ct),
                Err(e) => {
                    context.report.conversion_error(
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
                    context.report.conversion_error(
                        "COSE protected header content encoding",
                        &format!("{value:?}"),
                        &format!("Expected ContentEncoding: {e}"),
                        &format!("{COSE_DECODING_CONTEXT}, ContentEncoding"),
                    );
                },
            }
        }

        metadata.doc_type = cose_protected_header_find(
            protected,
            |key| matches!(key, coset::Label::Text(label) if label.eq_ignore_ascii_case(TYPE_KEY)),
        )
        .and_then(|value| {
            DocType::decode(
                &mut Decoder::new(&value.clone().to_vec().unwrap_or_default()),
                context,
            )
            .ok()
        });

        metadata.id = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            ID_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        )
        .map(|v| v.0);

        metadata.ver = decode_document_field_from_protected_header::<CborUuidV7>(
            protected,
            VER_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        )
        .map(|v| v.0);

        metadata.doc_ref = decode_document_field_from_protected_header(
            protected,
            REF_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        );
        metadata.template = decode_document_field_from_protected_header(
            protected,
            TEMPLATE_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        );
        metadata.reply = decode_document_field_from_protected_header(
            protected,
            REPLY_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        );
        metadata.section = decode_document_field_from_protected_header(
            protected,
            SECTION_KEY,
            COSE_DECODING_CONTEXT,
            context.report,
        );

        // process `parameters` field and all its aliases
        let (parameters, has_multiple_fields) = [
            PARAMETERS_KEY,
            BRAND_ID_KEY,
            CAMPAIGN_ID_KEY,
            CATEGORY_ID_KEY,
        ]
        .iter()
        .filter_map(|field_name| -> Option<DocumentRef> {
            decode_document_field_from_protected_header(
                protected,
                field_name,
                COSE_DECODING_CONTEXT,
                context.report,
            )
        })
        .fold((None, false), |(res, _), v| (Some(v), res.is_some()));
        if has_multiple_fields {
            context.report.duplicate_field(
                    "brand_id, campaign_id, category_id", 
                    "Only value at the same time is allowed parameters, brand_id, campaign_id, category_id", 
                    "Validation of parameters field aliases"
                );
        }
        metadata.parameters = parameters;

        if let Some(cbor_doc_collabs) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text(COLLABS_KEY.to_string())
        }) {
            if let Ok(collabs) = cbor_doc_collabs.clone().into_array() {
                let mut c = Vec::new();
                for (ids, collaborator) in collabs.iter().cloned().enumerate() {
                    match collaborator.clone().into_text() {
                        Ok(collaborator) => {
                            c.push(collaborator);
                        },
                        Err(_) => {
                            context.report.conversion_error(
                                &format!("COSE protected header collaborator index {ids}"),
                                &format!("{collaborator:?}"),
                                "Expected a CBOR String",
                                &format!(
                                    "{COSE_DECODING_CONTEXT}, converting collaborator to String",
                                ),
                            );
                        },
                    }
                }
                metadata.collabs = c;
            } else {
                context.report.conversion_error(
                    "CBOR COSE protected header collaborators",
                    &format!("{cbor_doc_collabs:?}"),
                    "Expected a CBOR Array",
                    &format!("{COSE_DECODING_CONTEXT}, converting collaborators to Array",),
                );
            };
        }

        metadata
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Metadata {{")?;
        writeln!(f, "  type: {:?},", self.doc_type().ok())?;
        writeln!(f, "  id: {:?},", self.doc_id().ok())?;
        writeln!(f, "  ver: {:?},", self.doc_ver().ok())?;
        writeln!(f, "  content_type: {:?},", self.content_type().ok())?;
        writeln!(f, "  content_encoding: {:?},", self.content_encoding())?;
        writeln!(f, "  additional_fields: {{")?;
        writeln!(f, "    ref: {:?}", self.doc_ref())?;
        writeln!(f, "    template: {:?},", self.template())?;
        writeln!(f, "    reply: {:?},", self.reply())?;
        writeln!(f, "    section: {:?},", self.section())?;
        writeln!(f, "    collabs: {:?},", self.collabs())?;
        writeln!(f, "    parameters: {:?},", self.parameters())?;
        writeln!(f, "  }},")?;
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
            .text_value(TYPE_KEY.to_string(), meta.doc_type()?.to_value())
            .text_value(
                ID_KEY.to_string(),
                Value::try_from(CborUuidV7(meta.doc_id()?))?,
            )
            .text_value(
                VER_KEY.to_string(),
                Value::try_from(CborUuidV7(meta.doc_ver()?))?,
            );

        if let Some(doc_ref) = meta.doc_ref() {
            builder = builder.text_value(REF_KEY.to_string(), Value::try_from(doc_ref)?);
        }
        if let Some(template) = meta.template() {
            builder = builder.text_value(TEMPLATE_KEY.to_string(), Value::try_from(template)?);
        }
        if let Some(reply) = meta.reply() {
            builder = builder.text_value(REPLY_KEY.to_string(), Value::try_from(reply)?);
        }

        if let Some(section) = meta.section() {
            builder = builder.text_value(SECTION_KEY.to_string(), Value::from(section.clone()));
        }

        if !meta.collabs().is_empty() {
            builder = builder.text_value(
                COLLABS_KEY.to_string(),
                Value::Array(meta.collabs().iter().cloned().map(Value::Text).collect()),
            );
        }

        if let Some(parameters) = meta.parameters() {
            builder = builder.text_value(PARAMETERS_KEY.to_string(), Value::try_from(parameters)?);
        }

        Ok(builder.build())
    }
}
