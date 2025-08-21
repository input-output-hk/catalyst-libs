//! Validator for Signed Document Version

use std::time::{Duration, SystemTime};

use crate::{providers::CatalystSignedDocumentProvider, CatalystSignedDocument};

pub(crate) struct VerRule;

impl VerRule {
    pub(crate) fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider,
    {
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
                if ver < id {
                    doc.report().invalid_value(
                        "ver",
                        &ver.to_string(),
                        "ver < id",
                        &format!(
                            "Document Version {ver} cannot be smaller than Document ID {id}"
                        ),
                    );
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(false),
        }
    }
}
