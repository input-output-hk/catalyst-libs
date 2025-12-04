//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use std::fmt::Debug;

use anyhow::Context;
use catalyst_signed_doc_spec::{DocSpec, DocSpecs, cddl_definitions::CddlDefinitions};
use futures::{FutureExt, future::BoxFuture};

use crate::{CatalystSignedDocument, providers::CatalystProvider};

mod chain;
mod collaborators;
mod content;
mod content_encoding;
mod content_type;
mod doc_ref;
mod id;
mod ownership;
mod parameters;
mod reply;
mod section;
mod signature;
mod signature_kid;
mod template;
mod utils;
mod ver;

pub(crate) use chain::ChainRule;
pub(crate) use collaborators::CollaboratorsRule;
pub(crate) use content::ContentRule;
pub(crate) use content_encoding::ContentEncodingRule;
pub(crate) use content_type::ContentTypeRule;
pub(crate) use doc_ref::RefRule;
pub(crate) use id::IdRule;
pub(crate) use ownership::DocumentOwnershipRule;
pub(crate) use parameters::ParametersRule;
pub(crate) use reply::ReplyRule;
pub(crate) use section::SectionRule;
pub(crate) use signature::SignatureRule;
pub(crate) use signature_kid::SignatureKidRule;
pub(crate) use template::TemplateRule;
pub(crate) use ver::VerRule;

/// `CatalystSignedDocument` check trait
pub trait CatalystSignedDocumentCheck: Send + Sync + Debug {
    /// Validates `CatalystSignedDocument`, return `false` if the provided
    /// `CatalystSignedDocument` violates some validation rules with properly filling the
    /// problem report.
    fn check<'a>(
        &'a self,
        doc: &'a CatalystSignedDocument,
        provider: &'a dyn CatalystProvider,
    ) -> BoxFuture<'a, anyhow::Result<bool>>;
}

/// Struct represented a full collection of rules for all fields
pub(crate) struct Rules {
    /// 'id' field validation rule
    id: IdRule,
    /// 'ver' field validation rule
    ver: VerRule,
    /// 'content-type' field validation rule
    content_type: ContentTypeRule,
    /// 'content-encoding' field validation rule
    content_encoding: ContentEncodingRule,
    /// 'template' field validation rule
    template: TemplateRule,
    /// 'ref' field validation rule
    doc_ref: RefRule,
    /// 'reply' field validation rule
    reply: ReplyRule,
    /// 'section' field validation rule
    section: SectionRule,
    /// 'parameters' field validation rule
    parameters: ParametersRule,
    /// 'chain' field validation rule
    chain: ChainRule,
    /// 'collaborators' field validation rule
    collaborators: CollaboratorsRule,
    /// document's content validation rule
    content: ContentRule,
    /// `kid` field validation rule
    kid: SignatureKidRule,
    /// document's signatures validation rule
    signature: SignatureRule,
    /// Original Author validation rule.
    ownership: DocumentOwnershipRule,
}

impl Rules {
    /// All field validation rules check
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn CatalystProvider,
    ) -> anyhow::Result<bool> {
        let rules = [
            self.id.check(doc, provider),
            self.ver.check_inner(doc, provider).boxed(),
            self.content_type.check(doc, provider),
            self.content_encoding.check_inner(doc).boxed(),
            self.template.check_inner(doc, provider).boxed(),
            self.doc_ref.check(doc, provider).boxed(),
            self.reply.check(doc, provider).boxed(),
            self.section.check_inner(doc).boxed(),
            self.parameters.check(doc, provider).boxed(),
            self.chain.check(doc, provider).boxed(),
            self.collaborators.check(doc, provider).boxed(),
            self.content.check(doc, provider).boxed(),
            self.kid.check_inner(doc).boxed(),
            self.signature.check_inner(doc, provider).boxed(),
            self.ownership.check(doc, provider).boxed(),
        ];

        let res = futures::future::join_all(rules)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .iter()
            .all(|res| *res);

        Ok(res)
    }

    /// Creating a `Rules` instance from the provided specs.
    fn new(
        cddl_defs: &CddlDefinitions,
        all_docs_specs: &DocSpecs,
        doc_spec: &DocSpec,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: IdRule,
            ver: VerRule,
            content_type: ContentTypeRule::new(&doc_spec.headers.content_type)?,
            content_encoding: ContentEncodingRule::new(&doc_spec.headers.content_encoding)?,
            template: TemplateRule::new(all_docs_specs, &doc_spec.metadata.template)?,
            parameters: ParametersRule::new(all_docs_specs, &doc_spec.metadata.parameters)?,
            chain: ChainRule::new(&doc_spec.metadata.chain, &doc_spec.metadata.collaborators)?,
            doc_ref: RefRule::new(all_docs_specs, &doc_spec.metadata.doc_ref)?,
            reply: ReplyRule::new(all_docs_specs, &doc_spec.metadata.reply)?,
            section: SectionRule::NotSpecified,
            collaborators: CollaboratorsRule::new(&doc_spec.metadata.collaborators),
            content: ContentRule::new(cddl_defs, &doc_spec.payload)?,
            kid: SignatureKidRule::new(&doc_spec.signers.roles)?,
            signature: SignatureRule,
            ownership: DocumentOwnershipRule::new(&doc_spec.signers.update, doc_spec)?,
        })
    }

    /// Returns an iterator with all defined Catalyst Signed Documents validation rules
    /// per corresponding document type based on the `signed_doc.json` file
    ///
    /// # Errors:
    ///  - `signed_doc.json` filed loading and JSON parsing errors.
    ///  - `catalyst-signed-doc-spec` crate version doesn't  align with the latest version
    ///    of the `signed_doc.json`.
    pub(crate) fn documents_rules() -> anyhow::Result<impl Iterator<Item = (crate::DocType, Rules)>>
    {
        let spec = catalyst_signed_doc_spec::CatalystSignedDocSpec::load_signed_doc_spec()?;

        let mut doc_rules = Vec::new();
        for (doc_name, doc_spec) in spec.docs.iter() {
            if doc_spec.draft {
                continue;
            }

            let rules = Self::new(&spec.cddl_definitions, &spec.docs, doc_spec)
                .context(format!("Fail to initializing document '{doc_name}'"))?;
            let doc_type = doc_spec.doc_type.parse()?;

            doc_rules.push((doc_type, rules));
        }

        Ok(doc_rules.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_documents_rules_test() {
        for (_doc_type, _rules) in Rules::documents_rules()
            .map_err(|e| format!("{e:#}"))
            .unwrap()
        {
            // println!("{doc_type}: {rules:?}");
        }
    }
}
