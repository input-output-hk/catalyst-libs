//! `ref` rule type impl.

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{Uuid, UuidV4},
};

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// `ref` field validation rule
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RefRule {
    /// Is 'ref' specified
    Specified {
        /// expected `type` field of the referenced doc
        exp_ref_type: UuidV4,
        /// optional flag for the `ref` field
        optional: bool,
    },
    /// 'ref' is not specified
    NotSpecified,
}
impl RefRule {
    /// Field validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified {
            exp_ref_type,
            optional,
        } = self
        {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                let ref_validator = |ref_doc: CatalystSignedDocument| {
                    Ok(referenced_doc_check(
                        &ref_doc,
                        exp_ref_type.uuid(),
                        "ref",
                        doc.report(),
                    ))
                };
                return validate_provided_doc(&doc_ref, provider, doc.report(), ref_validator)
                    .await;
            } else if !optional {
                doc.report()
                    .missing_field("ref", "Document must have a ref field");
                return Ok(false);
            }
        }
        if &Self::NotSpecified == self {
            if let Some(doc_ref) = doc.doc_meta().doc_ref() {
                doc.report().unknown_field(
                    "ref",
                    &doc_ref.to_string(),
                    "Document does not expect to have a ref field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A generic implementation of the referenced document validation.
pub(crate) fn referenced_doc_check(
    ref_doc: &CatalystSignedDocument, exp_ref_type: Uuid, field_name: &str, report: &ProblemReport,
) -> bool {
    let Ok(ref_doc_type) = ref_doc.doc_type() else {
        report.missing_field("type", "Referenced document must have type field");
        return false;
    };
    if ref_doc_type.uuid() != exp_ref_type {
        report.invalid_value(
            field_name,
            ref_doc_type.to_string().as_str(),
            exp_ref_type.to_string().as_str(),
            "Invalid referenced document type",
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::UuidV7;

    use super::*;
    use crate::{providers::tests::TestCatalystSignedDocumentProvider2, Builder};

    #[tokio::test]
    async fn ref_rule_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider2::default();

        let ref_doc_type = UuidV4::new();
        let ref_id = UuidV7::new();
        let ref_doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": ref_id.to_string(),
                "type": ref_doc_type.to_string()
            }))
            .unwrap()
            .build();
        provider.add_document(ref_doc).unwrap();

        // all correct
        let rule = RefRule::Specified {
            exp_ref_type: ref_doc_type,
            optional: true,
        };
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": ref_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // all correct, `ref` field is missing, but its optional
        let rule = RefRule::Specified {
            exp_ref_type: ref_doc_type,
            optional: true,
        };
        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // missing `ref` field, but its required
        let rule = RefRule::Specified {
            exp_ref_type: ref_doc_type,
            optional: false,
        };
        let doc = Builder::new().build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // reference to the document with another `type` field
        let another_ref_doc_type = UuidV4::new();
        let new_ref_id = UuidV7::new();
        let ref_doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": new_ref_id.to_string(),
                "type": another_ref_doc_type.to_string()
            }))
            .unwrap()
            .build();
        provider.add_document(ref_doc).unwrap();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": new_ref_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // missing `type` field in the referenced document
        let new_ref_id = UuidV7::new();
        let ref_doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "id": new_ref_id.to_string(),
            }))
            .unwrap()
            .build();
        provider.add_document(ref_doc).unwrap();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": new_ref_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());

        // cannot find a referenced document
        let new_ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": new_ref_id.to_string() }
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn ref_rule_not_specified_test() {
        let rule = RefRule::NotSpecified;
        let provider = TestCatalystSignedDocumentProvider2::default();

        let doc = Builder::new().build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        let ref_id = UuidV7::new();
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({"ref": {"id": ref_id.to_string() } }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
