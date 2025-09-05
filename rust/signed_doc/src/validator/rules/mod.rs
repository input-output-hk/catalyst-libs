//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use futures::FutureExt;

use crate::{
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
    CatalystSignedDocument,
};

mod content;
mod content_encoding;
mod content_type;
mod doc_ref;
mod id;
mod original_author;
mod parameters;
mod reply;
mod section;
mod signature;
mod signature_kid;
mod template;
mod utils;
mod ver;

pub(crate) use content::{ContentRule, ContentSchema};
pub(crate) use content_encoding::ContentEncodingRule;
pub(crate) use content_type::ContentTypeRule;
pub(crate) use doc_ref::RefRule;
pub(crate) use id::IdRule;
pub(crate) use original_author::OriginalAuthorRule;
pub(crate) use parameters::ParametersRule;
pub(crate) use reply::ReplyRule;
pub(crate) use section::SectionRule;
pub(crate) use signature::SignatureRule;
pub(crate) use signature_kid::SignatureKidRule;
pub(crate) use template::TemplateRule;
pub(crate) use ver::VerRule;

/// Struct represented a full collection of rules for all fields
#[derive(Debug)]
pub(crate) struct Rules {
    /// 'id' field validation rule
    pub(crate) id: IdRule,
    /// 'ver' field validation rule
    pub(crate) ver: VerRule,
    /// 'content-type' field validation rule
    pub(crate) content_type: ContentTypeRule,
    /// 'content-encoding' field validation rule
    pub(crate) content_encoding: ContentEncodingRule,
    /// 'template' field validation rule
    pub(crate) template: TemplateRule,
    /// 'ref' field validation rule
    pub(crate) doc_ref: RefRule,
    /// 'reply' field validation rule
    pub(crate) reply: ReplyRule,
    /// 'section' field validation rule
    pub(crate) section: SectionRule,
    /// 'parameters' field validation rule
    pub(crate) parameters: ParametersRule,
    /// document's content validation rule
    pub(crate) content: ContentRule,
    /// `kid` field validation rule
    pub(crate) kid: SignatureKidRule,
    /// document's signatures validation rule
    pub(crate) signature: SignatureRule,
    /// Original Author validation rule.
    pub(crate) original_author: OriginalAuthorRule,
}

impl Rules {
    /// All field validation rules check
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider + VerifyingKeyProvider,
    {
        let rules = [
            self.id.check(doc, provider).boxed(),
            self.ver.check(doc, provider).boxed(),
            self.content_type.check(doc).boxed(),
            self.content_encoding.check(doc).boxed(),
            self.template.check(doc, provider).boxed(),
            self.doc_ref.check(doc, provider).boxed(),
            self.reply.check(doc, provider).boxed(),
            self.section.check(doc).boxed(),
            self.parameters.check(doc, provider).boxed(),
            self.content.check(doc).boxed(),
            self.kid.check(doc).boxed(),
            self.signature.check(doc, provider).boxed(),
            self.original_author.check(doc, provider).boxed(),
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
