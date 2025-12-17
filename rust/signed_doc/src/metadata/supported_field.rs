//! Catalyst Signed Document unified metadata field.

use std::fmt;

use catalyst_types::uuid::UuidV7;
use serde::Deserialize;
use strum::{EnumDiscriminants, EnumTryAs, IntoDiscriminant as _};

use crate::{
    Chain, ContentEncoding, ContentType, DocType, DocumentRefs, Section,
    metadata::{collaborators::Collaborators, revocations::Revocations},
};

/// COSE label. May be either a signed integer or a string.
#[derive(Copy, Clone, Eq, PartialEq)]
enum Label<'a> {
    /// Integer label.
    ///
    /// Note that COSE isn't strictly limited to 8 bits for a label, but in practice it
    /// fits.
    ///
    /// If for any reason wider bounds would be necessary,
    /// then additional variants could be added to the [`Label`].
    U8(u8),
    /// Text label.
    Str(&'a str),
}

impl minicbor::Encode<()> for Label<'_> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            &Label::U8(u) => e.u8(u),
            Label::Str(s) => e.str(s),
        }?
        .ok()
    }
}

impl<'a, C> minicbor::Decode<'a, C> for Label<'a> {
    fn decode(
        d: &mut minicbor::Decoder<'a>,
        _: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U8 => d.u8().map(Self::U8),
            minicbor::data::Type::String => d.str().map(Self::Str),
            _ => {
                Err(minicbor::decode::Error::message(
                    "Datatype is neither 8bit unsigned integer nor text",
                )
                .at(d.position()))
            },
        }
    }
}

impl fmt::Display for Label<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Label::U8(u) => write!(f, "{u}"),
            Label::Str(s) => f.write_str(s),
        }
    }
}

/// Catalyst Signed Document metadata field.
/// Fields are assigned discriminants based on deterministic ordering (see [RFC 8949
/// section 4.2.1]).
///
/// Note that [`PartialEq`] implementation compares both keys and values.
///
/// [RFC 8949 section 4.2.1]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.1
#[derive(Clone, Debug, PartialEq, EnumDiscriminants, EnumTryAs)]
#[strum_discriminants(
    name(SupportedLabel),
    derive(serde::Deserialize, Hash),
    serde(rename_all = "kebab-case"),
    cfg_attr(test, derive(strum::VariantArray))
)]
#[non_exhaustive]
#[repr(usize)]
pub(crate) enum SupportedField {
    /// `content-type` field. In COSE it's represented as the signed integer `3` (see [RFC
    /// 8949 section 3.1]).
    ///
    /// [RFC 8949 section 3.1]: https://datatracker.ietf.org/doc/html/rfc8152#section-3.1
    ContentType(ContentType) = 0,
    /// `id` field.
    Id(UuidV7) = 1,
    /// `ref` field.
    Ref(DocumentRefs) = 2,
    /// `ver` field.
    Ver(UuidV7) = 3,
    /// `type` field.
    Type(DocType) = 4,
    /// `chain` field.
    Chain(Chain) = 5,
    /// `reply` field.
    Reply(DocumentRefs) = 6,
    /// `section` field.
    Section(Section) = 7,
    /// `template` field.
    Template(DocumentRefs) = 8,
    /// `parameters` field.
    Parameters(DocumentRefs) = 9,
    /// `revocations` field.
    Revocations(Revocations) = 10,
    /// `collaborators` field.
    Collaborators(Collaborators) = 11,
    /// `Content-Encoding` field.
    ContentEncoding(ContentEncoding) = 12,
}

