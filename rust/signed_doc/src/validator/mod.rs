//! Catalyst Signed Documents validation logic

pub(crate) mod rules;
pub(crate) mod utils;

use std::{collections::HashMap, sync::LazyLock};

use catalyst_types::{id_uri::IdUri, problem_report::ProblemReport, uuid::Uuid};
use coset::{CoseSign, CoseSignature};
use rules::{
    CategoryRule, ContentEncodingRule, ContentTypeRule, RefRule, ReplyRule, Rules, SectionRule,
    TemplateRule,
};

use crate::{
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE,
        PROPOSAL_TEMPLATE_UUID_TYPE,
    },
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
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
            validate_singature(&cose_sign, signature, kid, provider, doc.report())
        });

    let res = futures::future::join_all(sign_rules)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?
        .iter()
        .all(|res| *res);

    Ok(res)
}

async fn validate_singature<Provider>(
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
