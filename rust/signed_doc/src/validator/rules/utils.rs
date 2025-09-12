//! utility functions for validation rules

use std::fmt::Write;

use crate::{validator::json_schema::JsonSchema, CatalystSignedDocument};

/// Validating the document's content against the provided JSON schema
pub(crate) fn content_json_schema_check(
    doc: &CatalystSignedDocument,
    schema: &JsonSchema,
) -> bool {
    const CONTEXT: &str = "Document content JSON schema validation";

    let Ok(doc_content) = doc.decoded_content() else {
        doc.report().functional_validation(
            "Invalid Document content, cannot get decoded bytes",
            CONTEXT,
        );
        return false;
    };
    if doc_content.is_empty() {
        doc.report()
            .missing_field("payload", "Document must have a content");
        return false;
    }
    let Ok(doc_json) = serde_json::from_slice(&doc_content) else {
        doc.report()
            .functional_validation("Document content must be json encoded", CONTEXT);
        return false;
    };

    let schema_validation_errors =
        schema
            .iter_errors(&doc_json)
            .fold(String::new(), |mut str, e| {
                let _ = write!(str, "{{ {e} }}, ");
                str
            });

    if !schema_validation_errors.is_empty() {
        doc.report().functional_validation(
            &format!(
                "Proposal document content does not compliant with the json schema. [{schema_validation_errors}]"
            ),
            CONTEXT,
        );
        return false;
    }

    true
}

#[cfg(test)]
pub(super) fn create_dummy_key_pair(
    role_index: catalyst_types::catalyst_id::role_index::RoleId
) -> (
    ed25519_dalek::SigningKey,
    ed25519_dalek::VerifyingKey,
    catalyst_types::catalyst_id::CatalystId,
) {
    let sk = create_signing_key();
    let pk = sk.verifying_key();
    let kid =
        catalyst_types::catalyst_id::CatalystId::new("cardano", None, pk).with_role(role_index);
    (sk, pk, kid)
}

#[cfg(test)]
pub(super) fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}
