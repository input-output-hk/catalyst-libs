//! Comment Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/comment/#comment-document>

use catalyst_types::problem_report::ProblemReport;
use jsonpath_rust::JsonPath;

use crate::{
    doc_types::{
        COMMENT_DOCUMENT_UUID_TYPE, COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE,
    },
    error::CatalystSignedDocError,
    metadata::{ContentEncoding, ContentType},
    validator::{ValidationDataProvider, ValidationRule},
    CatalystSignedDocument,
};

/// Comment Document struct
#[allow(dead_code)]
pub struct CommentDocument(CatalystSignedDocument);

impl CommentDocument {
    /// Try to build `CommentDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, error_report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        let context = "Catalyst Signed Document to Comment Document";
        let mut failed = false;

        for rule in comment_document_validation_rules() {
            if !(rule.validator)(doc, error_report) {
                error_report.other(&rule.description, context);
                failed = true;
            }
        }

        if failed {
            anyhow::bail!("Failed to build `CommentDocument` from `CatalystSignedDoc`");
        }

        Ok(Self(doc.clone()))
    }

    /// A comprehensive validation of the `CommentDocument` content.
    pub(crate) fn validate_with_report(
        &self, validator: &impl ValidationDataProvider, error_report: &ProblemReport,
    ) {
        let context = "Comment Document Comprehensive Validation";
        let doc_ref = self.0.doc_meta().doc_ref();
        if let Some(doc_ref) = doc_ref {
            match validator.get_doc_ref(&doc_ref) {
                Some(proposal_doc) => {
                    for rule in reference_validation_rules() {
                        if !(rule.validator)(&proposal_doc, error_report) {
                            error_report.other(
                                &rule.description,
                                "During Comment Document reference validation",
                            );
                        }
                    }
                    if let Some(reply_ref) = self.0.doc_meta().reply() {
                        match validator.get_doc_ref(&reply_ref) {
                            Some(reply_doc) => {
                                let context = "During Comment Document reply validation";
                                for rule in reply_validation_rules() {
                                    if !(rule.validator)(&reply_doc, error_report) {
                                        error_report.other(&rule.description, context);
                                    }
                                }
                                let error_msg = "Reply document must reference the same proposal";
                                match reply_doc.doc_meta().doc_ref() {
                                    Some(reply_ref) => {
                                        if reply_ref != doc_ref {
                                            error_report.other(error_msg, context);
                                        }
                                    },
                                    None => {
                                        error_report.other(error_msg, context);
                                    },
                                }
                            },
                            None => {
                                error_report.other("Unable to fetch Reply document", context);
                            },
                        }
                    }
                },
                None => {
                    error_report.other("Unable to fetch reference proposal document", context);
                },
            }
        }
        // Validate content with template JSON Schema
        match self
            .0
            .doc_meta()
            .template()
            .and_then(|t| validator.get_doc_ref(&t))
        {
            Some(template_doc) => {
                //
                for rule in comment_document_template_validation_rules() {
                    if !(rule.validator)(&template_doc, error_report) {
                        error_report.other(
                            &rule.description,
                            "During Comment Document template validation",
                        );
                    }
                }
            },
            None => {
                error_report.other("No template was found", context);
            },
        }
    }
}

impl TryFrom<CatalystSignedDocument> for CommentDocument {
    type Error = CatalystSignedDocError;

    fn try_from(doc: CatalystSignedDocument) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Comment Document");
        let res = Self::from_signed_doc(&doc, &error_report)
            .map_err(|e| CatalystSignedDocError::new(error_report, e))?;
        Ok(res)
    }
}

