//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use futures::try_join;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

mod category;
mod content_encoding;
mod content_type;
mod doc_ref;
mod reply;
mod section;
mod template;

pub(crate) use category::CategoryRule;
pub(crate) use content_encoding::ContentEncodingRule;
pub(crate) use content_type::ContentTypeRule;
pub(crate) use doc_ref::RefRule;
pub(crate) use reply::ReplyRule;
pub(crate) use section::SectionRule;
pub(crate) use template::TemplateRule;

/// Struct represented a full collection of rules for all fields
pub(crate) struct Rules {
    /// 'content-type' field validation rule
    pub(crate) content_type: ContentTypeRule,
    /// 'content-encoding' field validation rule
    pub(crate) content_encoding: ContentEncodingRule,
}

impl Rules {
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, _provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        if let (true, true) = try_join!(
            self.content_type.check(doc),
            self.content_encoding.check(doc)
        )? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
