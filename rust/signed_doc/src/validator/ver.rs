//! Validator for Signed Document Version

use crate::CatalystSignedDocument;

pub(crate) struct VerRule;

impl VerRule {
    pub(crate) fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        let Ok(id) = doc.doc_id() else {
            doc.report().missing_field(
                "id",
                "Can't get a document id during the validation process",
            );
            return Ok(false);
        };

        let Ok(ver) = doc.doc_ver() else {
            doc.report().missing_field(
                "ver",
                "Can't get a document ver during the validation process",
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
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
