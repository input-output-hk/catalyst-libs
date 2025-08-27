//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use futures::FutureExt;

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

mod content_encoding;
mod content_type;
mod doc_ref;
mod id;
mod parameters;
mod reply;
mod section;
mod signature_kid;
mod template;
mod ver;

pub(crate) use content_encoding::ContentEncodingRule;
pub(crate) use content_type::ContentTypeRule;
pub(crate) use doc_ref::RefRule;
pub(crate) use id::IdRule;
pub(crate) use parameters::ParametersRule;
pub(crate) use reply::ReplyRule;
pub(crate) use section::SectionRule;
pub(crate) use signature_kid::SignatureKidRule;
pub(crate) use template::{ContentRule, ContentSchema};
pub(crate) use ver::VerRule;

/// Struct represented a full collection of rules for all fields
pub(crate) struct Rules {
    /// 'id' field validation rule
    pub(crate) id: IdRule,
    /// 'ver' field validation rule
    pub(crate) ver: VerRule,
    /// 'content-type' field validation rule
    pub(crate) content_type: ContentTypeRule,
    /// 'content-encoding' field validation rule
    pub(crate) content_encoding: ContentEncodingRule,
    /// 'ref' field validation rule
    pub(crate) doc_ref: RefRule,
    /// document's content validation rule
    pub(crate) content: ContentRule,
    /// 'reply' field validation rule
    pub(crate) reply: ReplyRule,
    /// 'section' field validation rule
    pub(crate) section: SectionRule,
    /// 'parameters' field validation rule
    pub(crate) parameters: ParametersRule,
    /// `kid` field validation rule
    pub(crate) kid: SignatureKidRule,
}

impl Rules {
    /// All field validation rules check
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        let rules = [
            self.id.check(doc, provider).boxed(),
            self.ver.check(doc, provider).boxed(),
            self.content_type.check(doc).boxed(),
            self.content_encoding.check(doc).boxed(),
            self.content.check(doc, provider).boxed(),
            self.doc_ref.check(doc, provider).boxed(),
            self.reply.check(doc, provider).boxed(),
            self.section.check(doc).boxed(),
            self.parameters.check(doc, provider).boxed(),
            self.kid.check(doc).boxed(),
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
