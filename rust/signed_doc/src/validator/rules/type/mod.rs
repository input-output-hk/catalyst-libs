//! Validator for Signed Document ID

#[cfg(test)]
mod tests;

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

/// Signed Document `type` field validation rule
#[derive(Debug)]
pub(crate) struct TypeRule;

impl CatalystSignedDocumentValidationRule for TypeRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Self::check_inner(doc, provider)?;
        Ok(!doc.report().is_problematic())
    }
}

impl TypeRule {
    /// For a new document version ('id' != 'ver'), 'type' for the new document
    ///  must be the same as the latest known submitted document's `type` for that `id`
    fn check_inner(
        doc: &CatalystSignedDocument,
        provider: &dyn Provider,
    ) -> anyhow::Result<()> {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Cannot get the document field during the field validation",
            );
            return Ok(());
        };
        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Cannot get the document field during the field validation",
            );
            return Ok(());
        };

        if id != ver {
            if let Some(last_doc) = provider.try_get_last_doc(id)? {
                let Ok(last_doc_type) = last_doc.doc_type() else {
                    doc.report().missing_field(
                    "type",
                    &format!(
                        "Missing `type` field in the latest known document. Last known document id: {id}."
                    ),
                );
                    return Ok(());
                };

                let Ok(doc_type) = doc.doc_type() else {
                    doc.report().missing_field("type", "Missing `type` field.");
                    return Ok(());
                };

                if last_doc_type != doc_type {
                    doc.report().functional_validation(
                        &format!("New document type should be the same that the submitted latest known. New document type: {doc_type}, latest known ver: {last_doc_type}"),
                        &format!("Document's type should be the same for all documents with the same id {id}"),
                    );
                }
            } else {
                doc.report().functional_validation(
                    &format!("`ver` and `id` are not equal, ver: {ver}, id: {id}. Document with `id` and `ver` being equal MUST exist"),
                    "Cannot get a first version document from the provider, document for which `id` and `ver` are equal.",
                );
            }
        }

        Ok(())
    }
}
