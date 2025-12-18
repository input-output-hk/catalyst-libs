//! `content-type` rule type impl.

#[cfg(test)]
mod tests;

use catalyst_types::json_schema::JsonSchema;

use crate::{
    CatalystSignedDocument, metadata::ContentType, providers::Provider,
    validator::CatalystSignedDocumentValidationRule,
};

/// `content-type` field validation rule
#[derive(Debug)]
pub(crate) enum ContentTypeRule {
    /// Content Type field must be present with the specific type in the document.
    Specified {
        /// expected `content-type` field
        exp: ContentType,
    },
    /// Content Type field must not be present in the document.
    NotSpecified,
}

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for ContentTypeRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl ContentTypeRule {
    /// Generating `ContentTypeRule` from specs
    pub(crate) fn new(
        spec: &catalyst_signed_doc_spec::headers::content_type::ContentType
    ) -> anyhow::Result<Self> {
        if let catalyst_signed_doc_spec::is_required::IsRequired::Excluded = spec.required {
            anyhow::ensure!(
                spec.value.is_none(),
                "'value' field must not exist when 'required' is 'excluded'"
            );
            return Ok(Self::NotSpecified);
        }

        anyhow::ensure!(
            catalyst_signed_doc_spec::is_required::IsRequired::Optional != spec.required,
            "'content type' field cannot been optional"
        );

        let value = spec
            .value
            .as_ref()
            .ok_or(anyhow::anyhow!("'content type' 'value' field must exist"))?;

        Ok(Self::Specified {
            exp: value.parse()?,
        })
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        if let Self::NotSpecified = &self
            && let Some(content_type) = doc.doc_content_type()
        {
            doc.report().unknown_field(
                "content-type",
                content_type.to_string().as_str(),
                "document does not expect to have the content type field",
            );
            return false;
        }
        if let Self::Specified { exp } = &self {
            let Some(content_type) = doc.doc_content_type() else {
                doc.report().missing_field(
                    "content-type",
                    "Cannot get a content type field during the field validation",
                );
                return false;
            };

            if content_type != *exp {
                doc.report().invalid_value(
                    "content-type",
                    content_type.to_string().as_str(),
                    exp.to_string().as_str(),
                    "Invalid Document content-type value",
                );
                return false;
            }
            let Ok(content) = doc.decoded_content() else {
                doc.report().functional_validation(
                    "Invalid Document content, cannot get decoded bytes",
                    "Cannot get a document content during the content type field validation",
                );
                return false;
            };
            if !validate(*exp, &content) {
                doc.report().invalid_value(
                    "payload",
                    &hex::encode(content),
                    &format!("Invalid Document content, should {content_type} encodable"),
                    "Invalid Document content",
                );
                return false;
            }
        }
        true
    }
}

/// Validates the provided `content` bytes to be a defined `ContentType`.
fn validate(
    content_type: ContentType,
    content: &[u8],
) -> bool {
    match content_type {
        ContentType::Json => {
            serde_json::from_slice::<&serde_json::value::RawValue>(content).is_ok()
        },
        ContentType::Cbor => {
            let mut decoder = minicbor::Decoder::new(content);
            decoder.skip().is_ok() && decoder.position() == content.len()
        },
        ContentType::SchemaJson => {
            let Ok(template_json_schema) = serde_json::from_slice(content) else {
                return false;
            };
            JsonSchema::try_from(&template_json_schema).is_ok()
        },
        ContentType::Cddl
        | ContentType::Css
        | ContentType::CssHandlebars
        | ContentType::Html
        | ContentType::HtmlHandlebars
        | ContentType::Markdown
        | ContentType::MarkdownHandlebars
        | ContentType::Plain
        | ContentType::PlainHandlebars => {
            // TODO: not implemented yet
            false
        },
    }
}
