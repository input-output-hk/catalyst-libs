//! Catalyst Signed Documents validation logic

pub(crate) mod rules;
pub(crate) mod utils;

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::{Duration, SystemTime},
};

use anyhow::Context;
use catalyst_types::{catalyst_id::role_index::RoleId, problem_report::ProblemReport};
use rules::{
    ContentEncodingRule, ContentRule, ContentSchema, ContentTypeRule, LinkField,
    ParameterLinkRefRule, ParametersRule, RefRule, ReplyRule, Rules, SectionRule, SignatureKidRule,
};

use crate::{
    doc_types::{
        deprecated::{self},
        BRAND_PARAMETERS, CAMPAIGN_PARAMETERS, CATEGORY_PARAMETERS, PROPOSAL, PROPOSAL_COMMENT,
        PROPOSAL_COMMENT_TEMPLATE, PROPOSAL_SUBMISSION_ACTION, PROPOSAL_TEMPLATE,
    },
    metadata::DocType,
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
    signature::{tbs_data, Signature},
    CatalystSignedDocument, ContentEncoding, ContentType,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<DocType, Arc<Rules>>> = LazyLock::new(document_rules_init);

/// Returns an `DocType` from the provided argument.
/// Reduce redundant conversion.
/// This function should be used for hardcoded values, panic if conversion fail.
#[allow(clippy::expect_used)]
pub(crate) fn expect_doc_type<T>(t: T) -> DocType
where
    T: TryInto<DocType>,
    T::Error: std::fmt::Debug,
{
    t.try_into().expect("Failed to convert to DocType")
}

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
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Templated {
            exp_template_type: PROPOSAL_TEMPLATE.clone(),
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
        param_link_ref: ParameterLinkRefRule::Specified {
            field: LinkField::Template,
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
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Templated {
            exp_template_type: PROPOSAL_COMMENT_TEMPLATE.clone(),
        },
        doc_ref: RefRule::Specified {
            exp_ref_type: PROPOSAL.clone(),
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
        // Link field can be either template or ref
        param_link_ref: ParameterLinkRefRule::Specified {
            field: LinkField::Template,
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

    let proposal_action_json_schema = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(
            &serde_json::from_str(include_str!(
                "./../../../../specs/definitions/signed_docs/docs/payload_schemas/proposal_submission_action.schema.json"
            ))
            .expect("Must be a valid json file"),
        )
        .expect("Must be a valid json scheme file");
    Rules {
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
            exp_ref_type: PROPOSAL.clone(),
            optional: false,
        },
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleId::Proposer],
        },
        param_link_ref: ParameterLinkRefRule::Specified {
            field: LinkField::Ref,
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

    // Insert old rules (for backward compatibility)
    document_rules_map.insert(
        expect_doc_type(deprecated::COMMENT_DOCUMENT_UUID_TYPE),
        Arc::clone(&comment_rules),
    );
    document_rules_map.insert(
        expect_doc_type(deprecated::PROPOSAL_ACTION_DOCUMENT_UUID_TYPE),
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
    doc: &CatalystSignedDocument, provider: &Provider,
) -> anyhow::Result<bool>
where Provider: CatalystSignedDocumentProvider {
    let Ok(doc_type) = doc.doc_type() else {
        doc.report().missing_field(
            "type",
            "Can't get a document type during the validation process",
        );
        return Ok(false);
    };

    if !validate_id_and_ver(doc, provider)? {
        return Ok(false);
    }

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

/// Validates document `id` and `ver` fields on the timestamps:
/// 1. document `ver` cannot be smaller than document id field
/// 2. If `provider.future_threshold()` not `None`, document `id` cannot be too far in the
///    future (`future_threshold` arg) from `SystemTime::now()` based on the provide
///    threshold
/// 3. If `provider.future_threshold()` not `None`, document `id` cannot be too far behind
///    (`past_threshold` arg) from `SystemTime::now()` based on the provide threshold
fn validate_id_and_ver<Provider>(
    doc: &CatalystSignedDocument, provider: &Provider,
) -> anyhow::Result<bool>
where Provider: CatalystSignedDocumentProvider {
    let id = doc.doc_id().ok();
    let ver = doc.doc_ver().ok();
    if id.is_none() {
        doc.report().missing_field(
            "id",
            "Can't get a document id during the validation process",
        );
    }
    if ver.is_none() {
        doc.report().missing_field(
            "ver",
            "Can't get a document ver during the validation process",
        );
    }
    match (id, ver) {
        (Some(id), Some(ver)) => {
            let mut is_valid = true;
            if ver < id {
                doc.report().invalid_value(
                    "ver",
                    &ver.to_string(),
                    "ver < id",
                    &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
                );
                is_valid = false;
            }

            let (ver_time_secs, ver_time_nanos) = ver
                .uuid()
                .get_timestamp()
                .ok_or(anyhow::anyhow!("Document ver field must be a UUIDv7"))?
                .to_unix();

            let Some(ver_time) =
                SystemTime::UNIX_EPOCH.checked_add(Duration::new(ver_time_secs, ver_time_nanos))
            else {
                doc.report().invalid_value(
                    "ver",
                    &ver.to_string(),
                    "Must a valid duration since `UNIX_EPOCH`",
                    "Cannot instantiate a valid `SystemTime` value from the provided `ver` field timestamp.",
                );
                return Ok(false);
            };

            let now = SystemTime::now();

            if let Ok(version_age) = ver_time.duration_since(now) {
                // `now` is earlier than `ver_time`
                if let Some(future_threshold) = provider.future_threshold() {
                    if version_age > future_threshold {
                        doc.report().invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < now + future_threshold",
                        &format!("Document Version timestamp {id} cannot be too far in future (threshold: {future_threshold:?}) from now: {now:?}"),
                    );
                        is_valid = false;
                    }
                }
            } else {
                // `ver_time` is earlier than `now`
                let version_age = now
                    .duration_since(ver_time)
                    .context("BUG! `ver_time` must be earlier than `now` at this place")?;

                if let Some(past_threshold) = provider.past_threshold() {
                    if version_age > past_threshold {
                        doc.report().invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver > now - past_threshold",
                        &format!("Document Version timestamp {id} cannot be too far behind (threshold: {past_threshold:?}) from now: {now:?}",),
                    );
                        is_valid = false;
                    }
                }
            }

            Ok(is_valid)
        },

        _ => Ok(false),
    }
}

/// Verify document signatures.
/// Return true if all signatures are valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate_signatures(
    doc: &CatalystSignedDocument, provider: &impl VerifyingKeyProvider,
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
    doc: &CatalystSignedDocument, sign: &Signature, provider: &Provider, report: &ProblemReport,
) -> anyhow::Result<bool>
where Provider: VerifyingKeyProvider {
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
    use std::time::SystemTime;

    use uuid::{Timestamp, Uuid};

    use crate::{
        builder::tests::Builder,
        metadata::SupportedField,
        providers::{tests::TestCatalystSignedDocumentProvider, CatalystSignedDocumentProvider},
        validator::{document_rules_init, validate_id_and_ver},
        UuidV7,
    };

    #[test]
    fn document_id_and_ver_test() {
        let provider = TestCatalystSignedDocumentProvider::default();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uuid_v7 = UuidV7::new();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(uuid_v7))
            .with_metadata_field(SupportedField::Ver(uuid_v7))
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(is_valid);

        let ver = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0))
            .try_into()
            .unwrap();
        let id = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0))
            .try_into()
            .unwrap();
        assert!(ver < id);
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(id))
            .with_metadata_field(SupportedField::Ver(ver))
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);

        let to_far_in_past = Uuid::new_v7(Timestamp::from_unix_time(
            now - provider.past_threshold().unwrap().as_secs() - 1,
            0,
            0,
            0,
        ))
        .try_into()
        .unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(to_far_in_past))
            .with_metadata_field(SupportedField::Ver(to_far_in_past))
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);

        let to_far_in_future = Uuid::new_v7(Timestamp::from_unix_time(
            now + provider.future_threshold().unwrap().as_secs() + 1,
            0,
            0,
            0,
        ))
        .try_into()
        .unwrap();
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Id(to_far_in_future))
            .with_metadata_field(SupportedField::Ver(to_far_in_future))
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