impl SupportedLabel {
    /// Try to convert from an arbitrary COSE [`Label`].
    /// This doesn't allow any aliases.
    fn from_cose(label: Label<'_>) -> Option<Self> {
        match label {
            Label::U8(3) => Some(Self::ContentType),
            Label::Str("id") => Some(Self::Id),
            Label::Str("ref") => Some(Self::Ref),
            Label::Str("ver") => Some(Self::Ver),
            Label::Str("type") => Some(Self::Type),
            Label::Str("chain") => Some(Self::Chain),
            Label::Str("reply") => Some(Self::Reply),
            Label::Str("revocations") => Some(Self::Revocations),
            Label::Str("collaborators") => Some(Self::Collaborators),
            Label::Str("section") => Some(Self::Section),
            Label::Str("template") => Some(Self::Template),
            Label::Str("parameters" | "brand_id" | "campaign_id" | "category_id") => {
                Some(Self::Parameters)
            },
            Label::Str(s) if s.eq_ignore_ascii_case("content-encoding") => {
                Some(Self::ContentEncoding)
            },
            _ => None,
        }
    }

    /// Convert to the corresponding COSE [`Label`].
    fn to_cose(self) -> Label<'static> {
        match self {
            Self::ContentType => Label::U8(3),
            Self::Id => Label::Str("id"),
            Self::Ref => Label::Str("ref"),
            Self::Ver => Label::Str("ver"),
            Self::Type => Label::Str("type"),
            Self::Chain => Label::Str("chain"),
            Self::Reply => Label::Str("reply"),
            Self::Revocations => Label::Str("revocations"),
            Self::Collaborators => Label::Str("collaborators"),
            Self::Section => Label::Str("section"),
            Self::Template => Label::Str("template"),
            Self::Parameters => Label::Str("parameters"),
            Self::ContentEncoding => Label::Str("content-encoding"),
        }
    }
}

impl fmt::Display for SupportedLabel {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::ContentType => write!(f, "content-type"),
            v => v.to_cose().fmt(f),
        }
    }
}

impl serde::ser::Serialize for SupportedField {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Id(v) | Self::Ver(v) => v.serialize(serializer),
            Self::Type(v) => v.serialize(serializer),
            Self::Chain(v) => v.serialize(serializer),
            Self::ContentType(v) => v.serialize(serializer),
            Self::ContentEncoding(v) => v.serialize(serializer),
            Self::Ref(v) | Self::Reply(v) | Self::Template(v) | Self::Parameters(v) => {
                v.serialize(serializer)
            },
            Self::Revocations(v) => v.serialize(serializer),
            Self::Collaborators(v) => v.serialize(serializer),
            Self::Section(v) => v.serialize(serializer),
        }
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for SupportedLabel {
    type Value = SupportedField;

    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<Self::Value, D::Error> {
        match self {
            SupportedLabel::ContentType => {
                Deserialize::deserialize(d).map(SupportedField::ContentType)
            },
            SupportedLabel::Id => Deserialize::deserialize(d).map(SupportedField::Id),
            SupportedLabel::Ref => Deserialize::deserialize(d).map(SupportedField::Ref),
            SupportedLabel::Ver => Deserialize::deserialize(d).map(SupportedField::Ver),
            SupportedLabel::Type => Deserialize::deserialize(d).map(SupportedField::Type),
            SupportedLabel::Chain => Deserialize::deserialize(d).map(SupportedField::Chain),
            SupportedLabel::Reply => Deserialize::deserialize(d).map(SupportedField::Reply),
            SupportedLabel::Revocations => {
                Deserialize::deserialize(d).map(SupportedField::Revocations)
            },
            SupportedLabel::Collaborators => {
                Deserialize::deserialize(d).map(SupportedField::Collaborators)
            },
            SupportedLabel::Section => Deserialize::deserialize(d).map(SupportedField::Section),
            SupportedLabel::Template => Deserialize::deserialize(d).map(SupportedField::Template),
            SupportedLabel::Parameters => {
                Deserialize::deserialize(d).map(SupportedField::Parameters)
            },
            SupportedLabel::ContentEncoding => {
                Deserialize::deserialize(d).map(SupportedField::ContentEncoding)
            },
        }
    }
}

