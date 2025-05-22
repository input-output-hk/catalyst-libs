//! Catalyst Signed Document Extra Fields.

use catalyst_types::problem_report::ProblemReport;
use coset::{cbor::Value, Label, ProtectedHeader};

use super::{
    cose_protected_header_find, utils::decode_document_field_from_protected_header, DocumentRef,
    Section,
};

/// `ref` field COSE key value
const REF_KEY: &str = "ref";
/// `template` field COSE key value
const TEMPLATE_KEY: &str = "template";
/// `reply` field COSE key value
const REPLY_KEY: &str = "reply";
/// `section` field COSE key value
const SECTION_KEY: &str = "section";
/// `collabs` field COSE key value
const COLLABS_KEY: &str = "collabs";
/// `parameters` field COSE key value
const PARAMETERS_KEY: &str = "parameters";
/// `brand_id` field COSE key value (alias of the `parameters` field)
const BRAND_ID_KEY: &str = "brand_id";
/// `campaign_id` field COSE key value (alias of the `parameters` field)
const CAMPAIGN_ID_KEY: &str = "campaign_id";
/// `category_id` field COSE key value (alias of the `parameters` field)
const CATEGORY_ID_KEY: &str = "category_id";

/// Extra Metadata Fields.
///
/// These values are extracted from the COSE Sign protected header labels.
#[derive(Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExtraFields {
    /// Reference to the latest document.
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    doc_ref: Option<DocumentRef>,
    /// Reference to the document template.
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<DocumentRef>,
    /// Reference to the document reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<DocumentRef>,
    /// Reference to the document section.
    #[serde(skip_serializing_if = "Option::is_none")]
    section: Option<Section>,
    /// Reference to the document collaborators. Collaborator type is TBD.
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    collabs: Vec<String>,

    /// Reference to the parameters document.
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<DocumentRef>,
    // TODO: get rid of these fields after `CatalystSignedDocument` would preserve original cbor
    // bytes
    /// Reference to the parameters document.
    #[serde(skip_serializing_if = "Option::is_none")]
    category_id: Option<DocumentRef>,
    /// Reference to the parameters document.
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign_id: Option<DocumentRef>,
    /// Reference to the parameters document.
    #[serde(skip_serializing_if = "Option::is_none")]
    brand_id: Option<DocumentRef>,
}

impl ExtraFields {
    /// Return `ref` field.
    #[must_use]
    pub fn doc_ref(&self) -> Option<DocumentRef> {
        self.doc_ref
    }

    /// Return `template` field.
    #[must_use]
    pub fn template(&self) -> Option<DocumentRef> {
        self.template
    }

    /// Return `reply` field.
    #[must_use]
    pub fn reply(&self) -> Option<DocumentRef> {
        self.reply
    }

    /// Return `section` field.
    #[must_use]
    pub fn section(&self) -> Option<&Section> {
        self.section.as_ref()
    }

    /// Return `collabs` field.
    #[must_use]
    pub fn collabs(&self) -> &Vec<String> {
        &self.collabs
    }

    /// Return `parameters` field.
    #[must_use]
    pub fn parameters(&self) -> Option<DocumentRef> {
        self.parameters
            .or(self.category_id)
            .or(self.campaign_id)
            .or(self.brand_id)
    }

