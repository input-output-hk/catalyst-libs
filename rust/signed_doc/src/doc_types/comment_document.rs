//! Comment Document object implementation
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_docs/comment/#comment-document>

use catalyst_types::{problem_report::ProblemReport, uuid::Uuid};

use crate::{
    doc_types::{COMMENT_TEMPLATE_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE},
    error::CatalystSignedDocError,
    metadata::{ContentEncoding, ContentType},
    validator::{utils::validate_provided_doc, StatefullValidation, StatelessValidation},
    CatalystSignedDocument, DocumentRef,
};

/// Comment document `UuidV4` type.
pub const COMMENT_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA);

/// Comment Document struct
pub struct CommentDocument {
    /// `template` field
    template: DocumentRef,
    /// `ref` field
    doc_ref: DocumentRef,
    /// `category_id` field
    reply: Option<DocumentRef>,
    /// `section` field
    #[allow(dead_code)]
    section: Option<String>,
    /// Comment content
    content: serde_json::Value,
}

impl StatelessValidation for CommentDocument {
    const STATELESS_RULES: &[crate::validator::StatelessRule] = &[
        type_check,
        content_type_check,
        content_encoding_check,
        template_check,
        reply_check,
    ];
}

impl<DocProvider> StatefullValidation<DocProvider> for CommentDocument
where DocProvider: 'static + Fn(&DocumentRef) -> Option<CatalystSignedDocument>
{
    const STATEFULL_RULES: &[crate::validator::StatefullRule<Self, DocProvider>] =
        &[template_full_check, ref_full_check, reply_full_check];
}

/// `type` field validation
fn type_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_type().uuid() != COMMENT_DOCUMENT_UUID_TYPE {
        report.invalid_value(
            "type",
            doc.doc_type().to_string().as_str(),
            COMMENT_DOCUMENT_UUID_TYPE.to_string().as_str(),
            "Invalid Comment Document type UUID value",
        );
        return false;
    }
    true
}

/// `content-type` validation
fn content_type_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_content_type() != ContentType::Json {
        report.invalid_value(
            "content-type",
            doc.doc_content_type().to_string().as_str(),
            ContentType::Json.to_string().as_str(),
            "Invalid Comment Document content-type value",
        );
        return false;
    }
    true
}

/// `content-encoding` validation
fn content_encoding_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if let Some(content_encoding) = doc.doc_content_encoding() {
        if content_encoding != ContentEncoding::Brotli {
            report.invalid_value(
                "content-encoding",
                content_encoding.to_string().as_str(),
                ContentEncoding::Brotli.to_string().as_str(),
                "Invalid Comment Document content-encoding value",
            );
            return false;
        }
    } else {
        report.missing_field(
            "content-encoding",
            "Comment Document must have a content-encoding field",
        );
        return false;
    }
    true
}

/// `template` validation
fn template_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_meta().template().is_none() {
        report.missing_field("template", "Comment Document must have a template field");
        return false;
    }
    true
}

/// `reply` validation
fn reply_check(doc: &CatalystSignedDocument, report: &ProblemReport) -> bool {
    if doc.doc_meta().doc_ref().is_none() {
        report.missing_field("ref", "Comment Document must have a ref field");
        return false;
    }
    true
}

/// `template` statefull validation
fn template_full_check<DocProvider>(
    doc: &CommentDocument, provider: &DocProvider, report: &ProblemReport,
) -> bool
where DocProvider: Fn(&DocumentRef) -> Option<CatalystSignedDocument> {
    let template_validator = |template_doc: CatalystSignedDocument| {
        if template_doc.doc_type().uuid() != COMMENT_TEMPLATE_UUID_TYPE {
            report.invalid_value(
                "template",
                template_doc.doc_type().to_string().as_str(),
                COMMENT_TEMPLATE_UUID_TYPE.to_string().as_str(),
                "Invalid referenced template document type",
            );
            return false;
        }
        let Ok(template_json_schema) =
            serde_json::from_slice(template_doc.doc_content().decoded_bytes())
        else {
            report.functional_validation(
                "Template document content must be json encoded",
                "Invalid referenced template document content",
            );
            return false;
        };
        let Ok(schema_validator) = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft7)
            .build(&template_json_schema)
        else {
            report.functional_validation(
                "Template document content must be Draft 7 JSON schema",
                "Invalid referenced template document content",
            );
            return false;
        };

        if schema_validator.validate(&doc.content).is_err() {
            report.functional_validation(
                "Comment document content does not compliant with the template json schema",
                "Invalid Comment document content",
            );
            return false;
        }

        true
    };
    validate_provided_doc(
        &doc.template,
        "Comment Template",
        provider,
        report,
        template_validator,
    )
}