impl minicbor::Decode<'_, crate::decode_context::DecodeContext> for Option<SupportedField> {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        ctx: &mut crate::decode_context::DecodeContext,
    ) -> Result<Self, minicbor::decode::Error> {
        const REPORT_CONTEXT: &str = "Metadata field decoding";

        let Ok(key) = d
            .decode::<SupportedLabel>()
            .inspect_err(|e| ctx.report().other(e.to_string().as_str(), REPORT_CONTEXT))
        else {
            return Ok(None);
        };

        let cbor_bytes = cbork_utils::decode_helper::decode_any(d, REPORT_CONTEXT)?;
        let mut d = minicbor::Decoder::new(cbor_bytes);

        let field = match key {
            SupportedLabel::ContentType => d.decode().map(SupportedField::ContentType),
            SupportedLabel::Id => {
                d.decode_with(&mut catalyst_types::uuid::CborContext::Tagged)
                    .map(SupportedField::Id)
            },
            SupportedLabel::Ref => {
                d.decode_with(&mut ctx.policy().clone())
                    .map(SupportedField::Ref)
            },
            SupportedLabel::Ver => {
                d.decode_with(&mut catalyst_types::uuid::CborContext::Tagged)
                    .map(SupportedField::Ver)
            },
            SupportedLabel::Type => d.decode().map(SupportedField::Type),
            SupportedLabel::Chain => d.decode().map(SupportedField::Chain),
            SupportedLabel::Reply => {
                d.decode_with(&mut ctx.policy().clone())
                    .map(SupportedField::Reply)
            },
            SupportedLabel::Revocations => d.decode().map(SupportedField::Revocations),
            SupportedLabel::Collaborators => d.decode().map(SupportedField::Collaborators),
            SupportedLabel::Section => d.decode().map(SupportedField::Section),
            SupportedLabel::Template => {
                d.decode_with(&mut ctx.policy().clone())
                    .map(SupportedField::Template)
            },
            SupportedLabel::Parameters => {
                d.decode_with(&mut ctx.policy().clone())
                    .map(SupportedField::Parameters)
            },
            SupportedLabel::ContentEncoding => d.decode().map(SupportedField::ContentEncoding),
        }
        .inspect_err(|e| {
            ctx.report().invalid_value(
                &format!("CBOR COSE protected header {key}"),
                &hex::encode(cbor_bytes),
                &format!("{e}"),
                REPORT_CONTEXT,
            );
        })
        .ok();

        Ok(field)
    }
}

impl<C> minicbor::Decode<'_, C> for SupportedLabel {
    fn decode(
        d: &mut minicbor::Decoder<'_>,
        _ctx: &mut C,
    ) -> Result<Self, minicbor::decode::Error> {
        let label = d.decode()?;
        Self::from_cose(label).ok_or(minicbor::decode::Error::message(format!(
            "Unsupported key {label}"
        )))
    }
}

impl minicbor::Encode<()> for SupportedField {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let key = self.discriminant().to_cose();
        e.encode(key)?;

        match self {
            SupportedField::ContentType(content_type) => content_type.encode(e, ctx),
            SupportedField::Id(uuid_v7) | SupportedField::Ver(uuid_v7) => {
                uuid_v7.encode(e, &mut catalyst_types::uuid::CborContext::Tagged)
            },
            SupportedField::Ref(document_ref)
            | SupportedField::Reply(document_ref)
            | SupportedField::Template(document_ref)
            | SupportedField::Parameters(document_ref) => document_ref.encode(e, ctx),
            SupportedField::Type(doc_type) => doc_type.encode(e, ctx),
            SupportedField::Chain(chain) => chain.encode(e, ctx),
            SupportedField::Revocations(revocations) => revocations.encode(e, ctx),
            SupportedField::Collaborators(collaborators) => collaborators.encode(e, ctx),
            SupportedField::Section(section) => section.encode(e, ctx),
            SupportedField::ContentEncoding(content_encoding) => content_encoding.encode(e, ctx),
        }
    }
}
