//! Catalyst Signed Documents validation logic

pub(crate) mod json_schema;
pub(crate) mod rules;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use anyhow::Context;
use catalyst_types::{catalyst_id::role_index::RoleId, problem_report::ProblemReport};
use rules::{
    ContentEncodingRule, ContentRule, ContentSchema, ContentTypeRule, IdRule, ParametersRule,
    RefRule, ReplyRule, Rules, SectionRule, SignatureKidRule, VerRule,
};

use crate::{
    doc_types::{
        BRAND_PARAMETERS, CAMPAIGN_PARAMETERS, CATEGORY_PARAMETERS, PROPOSAL, PROPOSAL_COMMENT,
        PROPOSAL_COMMENT_FORM_TEMPLATE, PROPOSAL_FORM_TEMPLATE, PROPOSAL_SUBMISSION_ACTION,
    },
    metadata::DocType,
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
    signature::{tbs_data, Signature},
    CatalystSignedDocument, ContentEncoding, ContentType,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<DocType, Arc<Rules>>> = LazyLock::new(document_rules_init);

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
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Templated {
            exp_template_type: PROPOSAL_FORM_TEMPLATE.clone(),
        },
        parameters: ParametersRule::Specified {
            exp_parameters_type: parameters.clone(),
            optional: false,
        },
        doc_ref: RefRule::NotSpecified,
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleId::Proposer],
        },
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
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Templated {
            exp_template_type: PROPOSAL_COMMENT_FORM_TEMPLATE.clone(),
        },
        doc_ref: RefRule::Specified {
            exp_ref_types: vec![PROPOSAL.clone()],
            optional: false,
        },
        reply: ReplyRule::Specified {
            exp_reply_type: PROPOSAL_COMMENT.clone(),
            optional: true,
        },
        section: SectionRule::NotSpecified,
        parameters: ParametersRule::Specified {
            exp_parameters_type: parameters.clone(),
            optional: false,
        },
        kid: SignatureKidRule {
            exp: &[RoleId::Role0],
        },
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
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Static(ContentSchema::Json(proposal_action_json_schema)),
        parameters: ParametersRule::Specified {
            exp_parameters_type: parameters,
            optional: false,
        },
        doc_ref: RefRule::Specified {
            exp_ref_types: vec![PROPOSAL.clone()],
            optional: false,
        },
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleId::Proposer],
        },
    }
}

/// `DOCUMENT_RULES` initialization function
fn document_rules_init() -> HashMap<DocType, Arc<Rules>> {
    let mut document_rules_map = HashMap::new();

    let proposal_rules = Arc::new(proposal_rule());
    let comment_rules = Arc::new(proposal_comment_rule());
    let action_rules = Arc::new(proposal_submission_action_rule());

    document_rules_map.insert(PROPOSAL.clone(), Arc::clone(&proposal_rules));
    document_rules_map.insert(PROPOSAL_COMMENT.clone(), Arc::clone(&comment_rules));
    document_rules_map.insert(
        PROPOSAL_SUBMISSION_ACTION.clone(),
        Arc::clone(&action_rules),
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
    Provider: CatalystSignedDocumentProvider,
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

/// Verify document signatures.
/// Return true if all signatures are valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate_signatures(
    doc: &CatalystSignedDocument,
    provider: &impl VerifyingKeyProvider,
) -> anyhow::Result<bool> {
    if doc.signatures().is_empty() {
        doc.report().other(
            "Catalyst Signed Document is unsigned",
            "During Catalyst Signed Document signature validation",
        );
        return Ok(false);
    }

    let sign_rules = doc
        .signatures()
        .iter()
        .map(|sign| validate_signature(doc, sign, provider, doc.report()));

    let res = futures::future::join_all(sign_rules)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?
        .iter()
        .all(|res| *res);

    Ok(res)
}

/// A single signature validation function
async fn validate_signature<Provider>(
    doc: &CatalystSignedDocument,
    sign: &Signature,
    provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<bool>
where
    Provider: VerifyingKeyProvider,
{
    let kid = sign.kid();

    let Some(pk) = provider.try_get_key(kid).await? else {
        report.other(
            &format!("Missing public key for {kid}."),
            "During public key extraction",
        );
        return Ok(false);
    };

    let tbs_data = tbs_data(kid, doc.doc_meta(), doc.content()).context("Probably a bug, cannot build CBOR COSE bytes for signature verification from the structurally valid COSE object.")?;

    let Ok(signature_bytes) = sign.signature().try_into() else {
        report.invalid_value(
            "cose signature",
            &format!("{}", sign.signature().len()),
            &format!("must be {}", ed25519_dalek::Signature::BYTE_SIZE),
            "During encoding cose signature to bytes",
        );
        return Ok(false);
    };

    let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
    if pk.verify_strict(&tbs_data, &signature).is_err() {
        report.functional_validation(
            &format!("Verification failed for signature with Key ID {kid}"),
            "During signature validation with verifying key",
        );
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::validator::document_rules_init;

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
