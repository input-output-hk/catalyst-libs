//! Catalyst Signed Document Metadata.
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

mod collaborators;
mod content_encoding;
mod content_type;
pub(crate) mod doc_type;
mod document_refs;
mod section;
mod supported_field;

use catalyst_types::{catalyst_id::CatalystId, problem_report::ProblemReport, uuid::UuidV7};
pub use content_encoding::ContentEncoding;
pub use content_type::ContentType;
pub use doc_type::DocType;
pub use document_refs::{DocLocator, DocumentRef, DocumentRefs};
use minicbor::Decoder;
pub use section::Section;
use strum::IntoDiscriminant as _;

pub(crate) use crate::metadata::supported_field::{SupportedField, SupportedLabel};

/// Document Metadata.
///
/// These values are extracted from the COSE Sign protected header.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Metadata(BTreeMap<SupportedLabel, SupportedField>);

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
    pub fn doc_ref(&self) -> Option<&DocumentRefs> {
        self.0
            .get(&SupportedLabel::Ref)
            .and_then(SupportedField::try_as_ref_ref)
    }

    /// Return `template` field.
    #[must_use]
    pub fn template(&self) -> Option<&DocumentRefs> {
        self.0
            .get(&SupportedLabel::Template)
            .and_then(SupportedField::try_as_template_ref)
    }

    /// Return `reply` field.
    #[must_use]
    pub fn reply(&self) -> Option<&DocumentRefs> {
        self.0
            .get(&SupportedLabel::Reply)
            .and_then(SupportedField::try_as_reply_ref)
    }

    /// Return `section` field.
    #[must_use]
    pub fn section(&self) -> Option<&Section> {
        self.0
            .get(&SupportedLabel::Section)
            .and_then(SupportedField::try_as_section_ref)
    }

    /// Return `collaborators` field.
    #[must_use]
    pub fn collaborators(&self) -> &[CatalystId] {
        self.0
            .get(&SupportedLabel::Collaborators)
            .and_then(SupportedField::try_as_collaborators_ref)
            .map_or(&[], |v| &**v)
    }

    /// Return `parameters` field.
    #[must_use]
    pub fn parameters(&self) -> Option<&DocumentRefs> {
        self.0
            .get(&SupportedLabel::Parameters)
            .and_then(SupportedField::try_as_parameters_ref)
    }

    /// Add `SupportedField` into the `Metadata`.
    ///
    /// # Warning
    ///
    /// Building metadata by-field with this function doesn't ensure the presence of
    /// required fields. Use [`Self::from_fields`] or [`Self::from_json`] if it's
    /// important for metadata to be valid.
    #[cfg(test)]
    pub(crate) fn add_field(&mut self, field: SupportedField) {
        self.0.insert(field.discriminant(), field);
    }

    /// Build `Metadata` object from the metadata fields, doing all necessary validation.
    pub(crate) fn from_fields<E>(
        fields: impl Iterator<Item = Result<SupportedField, E>>, report: &ProblemReport,
    ) -> Result<Self, E> {
        const REPORT_CONTEXT: &str = "Metadata building";

        let mut metadata = Metadata(BTreeMap::new());
        for v in fields {
            let v = v?;
            let k = v.discriminant();
            if metadata.0.insert(k, v).is_some() {
                report.duplicate_field(
                    &k.to_string(),
                    "Duplicate metadata fields are not allowed",
                    REPORT_CONTEXT,
                );
            }
        }

        if metadata.doc_type().is_err() {
            report.missing_field("type", REPORT_CONTEXT);
        }
        if metadata.doc_id().is_err() {
            report.missing_field("id", REPORT_CONTEXT);
        }
        if metadata.doc_ver().is_err() {
            report.missing_field("ver", REPORT_CONTEXT);
        }
        if metadata.content_type().is_err() {
            report.missing_field("content-type", REPORT_CONTEXT);
        }

        Ok(metadata)
    }

    /// Build `Metadata` object from the metadata fields, doing all necessary validation.
    pub(crate) fn from_json(fields: serde_json::Value) -> anyhow::Result<Self> {
        let fields = serde::Deserializer::deserialize_map(fields, MetadataDeserializeVisitor)?;
        let report = ProblemReport::new("Deserializing metadata from json");
        let metadata = Self::from_fields(fields.into_iter().map(anyhow::Result::<_>::Ok), &report)?;
        anyhow::ensure!(!report.is_problematic(), "{:?}", report);
        Ok(metadata)
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
        writeln!(f, "    collaborators: {:?},", self.collaborators())?;
        writeln!(f, "    parameters: {:?},", self.parameters())?;
        writeln!(f, "  }},")?;
        writeln!(f, "}}")
    }
}

impl minicbor::Encode<()> for Metadata {
    /// Encode as a CBOR map.
    ///
    /// Note that to put it in an [RFC 8152] protected header.
    /// The header must be then encoded as a binary string.
    ///
    /// Also note that this won't check the presence of the required fields,
    /// so the checks must be done elsewhere.
    ///
    /// [RFC 8152]: https://datatracker.ietf.org/doc/html/rfc8152#autoid-8
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(
            self.0
                .len()
                .try_into()
                .map_err(minicbor::encode::Error::message)?,
        )?;
        self.0
            .values()
            .try_fold(e, |e, field| e.encode(field))?
            .ok()
    }
}

impl minicbor::Decode<'_, crate::decode_context::DecodeContext<'_>> for Metadata {
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
        d: &mut Decoder<'_>, ctx: &mut crate::decode_context::DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        // TODO: use helpers from `cbork-utils` crate to verify that's the map is
        // deterministically CBOR encoded map.

        let Some(length) = d.map()? else {
            return Err(minicbor::decode::Error::message(
                "COSE protected headers object must be a definite size map ",
            ));
        };

        let report = ctx.report.clone();
        let fields = (0..length)
            .map(|_| Option::<SupportedField>::decode(d, ctx))
            .filter_map(Result::transpose);

        Self::from_fields(fields, &report)
    }
}

/// Implements [`serde::de::Visitor`], so that [`Metadata`] can be
/// deserialized by [`serde::Deserializer::deserialize_map`].
struct MetadataDeserializeVisitor;

impl<'de> serde::de::Visitor<'de> for MetadataDeserializeVisitor {
    type Value = Vec<SupportedField>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Catalyst Signed Document metadata key-value pairs")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut d: A) -> Result<Self::Value, A::Error> {
        let mut res = Vec::with_capacity(d.size_hint().unwrap_or(0));
        while let Some(k) = d.next_key::<SupportedLabel>()? {
            let v = d.next_value_seed(k)?;
            res.push(v);
        }
        Ok(res)
    }
}
