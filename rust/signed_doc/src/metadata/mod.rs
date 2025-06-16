//! Catalyst Signed Document Metadata.
use std::{
    collections::{btree_map, BTreeMap},
    error::Error,
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
use coset::CborSerializable;
pub use doc_type::DocType;
pub use document_ref::DocumentRef;
use minicbor::{Decode, Decoder};
pub use section::Section;
use strum::IntoDiscriminant as _;
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

/// [`Metadata`] encoding context for the [`minicbor::Encode`] implementation.
pub(crate) struct MetadataEncodeContext {
    /// Used by some fields' encoding implementations.
    pub uuid_context: catalyst_types::uuid::CborContext,
}

impl minicbor::Encode<MetadataEncodeContext> for Metadata {
    /// Encode as a CBOR map.
    ///
    /// Note that to put it in an [RFC 8152] protected header.
    /// The header must be then encoded as a binary string.
    ///
    /// Also note that this won't check the presence of the required fields,
    /// so the checks must be done elsewhere.
    ///
    /// [RFC 8152]: https://datatracker.ietf.org/doc/html/rfc8152#autoid-8
    #[allow(
        clippy::cast_possible_truncation,
        reason = "There can't be enough unique fields to overflow `u64`."
    )]
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut MetadataEncodeContext,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        self.0
            .values()
            .try_fold(e, |e, field| e.encode_with(field, ctx))?
            .ok()
    }
}

/// [`Metadata`] decoding context for the [`minicbor::Decode`] implementation.
pub(crate) struct MetadataDecodeContext {
    /// Used by some fields' decoding implementations.
    pub uuid_context: catalyst_types::uuid::CborContext,
    /// Used by some fields' decoding implementations.
    pub compatibility_policy: crate::CompatibilityPolicy,
    /// Used by some fields' decoding implementations.
    pub report: ProblemReport,
}

impl MetadataDecodeContext {
    /// [`DocType`] decoding context.
    fn doc_type_context(&mut self) -> crate::decode_context::DecodeContext {
        crate::decode_context::DecodeContext {
            compatibility_policy: self.compatibility_policy,
            report: &mut self.report,
        }
    }
}

/// An error that's been reported, but doesn't affect the further decoding.
/// [`minicbor::Decoder`] should be assumed to be in a correct state and advanced towards
/// the next item.
///
/// The wrapped error can be returned up the call stack.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct TransientDecodeError(pub minicbor::decode::Error);

/// Creates a [`TransientDecodeError`] and wraps it in a
/// [`minicbor::decode::Error::custom`].
fn custom_transient_decode_error(
    message: &str, position: Option<usize>,
) -> minicbor::decode::Error {
    let mut inner = minicbor::decode::Error::message(message);
    if let Some(pos) = position {
        inner = inner.at(pos);
    }
    minicbor::decode::Error::custom(TransientDecodeError(inner))
}

impl minicbor::Decode<'_, MetadataDecodeContext> for Metadata {
    /// Decode from a CBOR map.
    ///
    /// Note that this won't decode an [RFC 8152] protected header as is.
    /// The header must be first decoded as a binary string.
    ///
    /// Also note that this won't check the absence of the required fields,
    /// so the checks must be done elsewhere.
    ///
    /// [RFC 8152]: https://datatracker.ietf.org/doc/html/rfc8152#autoid-8
    fn decode(
        d: &mut Decoder<'_>, ctx: &mut MetadataDecodeContext,
    ) -> Result<Self, minicbor::decode::Error> {
        const REPORT_CONTEXT: &str = "Metadata decoding";

        let Some(len) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "Indefinite map is not supported",
            ));
        };

        // TODO: verify key order.
        // TODO: use helpers from <https://github.com/input-output-hk/catalyst-libs/pull/360> once it's merged.

        let mut metadata_map = BTreeMap::new();
        let mut first_err = None;

        // This will return an error on the end of input.
        for _ in 0..len {
            let entry_pos = d.position();
            match d.decode_with::<_, SupportedField>(ctx) {
                Ok(field) => {
                    let label = field.discriminant();
                    let entry = metadata_map.entry(label);
                    if let btree_map::Entry::Vacant(entry) = entry {
                        entry.insert(field);
                    } else {
                        ctx.report.duplicate_field(
                            &label.to_string(),
                            "Duplicate metadata fields are not allowed",
                            REPORT_CONTEXT,
                        );
                        first_err.get_or_insert(custom_transient_decode_error(
                            "Duplicate fields",
                            Some(entry_pos),
                        ));
                    }
                },
                Err(err)
                    if err
                        .source()
                        .is_some_and(<dyn std::error::Error>::is::<TransientDecodeError>) =>
                {
                    first_err.get_or_insert(err);
                },
                Err(err) => return Err(err),
            }
        }

        first_err.map_or(Ok(Self(metadata_map)), Err)
    }
}
