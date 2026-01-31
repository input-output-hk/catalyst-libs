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

impl CatalystSignedDocumentValidationRule for SignatureRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Self::check_inner(doc, provider)?;
        Ok(!doc.report().is_problematic())
    }
}

impl SignatureRule {
    /// Verify document signatures.
    /// Return true if all signatures are valid, otherwise return false.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    fn check_inner(
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<()> {
        if doc.signatures().is_empty() {
            doc.report().other(
                "Catalyst Signed Document is unsigned",
                "During Catalyst Signed Document signature validation",
            );
            return Ok(());
        }

        for signature in doc.signatures().iter() {
            validate_signature(doc, signature, provider, doc.report())?;
        }

        Ok(())
    }
}

/// A single signature validation function
fn validate_signature(
    doc: &CatalystSignedDocument,
    sign: &Signature,
    provider: &dyn CatalystIdProvider,
    report: &ProblemReport,
) -> anyhow::Result<()> {
    let kid = sign.kid();

    let Some(pk) = provider.try_get_registered_key(kid)? else {
        report.other(
            &format!("Missing public key for {kid}."),
            "During public key extraction",
        );
        return Ok(());
    };

    let tbs_data = tbs_data(kid, doc.doc_meta(), doc.content()).context("Probably a bug, cannot build CBOR COSE bytes for signature verification from the structurally valid COSE object.")?;

    let Ok(signature_bytes) = sign.signature().try_into() else {
        report.invalid_value(
            "cose signature",
            &format!("{}", sign.signature().len()),
            &format!("must be {}", ed25519_dalek::Signature::BYTE_SIZE),
            "During encoding cose signature to bytes",
        );
        return Ok(());
    };

    let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
    if pk.verify_strict(&tbs_data, &signature).is_err() {
        report.functional_validation(
            &format!("Verification failed for signature with Key ID {kid}"),
            "During signature validation with verifying key",
        );
    }

    Ok(())
}