/// `ref` statefull validation
fn ref_full_check<DocProvider>(
    doc: &CommentDocument, provider: &DocProvider, report: &ProblemReport,
) -> bool
where DocProvider: Fn(&DocumentRef) -> Option<CatalystSignedDocument> {
    let ref_validator = |proposal_doc: CatalystSignedDocument| -> bool {
        if proposal_doc.doc_type().uuid() != PROPOSAL_DOCUMENT_UUID_TYPE {
            report.invalid_value(
                "ref",
                proposal_doc.doc_type().to_string().as_str(),
                PROPOSAL_DOCUMENT_UUID_TYPE.to_string().as_str(),
                "Invalid referenced proposal document type",
            );
            return false;
        }
        true
    };
    validate_provided_doc(&doc.doc_ref, "Proposal", provider, report, ref_validator)
}

/// `reply` statefull validation
fn reply_full_check<DocProvider>(
    doc: &CommentDocument, provider: &DocProvider, report: &ProblemReport,
) -> bool
where DocProvider: Fn(&DocumentRef) -> Option<CatalystSignedDocument> {
    if let Some(reply) = &doc.reply {
        let reply_validator = |comment_doc: CatalystSignedDocument| -> bool {
            if comment_doc.doc_type().uuid() != COMMENT_DOCUMENT_UUID_TYPE {
                report.invalid_value(
                    "reply",
                    comment_doc.doc_type().to_string().as_str(),
                    COMMENT_DOCUMENT_UUID_TYPE.to_string().as_str(),
                    "Invalid referenced comment document type",
                );
                return false;
            }
            let Some(doc_ref) = comment_doc.doc_meta().doc_ref() else {
                report.missing_field("ref", "Invalid referenced comment document");
                return false;
            };

            if doc_ref.id != doc.doc_ref.id {
                report.invalid_value(
                    "reply",
                    doc_ref.id .to_string().as_str(),
                    doc.doc_ref.id.to_string().as_str(),
                    "Invalid referenced comment document. Proposal ID should aligned with the replied comment.",
                );
                return false;
            }

            true
        };
        return validate_provided_doc(reply, "Comment", provider, report, reply_validator);
    }
    true
}

impl CommentDocument {
    /// Try to build `CommentDocument` from `CatalystSignedDoc` doing all necessary
    /// stateless verifications,
    pub(crate) fn from_signed_doc(
        doc: &CatalystSignedDocument, report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        if <Self as StatelessValidation>::validate(doc, report) {
            anyhow::bail!("Failed to build `CommentDocument` from `CatalystSignedDoc`");
        }

        let template = doc
            .doc_meta()
            .template()
            .ok_or(anyhow::anyhow!("missing `template` field"))?;
        let doc_ref = doc
            .doc_meta()
            .doc_ref()
            .ok_or(anyhow::anyhow!("missing `ref` field"))?;
        let reply = doc.doc_meta().reply();
        let section = doc.doc_meta().section().cloned();
        let content = serde_json::from_slice(doc.doc_content().decoded_bytes())?;

        Ok(Self {
            template,
            doc_ref,
            reply,
            section,
            content,
        })
    }
}

impl TryFrom<CatalystSignedDocument> for CommentDocument {
    type Error = CatalystSignedDocError;

    fn try_from(doc: CatalystSignedDocument) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Catalyst Signed Document to Comment Document");
        let res = Self::from_signed_doc(&doc, &error_report)
            .map_err(|e| CatalystSignedDocError::new(error_report, e))?;
        Ok(res)
    }
}
