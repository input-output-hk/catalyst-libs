//! Validator for Signed Document Version

use crate::CatalystSignedDocument;

/// Signed Document `ver` field validation rule
pub(crate) struct VerRule;

impl VerRule {
    /// Validates document `ver` field on the timestamps:
    /// 1. document `ver` cannot be smaller than document `id` field
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
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

        if ver < id {
            doc.report().invalid_value(
                "ver",
                &ver.to_string(),
                "ver < id",
                &format!("Document Version {ver} cannot be smaller than Document ID {id}"),
            );
            return Ok(false);
        }

        Ok(true)
    }
}
