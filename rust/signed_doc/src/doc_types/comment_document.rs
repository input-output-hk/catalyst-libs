//! Comment Document object implementation
//! https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/comment/#comment-document

use catalyst_types::problem_report::ProblemReport;

use crate::{error::CatalystSignedDocError, CatalystSignedDocument};

/// Comment document `UuidV4` type.
pub const COMMENT_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA);

/// Comment Document struct
pub struct CommentDocument {
    /// Proposal document content data
    /// TODO: change it to `serde_json::Value` type
    #[allow(dead_code)]
    content: Vec<u8>,
}

impl CommentDocument {
    /// Try to build `CommentDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    #[allow(dead_code)]
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, error_report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        /// Context for error messages.
        const CONTEXT: &str = "Catalyst Signed Document to Proposal Document";
        let mut failed = false;

        if doc.doc_type().uuid() != COMMENT_DOCUMENT_UUID_TYPE {
            error_report.invalid_value(
                "`type`",
                &doc.doc_type().to_string(),
                &format!("Proposal Document type UUID value is {COMMENT_DOCUMENT_UUID_TYPE}"),
                CONTEXT,
            );
            failed = true;
        }

        // TODO add other validation

        if failed {
            anyhow::bail!("Failed to build `CommentDocument` from `CatalystSignedDoc`");
        }

        let content = doc.doc_content().decoded_bytes().to_vec();
        Ok(Self { content })
    }

    /// A comprehensive validation of the `CommentDocument` content.
    #[allow(clippy::unused_self)]
    pub(crate) fn validate_with_report<F>(&self, _doc_getter: F, _error_report: &ProblemReport)
    where F: FnMut() -> Option<CatalystSignedDocument> {
        // TODO: implement the rest of the validation
    }
}

impl TryFrom<CatalystSignedDocument> for CommentDocument {
    type Error = CatalystSignedDocError;

    fn try_from(doc: CatalystSignedDocument) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Proposal Document");
        let res = Self::from_signed_doc(&doc, &error_report)
            .map_err(|e| CatalystSignedDocError::new(error_report, e))?;
        Ok(res)
    }
}
