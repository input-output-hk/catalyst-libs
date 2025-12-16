//! Validator for Signatures

#[cfg(test)]
mod tests;

use anyhow::Context;
use catalyst_types::problem_report::ProblemReport;

use crate::{
    CatalystSignedDocument,
    providers::{CatalystIdProvider, Provider},
    signature::{Signature, tbs_data},
    validator::CatalystSignedDocumentValidationRule,
};

/// Signed Document signatures validation rule.
#[derive(Debug)]
pub(crate) struct SignatureRule;

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for SignatureRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider).await
    }
}

impl SignatureRule {
    /// Verify document signatures.
    /// Return true if all signatures are valid, otherwise return false.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    async fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
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
}

/// A single signature validation function
async fn validate_signature(
    doc: &CatalystSignedDocument,
    sign: &Signature,
    provider: &dyn CatalystIdProvider,
    report: &ProblemReport,
) -> anyhow::Result<bool> {
    let kid = sign.kid();

    let Some(pk) = provider.try_get_registered_key(kid).await? else {
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