    /// Fill the COSE header `ExtraFields` data into the header builder.
    pub(super) fn fill_cose_header_fields(
        &self, mut builder: coset::HeaderBuilder,
    ) -> anyhow::Result<coset::HeaderBuilder> {
        if let Some(doc_ref) = &self.doc_ref {
            builder = builder.text_value(REF_KEY.to_string(), Value::try_from(*doc_ref)?);
        }
        if let Some(template) = &self.template {
            builder = builder.text_value(TEMPLATE_KEY.to_string(), Value::try_from(*template)?);
        }
        if let Some(reply) = &self.reply {
            builder = builder.text_value(REPLY_KEY.to_string(), Value::try_from(*reply)?);
        }

        if let Some(section) = &self.section {
            builder = builder.text_value(SECTION_KEY.to_string(), Value::from(section.clone()));
        }

        if !self.collabs.is_empty() {
            builder = builder.text_value(
                COLLABS_KEY.to_string(),
                Value::Array(self.collabs.iter().cloned().map(Value::Text).collect()),
            );
        }

        if let Some(parameters) = &self.parameters {
            builder = builder.text_value(PARAMETERS_KEY.to_string(), Value::try_from(*parameters)?);
        }
        if let Some(category_id) = &self.category_id {
            builder =
                builder.text_value(CATEGORY_ID_KEY.to_string(), Value::try_from(*category_id)?);
        }
        if let Some(campaign_id) = &self.campaign_id {
            builder =
                builder.text_value(CAMPAIGN_ID_KEY.to_string(), Value::try_from(*campaign_id)?);
        }
        if let Some(brand_id) = &self.brand_id {
            builder = builder.text_value(BRAND_ID_KEY.to_string(), Value::try_from(*brand_id)?);
        }

        Ok(builder)
    }

    /// Converting COSE Protected Header to `ExtraFields`.
    pub(crate) fn from_protected_header(
        protected: &ProtectedHeader, error_report: &ProblemReport,
    ) -> Self {
        /// Context for problem report messages during decoding from COSE protected
        /// header.
        const COSE_DECODING_CONTEXT: &str = "COSE ProtectedHeader to ExtraFields";

        let doc_ref = decode_document_field_from_protected_header(
            protected,
            REF_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let template = decode_document_field_from_protected_header(
            protected,
            TEMPLATE_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let reply = decode_document_field_from_protected_header(
            protected,
            REPLY_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let section = decode_document_field_from_protected_header(
            protected,
            SECTION_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );

        // process `parameters` field and all its aliases
        let parameters = decode_document_field_from_protected_header(
            protected,
            PARAMETERS_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let category_id = decode_document_field_from_protected_header(
            protected,
            CATEGORY_ID_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let campaign_id = decode_document_field_from_protected_header(
            protected,
            CAMPAIGN_ID_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );
        let brand_id = decode_document_field_from_protected_header(
            protected,
            BRAND_ID_KEY,
            COSE_DECODING_CONTEXT,
            error_report,
        );

        if [&parameters, &category_id, &campaign_id, &brand_id]
            .iter()
            .filter(|v| v.is_some())
            .count()
            > 1
        {
            error_report.duplicate_field(
                    "brand_id, campaign_id, category_id", 
                    "Only value at the same time is allowed parameters, brand_id, campaign_id, category_id", 
                    "Validation of parameters field aliases"
                );
        }

        let mut extra = ExtraFields {
            doc_ref,
            template,
            reply,
            section,
            parameters,
            category_id,
            campaign_id,
            brand_id,
            ..Default::default()
        };

        if let Some(cbor_doc_collabs) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(COLLABS_KEY.to_string())
        }) {
            if let Ok(collabs) = cbor_doc_collabs.clone().into_array() {
                let mut c = Vec::new();
                for (ids, collaborator) in collabs.iter().cloned().enumerate() {
                    match collaborator.clone().into_text() {
                        Ok(collaborator) => {
                            c.push(collaborator);
                        },
                        Err(_) => {
                            error_report.conversion_error(
                                &format!("COSE protected header collaborator index {ids}"),
                                &format!("{collaborator:?}"),
                                "Expected a CBOR String",
                                &format!(
                                    "{COSE_DECODING_CONTEXT}, converting collaborator to String",
                                ),
                            );
                        },
                    }
                }
                extra.collabs = c;
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header collaborators",
                    &format!("{cbor_doc_collabs:?}"),
                    "Expected a CBOR Array",
                    &format!("{COSE_DECODING_CONTEXT}, converting collaborators to Array",),
                );
            };
        }

        extra
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_extra_fields_json_serde_test() {
        let extra = ExtraFields::default();

        let json = serde_json::to_value(extra).unwrap();
        assert_eq!(json, serde_json::json!({}));
    }
}
