//! A list of validation rules for all metadata fields
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/meta/>

mod category;
mod content_encoding;
mod content_type;
mod doc_ref;
mod reply;
mod template;

use category::CategoryRule;
use content_encoding::ContentEncodingRule;
use content_type::ContentTypeRule;
use doc_ref::RefRule;
use reply::ReplyRule;
use template::TemplateRule;

use super::ValidationRule;
use crate::{
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE,
        PROPOSAL_TEMPLATE_UUID_TYPE,
    },
    metadata::{ContentEncoding, ContentType},
    providers::CatalystSignedDocumentProvider,
    validator::boxed_rule,
};

/// A list of rules for the Proposal Document
pub(crate) fn proposal_document_rules<Provider>() -> Vec<Box<dyn ValidationRule<Provider>>>
where Provider: 'static + CatalystSignedDocumentProvider {
    vec![
        boxed_rule(ContentTypeRule {
            exp: ContentType::Json,
        }),
        boxed_rule(ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        }),
        boxed_rule(TemplateRule {
            template_type: PROPOSAL_TEMPLATE_UUID_TYPE,
        }),
        boxed_rule(CategoryRule { optional: true }),
    ]
}

/// A list of rules for the Comment Document
pub(crate) fn comment_document_rules<Provider>() -> Vec<Box<dyn ValidationRule<Provider>>>
where Provider: 'static + CatalystSignedDocumentProvider {
    vec![
        boxed_rule(ContentTypeRule {
            exp: ContentType::Json,
        }),
        boxed_rule(ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        }),
        boxed_rule(TemplateRule {
            template_type: COMMENT_TEMPLATE_UUID_TYPE,
        }),
        boxed_rule(RefRule {
            ref_type: PROPOSAL_DOCUMENT_UUID_TYPE,
            optional: false,
        }),
        boxed_rule(ReplyRule {
            reply_type: COMMENT_DOCUMENT_UUID_TYPE,
            optional: true,
        }),
    ]
}
