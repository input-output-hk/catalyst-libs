//! Catalyst Signed Documents validation logic

pub(crate) mod rules;
pub(crate) mod utils;

use std::{
    collections::HashMap,
    sync::LazyLock,
    time::{Duration, SystemTime},
};

use anyhow::Context;
use catalyst_types::{
    id_uri::{role_index::RoleIndex, IdUri},
    problem_report::ProblemReport,
    uuid::Uuid,
};
use coset::{CoseSign, CoseSignature};
use rules::{
    CategoryRule, ContentEncodingRule, ContentRule, ContentSchema, ContentTypeRule, RefRule,
    ReplyRule, Rules, SectionRule, SignatureKidRule,
};

use crate::{
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        PROPOSAL_DOCUMENT_UUID_TYPE, PROPOSAL_TEMPLATE_UUID_TYPE,
    },
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
    CatalystSignedDocument, ContentEncoding, ContentType,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<Uuid, Rules>> = LazyLock::new(document_rules_init);

/// `DOCUMENT_RULES` initialization function
#[allow(clippy::expect_used)]
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
        content: ContentRule::Templated {
            exp_template_type: PROPOSAL_TEMPLATE_UUID_TYPE
                .try_into()
                .expect("Must be a valid UUID V4"),
        },
        category: CategoryRule::Specified { optional: true },
        doc_ref: RefRule::NotSpecified,
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleIndex::PROPOSER],
        },
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
        content: ContentRule::Templated {
            exp_template_type: COMMENT_TEMPLATE_UUID_TYPE
                .try_into()
                .expect("Must be a valid UUID V4"),
        },
        doc_ref: RefRule::Specified {
            exp_ref_type: PROPOSAL_DOCUMENT_UUID_TYPE
                .try_into()
                .expect("Must be a valid UUID V4"),
            optional: false,
        },
        reply: ReplyRule::Specified {
            exp_reply_type: COMMENT_DOCUMENT_UUID_TYPE
                .try_into()
                .expect("Must be a valid UUID V4"),
            optional: true,
        },
        section: SectionRule::Specified { optional: true },
        category: CategoryRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleIndex::ROLE_0],
        },
    };
    document_rules_map.insert(COMMENT_DOCUMENT_UUID_TYPE, comment_document_rules);

    let proposal_action_json_schema = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .build(
            &serde_json::from_str(include_str!(
                "./../../../../specs/signed_docs/docs/payload_schemas/proposal_submission_action.schema.json"
            ))
            .expect("Must be a valid json file"),
        )
        .expect("Must be a valid json scheme file");
    let proposal_submission_action_rules = Rules {
        content_type: ContentTypeRule {
            exp: ContentType::Json,
        },
        content_encoding: ContentEncodingRule {
            exp: ContentEncoding::Brotli,
            optional: false,
        },
        content: ContentRule::Static(ContentSchema::Json(proposal_action_json_schema)),
        category: CategoryRule::Specified { optional: true },
        doc_ref: RefRule::Specified {
            exp_ref_type: PROPOSAL_DOCUMENT_UUID_TYPE
                .try_into()
                .expect("Must be a valid UUID V4"),
            optional: false,
        },
        reply: ReplyRule::NotSpecified,
        section: SectionRule::NotSpecified,
        kid: SignatureKidRule {
            exp: &[RoleIndex::PROPOSER],
        },
    };

    document_rules_map.insert(
        PROPOSAL_ACTION_DOCUMENT_UUID_TYPE,
        proposal_submission_action_rules,
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
    let Ok(cose_sign) = doc.as_cose_sign() else {
        doc.report().other(
            "Cannot build a COSE sign object",
            "During encoding signed document as COSE SIGN",
        );
        return Ok(false);
    };

    let sign_rules = doc
        .signatures()
        .cose_signatures_with_kids()
        .map(|(signature, kid)| {
            validate_signature(&cose_sign, signature, kid, provider, doc.report())
        });

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
    cose_sign: &CoseSign, signature: &CoseSignature, kid: &IdUri, provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<bool>
where
    Provider: VerifyingKeyProvider,
{
    let Some(pk) = provider.try_get_key(kid).await? else {
        report.other(
            &format!("Missing public key for {kid}."),
            "During public key extraction",
        );
        return Ok(false);
    };

    let tbs_data = cose_sign.tbs_data(&[], signature);
    let Ok(signature_bytes) = signature.signature.as_slice().try_into() else {
        report.invalid_value(
            "cose signature",
            &format!("{}", signature.signature.len()),
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
        providers::{tests::TestCatalystSignedDocumentProvider, CatalystSignedDocumentProvider},
        validator::{document_rules_init, validate_id_and_ver},
        Builder, UuidV7,
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
            .with_json_metadata(serde_json::json!({
                "id": uuid_v7.to_string(),
                "ver": uuid_v7.to_string()
            }))
            .unwrap()
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(is_valid);

        let ver = Uuid::new_v7(Timestamp::from_unix_time(now - 1, 0, 0, 0));
        let id = Uuid::new_v7(Timestamp::from_unix_time(now + 1, 0, 0, 0));
        assert!(ver < id);
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": id.to_string(),
                "ver": ver.to_string()
            }))
            .unwrap()
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);

        let to_far_in_past = Uuid::new_v7(Timestamp::from_unix_time(
            now - provider.past_threshold().unwrap().as_secs() - 1,
            0,
            0,
            0,
        ));
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": to_far_in_past.to_string(),
                "ver": to_far_in_past.to_string()
            }))
            .unwrap()
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);

        let to_far_in_future = Uuid::new_v7(Timestamp::from_unix_time(
            now + provider.future_threshold().unwrap().as_secs() + 1,
            0,
            0,
            0,
        ));
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": to_far_in_future.to_string(),
                "ver": to_far_in_future.to_string()
            }))
            .unwrap()
            .build();

        let is_valid = validate_id_and_ver(&doc, &provider).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
