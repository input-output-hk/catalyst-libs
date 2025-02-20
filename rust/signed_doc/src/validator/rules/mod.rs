//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use futures::FutureExt;

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
    /// 'ref' field validation rule
    pub(crate) doc_ref: RefRule,
    /// 'template' field validation rule
    pub(crate) template: TemplateRule,
    /// 'reply' field validation rule
    pub(crate) reply: ReplyRule,
    /// 'section' field validation rule
    pub(crate) section: SectionRule,
    /// 'category' field validation rule
    pub(crate) category: CategoryRule,
}

impl Rules {
    /// All field validation rules check
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: 'static + CatalystSignedDocumentProvider {
        let rules = [
            self.content_type.check(doc).boxed(),
            self.content_encoding.check(doc).boxed(),
            self.doc_ref.check(doc, provider).boxed(),
            self.template.check(doc, provider).boxed(),
            self.reply.check(doc, provider).boxed(),
            self.section.check(doc).boxed(),
            self.category.check(doc, provider).boxed(),
        ];

        let res = futures::future::join_all(rules)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .iter()
            .all(|res| *res);
        Ok(res)
    }
}
