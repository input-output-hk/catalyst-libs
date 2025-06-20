//! Catalyst Signed Document unified metadata field.

use std::fmt::{self, Display};
#[cfg(test)]
use std::{cmp, convert::Infallible};

use catalyst_types::uuid::UuidV7;
use serde::Deserialize;
use strum::{EnumDiscriminants, EnumTryAs, IntoDiscriminant as _};

use crate::{
    metadata::custom_transient_decode_error, ContentEncoding, ContentType, DocType, DocumentRef,
    Section,
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
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            &Label::U8(u) => e.u8(u),
            Label::Str(s) => e.str(s),
        }?
        .ok()
    }
}

impl<'a, C> minicbor::Decode<'a, C> for Label<'a> {
    fn decode(d: &mut minicbor::Decoder<'a>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U8 => d.u8().map(Self::U8),
            minicbor::data::Type::String => d.str().map(Self::Str),
            _ => Err(minicbor::decode::Error::message(
                "Datatype is neither 8bit unsigned integer nor text",
            )
            .at(d.position())),
        }
    }
}

#[cfg(test)]
impl Label<'_> {
    /// Compare by [RFC 8949 section 4.2.1] specification.
    ///
    ///  [RFC 8949 section 4.2.1]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.1
    fn rfc8949_cmp(
        &self, other: &Self,
    ) -> Result<cmp::Ordering, minicbor::encode::Error<Infallible>> {
        let lhs = minicbor::to_vec(self)?;
        let rhs = minicbor::to_vec(other)?;
        let ord = lhs.len().cmp(&rhs.len()).then_with(|| lhs.cmp(&rhs));
        Ok(ord)
    }
}

impl Display for Label<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    derive(Ord, PartialOrd, serde::Deserialize),
    serde(rename_all = "kebab-case"),
    cfg_attr(test, derive(strum::VariantArray))
)]
#[non_exhaustive]
#[repr(usize)]
pub enum SupportedField {
    /// `content-type` field. In COSE it's represented as the signed integer `3` (see [RFC
    /// 8949 section 3.1]).
    ///
    /// [RFC 8949 section 3.1]: https://datatracker.ietf.org/doc/html/rfc8152#section-3.1
    ContentType(ContentType) = 0,
    /// `id` field.
    Id(UuidV7) = 1,
    /// `ref` field.
    Ref(DocumentRef) = 2,
    /// `ver` field.
    Ver(UuidV7) = 3,
    /// `type` field.
    Type(DocType) = 4,
    /// `reply` field.
    Reply(DocumentRef) = 5,
    /// `collabs` field.
    Collabs(Vec<String>) = 7,
    /// `section` field.
    Section(Section) = 8,
    /// `template` field.
    Template(DocumentRef) = 9,
    /// `parameters` field.
    Parameters(DocumentRef) = 10,
    /// `Content-Encoding` field.
    ContentEncoding(ContentEncoding) = 11,
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
            Label::Str("reply") => Some(Self::Reply),
            Label::Str("collabs") => Some(Self::Collabs),
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
            Self::Reply => Label::Str("reply"),
            Self::Collabs => Label::Str("collabs"),
            Self::Section => Label::Str("section"),
            Self::Template => Label::Str("template"),
            Self::Parameters => Label::Str("parameters"),
            Self::ContentEncoding => Label::Str("content-encoding"),
        }
    }
}

