//! Validator for Signed Document Version

#[cfg(test)]
mod tests;

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// Signed Document `ver` field validation rule
#[derive(Debug)]
pub(crate) struct VerRule;

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for VerRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc, provider).await
    }
}

impl VerRule {
    /// Validates document `ver` field according to the following rules:
    /// 1. Document `ver` cannot be smaller than document `id` field
    /// 2. IF `ver` does not == `id` then a document with `id` and `ver` being equal
    ///    *MUST* exist
    /// 3. When a document with the same `id` already exists, the new document's `ver`
    ///    must be greater than the latest known submitted version for that `id`
    /// 4. When a document with the same `id` already exists, the new document's `type`
    ///    must be the same as the latest known submitted document's `type` for that `id`
    async fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };
        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Cannot get the document field during the field validation",
            );
            return Ok(false);
        };

        let mut is_valid = true;

        if ver < id {
            doc.report().invalid_value(
                "ver",
                &ver.to_string(),
                "ver < id",
                &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
            );
            is_valid = false;
        } else if let Some(last_doc) = provider.try_get_last_doc(id).await? {
            let Ok(last_doc_ver) = last_doc.doc_ver() else {
                doc.report().missing_field(
                    "ver",
                    &format!(
                        "Missing `ver` field in the latest known document, for the the id {id}"
                    ),
                );
                return Ok(false);
            };

            if last_doc_ver >= ver {
                doc.report().functional_validation(
                    &format!("New document ver should be greater that the submitted latest known. New document ver: {ver}, latest known ver: {last_doc_ver}"),
                    &format!("Document's `ver` field should continuously increasing, for the the id {id}"),
                );
                is_valid = false;
            }

            let Ok(last_doc_type) = last_doc.doc_type() else {
                doc.report().missing_field(
                    "type",
                    &format!(
                        "Missing `type` field in the latest known document. Last known document id: {id}, ver: {last_doc_ver}."
                    ),
                );
                return Ok(false);
            };

            let Ok(doc_type) = doc.doc_type() else {
                doc.report().missing_field("type", "Missing `type` field.");
                return Ok(false);
            };

            if last_doc_type != doc_type {
                doc.report().functional_validation(
                    &format!("New document type should be the same that the submitted latest known. New document type: {doc_type}, latest known ver: {last_doc_type}"),
                    &format!("Document's type should be the same for all documents with the same id {id}"),
                );
                is_valid = false;
            }
        } else if ver != id {
            doc.report().functional_validation(
                &format!("`ver` and `id` are not equal, ver: {ver}, id: {id}. Document with `id` and `ver` being equal MUST exist"),
                "Cannot get a first version document from the provider, document for which `id` and `ver` are equal.",
            );
            is_valid = false;
        }

        Ok(is_valid)
    }
}