/// Stateless validation rules for Comment Document
fn comment_document_validation_rules() -> Vec<ValidationRule<CatalystSignedDocument>> {
    vec![
        ValidationRule {
            field: "content-type".to_string(),
            description: format!(
                "Comment Document content-type must be {}",
                ContentType::Json
            ),
            validator: |doc: &CatalystSignedDocument, _| {
                doc.doc_content_type() == ContentType::Json
            },
        },
        ValidationRule {
            field: "content-encoding".to_string(),
            description: format!(
                "Comment Document content-encoding must be {}",
                ContentEncoding::Brotli,
            ),
            validator: |doc: &CatalystSignedDocument, _| {
                match doc.doc_content_encoding() {
                    Some(encoding) => encoding != ContentEncoding::Brotli,
                    None => false,
                }
            },
        },
        ValidationRule {
            field: "type".to_string(),
            description: format!(
                "Comment Document type UUID value must be {COMMENT_DOCUMENT_UUID_TYPE}"
            ),
            validator: |doc: &CatalystSignedDocument, _| {
                doc.doc_type().uuid() != COMMENT_DOCUMENT_UUID_TYPE
            },
        },
        ValidationRule {
            field: "ref".to_string(),
            description: "Comment Document ref must be valid".to_string(),
            validator: |doc: &CatalystSignedDocument, _| {
                match doc.doc_meta().doc_ref() {
                    Some(doc_ref) => doc_ref.is_valid(),
                    None => true,
                }
            },
        },
        ValidationRule {
            field: "template".to_string(),
            description: format!(
                "Comment Document template UUID value must be {COMMENT_TEMPLATE_UUID_TYPE}"
            ),
            validator: |doc: &CatalystSignedDocument, _| {
                match doc.doc_meta().template() {
                    Some(template) => template.id.uuid() != COMMENT_TEMPLATE_UUID_TYPE,
                    None => false,
                }
            },
        },
        ValidationRule {
            field: "reply".to_string(),
            description: "Comment Document reply document reference must be valid".to_string(),
            validator: |doc: &CatalystSignedDocument, _| {
                match doc.doc_meta().reply() {
                    Some(reply) => reply.is_valid(),
                    None => true,
                }
            },
        },
        ValidationRule {
            field: "section".to_string(),
            description: "Comment Document section must be valid JSON path".to_string(),
            validator: |doc: &CatalystSignedDocument, _| {
                match doc.doc_meta().section() {
                    Some(section) => {
                        JsonPath::<serde_json::Value>::try_from(section.as_str()).is_ok()
                    },
                    None => true,
                }
            },
        },
    ]
}

/// Functional validation rules for Comment Document reference
fn reference_validation_rules() -> Vec<ValidationRule<CatalystSignedDocument>> {
    vec![
        ValidationRule {
            field: "ref".to_string(),
            description: format!(
                "Comment Document reference document type must be {PROPOSAL_DOCUMENT_UUID_TYPE}"
            ),
            validator: |signed_doc: &CatalystSignedDocument, _| {
                signed_doc.doc_type().uuid() == PROPOSAL_DOCUMENT_UUID_TYPE
            },
        },
        ValidationRule {
            field: "ref".to_string(),
            description: format!(
                "Comment Document reference document type must be {PROPOSAL_DOCUMENT_UUID_TYPE}"
            ),
            validator: |signed_doc: &CatalystSignedDocument, _| {
                signed_doc.doc_type().uuid() == PROPOSAL_DOCUMENT_UUID_TYPE
            },
        },
    ]
}

/// Functional validation rules for Comment Document template
fn comment_document_template_validation_rules() -> Vec<ValidationRule<CatalystSignedDocument>> {
    vec![ValidationRule {
        field: "template".to_string(),
        description: "Comment Document conforms to template schema".to_string(),
        validator: |signed_doc: &CatalystSignedDocument, error_report| {
            let mut success = false;
            match serde_json::from_slice(signed_doc.doc_content().decoded_bytes()) {
                Ok(template_json) => {
                    match jsonschema::draft7::new(&template_json) {
                        Ok(schema) => {
                            success = schema.is_valid(&template_json);
                        },
                        Err(e) => {
                            error_report.other(&format!("Invalid JSON schema: {e:?}"), "");
                        },
                    }
                },
                Err(e) => {
                    error_report.other(
                        &format!("Document does not conform to template schema: {e:?}"),
                        "",
                    );
                },
            }
            success
        },
    }]
}

/// Functional validation rules for Comment Document reply
fn reply_validation_rules() -> Vec<ValidationRule<CatalystSignedDocument>> {
    vec![ValidationRule {
        field: "ref".to_string(),
        description: format!(
            "Comment Document reference document type must be {COMMENT_DOCUMENT_UUID_TYPE}"
        ),
        validator: |signed_doc: &CatalystSignedDocument, _| {
            signed_doc.doc_type().uuid() != COMMENT_DOCUMENT_UUID_TYPE
        },
    }]
}
