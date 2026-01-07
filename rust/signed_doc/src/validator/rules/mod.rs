//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

use anyhow::Context;
use catalyst_signed_doc_spec::{DocSpec, DocSpecs, cddl_definitions::CddlDefinitions};

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
mod revocations;
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
pub(crate) use revocations::RevocationsRule;
pub(crate) use section::SectionRule;
pub(crate) use signature::SignatureRule;
pub(crate) use signature_kid::SignatureKidRule;
pub(crate) use template::TemplateRule;
pub(crate) use ver::VerRule;

use crate::validator::Rules;

/// Creating a rules instances from the provided specs.
fn rules_for_doc(
    cddl_defs: &CddlDefinitions,
    all_docs_specs: &DocSpecs,
    doc_spec: &DocSpec,
) -> anyhow::Result<Rules> {
    Ok(vec![
        Box::new(IdRule),
        Box::new(VerRule),
        Box::new(ContentTypeRule::new(&doc_spec.headers.content_type)?),
        Box::new(ContentEncodingRule::new(
            &doc_spec.headers.content_encoding,
        )?),
        Box::new(TemplateRule::new(
            all_docs_specs,
            &doc_spec.metadata.template,
        )?),
        Box::new(ParametersRule::new(
            all_docs_specs,
            &doc_spec.metadata.parameters,
        )?),
        Box::new(ChainRule::new(
            &doc_spec.metadata.chain,
            &doc_spec.metadata.collaborators,
        )?),
        Box::new(RefRule::new(all_docs_specs, &doc_spec.metadata.doc_ref)?),
        Box::new(ReplyRule::new(all_docs_specs, &doc_spec.metadata.reply)?),
        // TODO:
        // Box::new(RevocationsRule::new(all_docs_specs, &doc_spec.metadata.reply)?),
        Box::new(SectionRule::NotSpecified),
        Box::new(CollaboratorsRule::new(&doc_spec.metadata.collaborators)),
        Box::new(ContentRule::new(cddl_defs, &doc_spec.payload)?),
        Box::new(SignatureKidRule::new(&doc_spec.signers.roles)?),
        Box::new(SignatureRule),
        Box::new(DocumentOwnershipRule::new(
            &doc_spec.signers.update,
            doc_spec,
        )?),
    ])
}

/// Returns an iterator with all defined Catalyst Signed Documents validation rules
/// per corresponding document type based on the `signed_doc.json` file
///
/// # Errors:
///  - `signed_doc.json` filed loading and JSON parsing errors.
///  - `catalyst-signed-doc-spec` crate version doesn't  align with the latest version of
///    the `signed_doc.json`.
pub(crate) fn documents_rules_from_spec()
-> anyhow::Result<impl Iterator<Item = (crate::DocType, Rules)>> {
    let spec = catalyst_signed_doc_spec::CatalystSignedDocSpec::load_signed_doc_spec()?;

    let mut doc_rules = Vec::new();
    for (doc_name, doc_spec) in spec.docs.iter() {
        if doc_spec.draft {
            continue;
        }

        let rules = rules_for_doc(&spec.cddl_definitions, &spec.docs, doc_spec)
            .context(format!("Fail to initializing document '{doc_name}'"))?;
        let doc_type = doc_spec.doc_type.parse()?;

        doc_rules.push((doc_type, rules));
    }

    Ok(doc_rules.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_documents_rules_test() {
        for (doc_type, rules) in documents_rules_from_spec()
            .map_err(|e| format!("{e:#}"))
            .unwrap()
        {
            println!("{doc_type}: {rules:?}");
        }
    }
}