impl Display for SupportedLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_cose(), f)
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for SupportedLabel {
    type Value = SupportedField;

    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        match self {
            SupportedLabel::ContentType => {
                Deserialize::deserialize(d).map(SupportedField::ContentType)
            },
            SupportedLabel::Id => Deserialize::deserialize(d).map(SupportedField::Id),
            SupportedLabel::Ref => Deserialize::deserialize(d).map(SupportedField::Ref),
            SupportedLabel::Ver => Deserialize::deserialize(d).map(SupportedField::Ver),
            SupportedLabel::Type => Deserialize::deserialize(d).map(SupportedField::Type),
            SupportedLabel::Reply => Deserialize::deserialize(d).map(SupportedField::Reply),
            SupportedLabel::Collabs => Deserialize::deserialize(d).map(SupportedField::Collabs),
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

impl minicbor::Decode<'_, crate::decode_context::DecodeContext<'_>> for SupportedField {
    #[allow(clippy::todo, reason = "Not migrated to `minicbor` yet.")]
    fn decode(
        d: &mut minicbor::Decoder<'_>, ctx: &mut crate::decode_context::DecodeContext<'_>,
    ) -> Result<Self, minicbor::decode::Error> {
        const REPORT_CONTEXT: &str = "Metadata field decoding";

        let label_pos = d.position();
        let label = Label::decode(d, &mut ())?;
        let Some(key) = SupportedLabel::from_cose(label) else {
            let value_start = d.position();
            d.skip()?;
            let value_end = d.position();
            // Since the high level type isn't know, the value CBOR is tokenized and reported as
            // such.
            let value = minicbor::decode::Tokenizer::new(
                d.input().get(value_start..value_end).unwrap_or_default(),
            )
            .to_string();
            ctx.report
                .unknown_field(&label.to_string(), &value, REPORT_CONTEXT);
            return Err(custom_transient_decode_error(
                "Not a supported key",
                Some(label_pos),
            ));
        };

        let field = match key {
            SupportedLabel::ContentType => todo!(),
            SupportedLabel::Id => d
                .decode_with(&mut catalyst_types::uuid::CborContext::Tagged)
                .map(Self::Id),
            SupportedLabel::Ref => todo!(),
            SupportedLabel::Ver => d
                .decode_with(&mut catalyst_types::uuid::CborContext::Tagged)
                .map(Self::Ver),
            SupportedLabel::Type => d.decode_with(ctx).map(Self::Type),
            SupportedLabel::Reply => todo!(),
            SupportedLabel::Collabs => todo!(),
            SupportedLabel::Section => todo!(),
            SupportedLabel::Template => todo!(),
            SupportedLabel::Parameters => todo!(),
            SupportedLabel::ContentEncoding => todo!(),
        }?;

        Ok(field)
    }
}

impl minicbor::Encode<()> for SupportedField {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
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
            SupportedField::Collabs(collabs) => {
                if !collabs.is_empty() {
                    e.array(
                        collabs
                            .len()
                            .try_into()
                            .map_err(minicbor::encode::Error::message)?,
                    )?;
                    for c in collabs {
                        e.str(c)?;
                    }
                }
                Ok(())
            },
            SupportedField::Section(section) => section.encode(e, ctx),
            SupportedField::ContentEncoding(content_encoding) => content_encoding.encode(e, ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::VariantArray as _;

    use super::*;

    /// Checks that [`Label::rfc8949_cmp`] ordering is compliant with the RFC.
    #[test]
    fn label_rfc8949_cmp() {
        assert_eq!(
            Label::Str("a").rfc8949_cmp(&Label::Str("a")).unwrap(),
            cmp::Ordering::Equal
        );
        assert_eq!(
            Label::Str("a").rfc8949_cmp(&Label::Str("aa")).unwrap(),
            cmp::Ordering::Less
        );
        assert_eq!(
            Label::Str("a").rfc8949_cmp(&Label::Str("b")).unwrap(),
            cmp::Ordering::Less
        );
        assert_eq!(
            Label::Str("aa").rfc8949_cmp(&Label::Str("b")).unwrap(),
            cmp::Ordering::Greater
        );
        assert_eq!(
            Label::U8(3).rfc8949_cmp(&Label::Str("id")).unwrap(),
            cmp::Ordering::Less
        );
    }

    /// Checks that [`SupportedLabel`] enum integer values correspond to
    /// [`Label::rfc8949_cmp`] ordering.
    #[test]
    fn supported_label_rfc8949_ord() {
        let mut enum_ord = SupportedLabel::VARIANTS.to_vec();
        // Sorting by the Rust enum representation.
        enum_ord.sort_unstable();

        let mut cose_ord = SupportedLabel::VARIANTS.to_vec();
        // Sorting by the corresponding COSE labels.
        cose_ord.sort_unstable_by(|lhs, rhs| lhs.to_cose().rfc8949_cmp(&rhs.to_cose()).unwrap());

        assert_eq!(enum_ord, cose_ord);
    }
}
