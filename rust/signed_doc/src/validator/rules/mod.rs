//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

mod content_encoding;
mod content_type;

pub(crate) use content_encoding::ContentEncodingRule;
pub(crate) use content_type::ContentTypeRule;
