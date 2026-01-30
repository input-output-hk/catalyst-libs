//! `content-encoding` rule type impl.

use std::string::ToString;

use catalyst_signed_doc_spec::is_required::IsRequired;

use crate::{
    CatalystSignedDocument, metadata::ContentEncoding, providers::Provider,
    validator::CatalystSignedDocumentValidationRule,
};

/// `content-encoding` field validation rule.
#[derive(Debug)]
pub(crate) enum ContentEncodingRule {
    /// Content Encoding field is optionally present in the document.
    Specified {
        /// expected `content-encoding` field.
        exp: Vec<ContentEncoding>,
        /// optional flag for the `content-encoding` field.
        optional: bool,
    },
    /// Content Encoding field must not be present in the document.
    NotSpecified,
}

impl CatalystSignedDocumentValidationRule for ContentEncodingRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        self.check_inner(doc);
        Ok(!doc.report().is_problematic())
    }
}

impl ContentEncodingRule {
    /// Create a new rule from specs.
    pub(crate) fn new(
        spec: &catalyst_signed_doc_spec::headers::content_encoding::ContentEncoding
    ) -> anyhow::Result<Self> {
        if let IsRequired::Excluded = spec.required {
            anyhow::ensure!(
                spec.value.is_none(),
                "'content type' field must not exist when 'required' is 'excluded'"
            );
            return Ok(Self::NotSpecified);
        }

        let optional = IsRequired::Optional == spec.required;

        let exp = spec
            .value
            .as_ref()
            .ok_or(anyhow::anyhow!("'content-encoding' field must have value "))?
            .iter()
            .flat_map(|encoding| encoding.parse())
            .collect();

        Ok(Self::Specified { exp, optional })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) {
        let context = "Content Encoding Rule check";
        match self {
            Self::NotSpecified => {
                if let Some(content_encoding) = doc.doc_content_encoding() {
                    doc.report().unknown_field(
                        "content-encoding",
                        &content_encoding.to_string(),
                        &format!(
                            "{context}, document does not expect to have a content-encoding field"
                        ),
                    );
                }
            },
            Self::Specified { exp, optional } => {
                if let Some(content_encoding) = doc.doc_content_encoding() {
                    if !exp.contains(&content_encoding) {
                        doc.report().invalid_value(
                            "content-encoding",
                            content_encoding.to_string().as_str(),
                            &exp.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", "),
                            "Invalid document content-encoding value",
                        );
                    }
                    if content_encoding.decode(doc.encoded_content()).is_err() {
                        doc.report().invalid_value(
                            "payload",
                            &hex::encode(doc.encoded_content()),
                            content_encoding.to_string().as_str(),
                            "Document content is not decodable with the expected content-encoding",
                        );
                    }
                } else if !optional {
                    doc.report().missing_field(
                        "content-encoding",
                        "Document must have a content-encoding field",
                    );
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[test]
    fn content_encoding_is_specified_rule_test() {
        let content_encoding = ContentEncoding::Brotli;

        let rule = ContentEncodingRule::Specified {
            exp: vec![content_encoding],
            optional: true,
        };

        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .with_content(content_encoding.encode(&[1, 2, 3]).unwrap())
            .build();
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());

        // empty content (empty bytes) could not be brotli decoded
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        rule.check_inner(&doc);
        let report = format!("{:?}", doc.report());
        println!("{report}");
        assert!(doc.report().is_problematic());
        assert!(
            report.contains("Document content is not decodable with the expected content-encoding")
        );

        let doc = Builder::with_required_fields().build();
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());

        let rule = ContentEncodingRule::Specified {
            exp: vec![content_encoding],
            optional: false,
        };
        rule.check_inner(&doc);
        let report = format!("{:?}", doc.report());
        println!("{report}");
        assert!(doc.report().is_problematic());
        assert!(report.contains("Document must have a content-encoding field"));
    }

    #[test]
    fn content_encoding_is_not_specified_rule_test() {
        let content_encoding = ContentEncoding::Brotli;

        let rule = ContentEncodingRule::NotSpecified;

        // With Brotli content encoding
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::ContentEncoding(content_encoding))
            .build();
        rule.check_inner(&doc);
        let report = format!("{:?}", doc.report());
        println!("{report}");
        assert!(doc.report().is_problematic());
        assert!(report.contains("document does not expect to have a content-encoding field"));

        // No content encoding
        let doc = Builder::with_required_fields().build();
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());
    }
}
