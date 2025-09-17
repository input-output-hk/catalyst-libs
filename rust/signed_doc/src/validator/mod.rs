//! Catalyst Signed Documents validation logic

pub(crate) mod json_schema;
pub(crate) mod rules;

use std::{collections::HashMap, sync::LazyLock};

use catalyst_types::catalyst_id::role_index::RoleId;
use rules::{
    ContentEncodingRule, ContentRule, ContentSchema, ContentTypeRule, IdRule, OriginalAuthorRule,
    ParametersRule, RefRule, ReplyRule, Rules, SectionRule, SignatureKidRule, VerRule,
};

use crate::{
    doc_types::{
        BRAND_PARAMETERS, CAMPAIGN_PARAMETERS, CATEGORY_PARAMETERS, PROPOSAL, PROPOSAL_COMMENT,
        PROPOSAL_COMMENT_FORM_TEMPLATE, PROPOSAL_FORM_TEMPLATE, PROPOSAL_SUBMISSION_ACTION,
    },
    metadata::DocType,
    providers::{CatalystIdProvider, CatalystSignedDocumentProvider},
    validator::rules::{CollaboratorsRule, SignatureRule, TemplateRule},
    CatalystSignedDocument, ContentEncoding, ContentType,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<DocType, Rules>> = LazyLock::new(document_rules_init);

/// Proposal
/// Require field: type, id, ver, template, parameters
/// <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal/>
fn proposal_rule() -> Rules {
    // Parameter can be either brand, campaign or category
    let parameters = vec![
        BRAND_PARAMETERS.clone(),
        CAMPAIGN_PARAMETERS.clone(),
        CATEGORY_PARAMETERS.clone(),
    ];
    Rules {
        id: IdRule,
        ver: VerRule,
        content_type: ContentTypeRule::Specified {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule::Specified {
            exp: vec![ContentEncoding::Brotli],
            optional: false,
        },
        template: TemplateRule::Specified {
            allowed_type: PROPOSAL_FORM_TEMPLATE.clone(),
        },
        parameters: ParametersRule::Specified {
            allowed_type: parameters.clone(),
            optional: false,
        },
        doc_ref: RefRule::NotSpecified,
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        collaborators: CollaboratorsRule::NotSpecified,
        content: ContentRule::NotNil,
        kid: SignatureKidRule {
            allowed_roles: vec![RoleId::Proposer],
        },
        signature: SignatureRule { mutlisig: false },
        original_author: OriginalAuthorRule,
    }
}

/// Proposal Comment
/// Require field: type, id, ver, ref, template, parameters
/// <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_comment_template/>
fn proposal_comment_rule() -> Rules {
    // Parameter can be either brand, campaign or category
    let parameters = vec![
        BRAND_PARAMETERS.clone(),
        CAMPAIGN_PARAMETERS.clone(),
        CATEGORY_PARAMETERS.clone(),
    ];
    Rules {
        id: IdRule,
        ver: VerRule,
        content_type: ContentTypeRule::Specified {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule::Specified {
            exp: vec![ContentEncoding::Brotli],
            optional: false,
        },
        template: TemplateRule::Specified {
            allowed_type: PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
        },
        doc_ref: RefRule::Specified {
            allowed_type: vec![PROPOSAL.clone()],
            multiple: false,
            optional: false,
        },
        reply: ReplyRule::Specified {
            allowed_type: PROPOSAL_COMMENT.clone(),
            optional: true,
        },
        section: SectionRule::NotSpecified,
        parameters: ParametersRule::Specified {
            allowed_type: parameters.clone(),
            optional: false,
        },
        collaborators: CollaboratorsRule::NotSpecified,
        content: ContentRule::NotNil,
        kid: SignatureKidRule {
            allowed_roles: vec![RoleId::Role0],
        },
        signature: SignatureRule { mutlisig: false },
        original_author: OriginalAuthorRule,
    }
}

/// Proposal Submission Action
/// Require fields: type, id, ver, ref, parameters
/// <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/docs/proposal_submission_action/>
#[allow(clippy::expect_used)]
fn proposal_submission_action_rule() -> Rules {
    // Parameter can be either brand, campaign or category
    let parameters = vec![
        BRAND_PARAMETERS.clone(),
        CAMPAIGN_PARAMETERS.clone(),
        CATEGORY_PARAMETERS.clone(),
    ];

    let proposal_action_json_schema_content = &serde_json::from_str(include_str!(
        "./../../../../specs/definitions/signed_docs/docs/payload_schemas/proposal_submission_action.schema.json"
    ))
    .expect("Must be a valid json file");

    let proposal_action_json_schema =
        json_schema::JsonSchema::try_from(proposal_action_json_schema_content)
            .expect("Must be a valid json scheme file");

    Rules {
        id: IdRule,
        ver: VerRule,
        content_type: ContentTypeRule::Specified {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule::Specified {
            exp: vec![ContentEncoding::Brotli],
            optional: false,
        },
        template: TemplateRule::NotSpecified,
        parameters: ParametersRule::Specified {
            allowed_type: parameters,
            optional: false,
        },
        doc_ref: RefRule::Specified {
            allowed_type: vec![PROPOSAL.clone()],
            multiple: false,
            optional: false,
        },
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        collaborators: CollaboratorsRule::NotSpecified,
        content: ContentRule::StaticSchema(ContentSchema::Json(proposal_action_json_schema)),
        kid: SignatureKidRule {
            allowed_roles: vec![RoleId::Proposer],
        },
        signature: SignatureRule { mutlisig: false },
        original_author: OriginalAuthorRule,
    }
}

/// `DOCUMENT_RULES` initialization function
#[allow(clippy::expect_used)]
fn document_rules_init() -> HashMap<DocType, Rules> {
    let mut document_rules_map: HashMap<DocType, Rules> = Rules::documents_rules()
        .expect("cannot fail to initialize validation rules")
        .collect();

    // TODO: remove this redefinitions of the validation rules after
    // `catalyst_signed_documents_rules!` macro would be fully finished
    document_rules_map.insert(PROPOSAL.clone(), proposal_rule());
    document_rules_map.insert(PROPOSAL_COMMENT.clone(), proposal_comment_rule());
    document_rules_map.insert(
        PROPOSAL_SUBMISSION_ACTION.clone(),
        proposal_submission_action_rule(),
    );

    document_rules_map
}

/// A comprehensive document type based validation of the `CatalystSignedDocument`.
/// Includes time based validation of the `id` and `ver` fields based on the provided
/// `future_threshold` and `past_threshold` threshold values (in seconds).
/// Return true if it is valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument,
    provider: &Provider,
) -> anyhow::Result<bool>
where
    Provider: CatalystSignedDocumentProvider + CatalystIdProvider,
{
    let Ok(doc_type) = doc.doc_type() else {
        doc.report().missing_field(
            "type",
            "Can't get a document type during the validation process",
        );
        return Ok(false);
    };

    let Some(rules) = DOCUMENT_RULES.get(doc_type) else {
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

#[cfg(test)]
mod tests {
    use crate::validator::document_rules_init;

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
