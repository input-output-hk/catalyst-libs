//! Catalyst Signed Documents validation

pub(crate) mod rules;
pub(crate) mod utils;

use std::{collections::HashMap, sync::LazyLock};

use catalyst_types::uuid::Uuid;
use rules::{
    CategoryRule, ContentEncodingRule, ContentTypeRule, RefRule, ReplyRule, Rules, SectionRule,
    TemplateRule,
};

use crate::{
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE,
        PROPOSAL_TEMPLATE_UUID_TYPE,
    },
    providers::CatalystSignedDocumentProvider,
    CatalystSignedDocument, ContentEncoding, ContentType,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<Uuid, Rules>> = LazyLock::new(document_rules_init);

/// `DOCUMENT_RULES` initialization function
fn document_rules_init() -> HashMap<Uuid, Rules> {
    let mut document_rules_map = HashMap::new();

    let proposal_document_rules = Rules {
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        template: TemplateRule::Specified {
            exp_template_type: PROPOSAL_TEMPLATE_UUID_TYPE,
        },
        category: CategoryRule::Specified { optional: false },
        doc_ref: RefRule::NotSpecified,
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
    };
    document_rules_map.insert(PROPOSAL_DOCUMENT_UUID_TYPE, proposal_document_rules);

    let comment_document_rules = Rules {
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        template: TemplateRule::Specified {
            exp_template_type: COMMENT_TEMPLATE_UUID_TYPE,
        },
        doc_ref: RefRule::Specified {
            exp_ref_type: PROPOSAL_DOCUMENT_UUID_TYPE,
            optional: false,
        },
        reply: ReplyRule::Specified {
            exp_reply_type: COMMENT_DOCUMENT_UUID_TYPE,
            optional: true,
        },
        section: SectionRule::Specified { optional: true },
        category: CategoryRule::NotSpecified,
    };
    document_rules_map.insert(COMMENT_DOCUMENT_UUID_TYPE, comment_document_rules);

    document_rules_map
}

/// A comprehensive document type based validation of the `CatalystSignedDocument`.
/// Return true if all signatures are valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument, provider: &Provider,
) -> anyhow::Result<bool>
where Provider: 'static + CatalystSignedDocumentProvider {
    let Ok(doc_type) = doc.doc_type() else {
        doc.report().missing_field(
            "type",
            "Can't get a document type during the validation process",
        );
        return Ok(false);
    };

    let Some(rules) = DOCUMENT_RULES.get(&doc_type.uuid()) else {
        doc.report().invalid_value(
            "`type`",
            &doc.doc_type()?.to_string(),
            "Must be a known document type value",
            "Unsupported document type",
        );
        return Ok(false);
    };
    rules.check(doc, provider).await
}
