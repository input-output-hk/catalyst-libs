//! Catalyst Signed Document Extra Fields.

use catalyst_types::{problem_report::ProblemReport, uuid::UuidV4};
use coset::{cbor::Value, Label, ProtectedHeader};

use super::{cose_protected_header_find, decode_cbor_uuid, encode_cbor_uuid, DocumentRef, Section};

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
/// `brand_id` field COSE key value
const BRAND_ID_KEY: &str = "brand_id";
/// `campaign_id` field COSE key value
const CAMPAIGN_ID_KEY: &str = "campaign_id";
/// `election_id` field COSE key value
const ELECTION_ID_KEY: &str = "election_id";
/// `category_id` field COSE key value
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
    /// Unique identifier for the brand that is running the voting.
    #[serde(skip_serializing_if = "Option::is_none")]
    brand_id: Option<DocumentRef>,
    /// Unique identifier for the campaign of voting.
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign_id: Option<DocumentRef>,
    /// Unique identifier for the election.
    #[serde(skip_serializing_if = "Option::is_none")]
    election_id: Option<UuidV4>,
    /// Unique identifier for the voting category as a collection of proposals.
    #[serde(skip_serializing_if = "Option::is_none")]
    category_id: Option<DocumentRef>,
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

    /// Return `brand_id` field.
    #[must_use]
    pub fn brand_id(&self) -> Option<DocumentRef> {
        self.brand_id
    }

    /// Return `campaign_id` field.
    #[must_use]
    pub fn campaign_id(&self) -> Option<DocumentRef> {
        self.campaign_id
    }

    /// Return `election_id` field.
    #[must_use]
    pub fn election_id(&self) -> Option<UuidV4> {
        self.election_id
    }

    /// Return `category_id` field.
    #[must_use]
    pub fn category_id(&self) -> Option<DocumentRef> {
        self.category_id
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
        if let Some(brand_id) = &self.brand_id {
            builder = builder.text_value(BRAND_ID_KEY.to_string(), Value::try_from(*brand_id)?);
        }

        if let Some(campaign_id) = &self.campaign_id {
            builder =
                builder.text_value(CAMPAIGN_ID_KEY.to_string(), Value::try_from(*campaign_id)?);
        }

        if let Some(election_id) = &self.election_id {
            builder =
                builder.text_value(ELECTION_ID_KEY.to_string(), encode_cbor_uuid(election_id)?);
        }

        if let Some(category_id) = &self.category_id {
            builder =
                builder.text_value(CATEGORY_ID_KEY.to_string(), Value::try_from(*category_id)?);
        }
        Ok(builder)
    }

    /// Converting COSE Protected Header to `ExtraFields`.
    /// Return `None` if it fails during
    #[allow(clippy::too_many_lines)]
    pub(crate) fn from_protected_header(
        protected: &ProtectedHeader, error_report: &ProblemReport,
    ) -> Self {
        /// Context for error messages.
        const CONTEXT: &str = "COSE ProtectedHeader to ExtraFields";
        let mut extra = ExtraFields::default();

        if let Some(cbor_doc_ref) =
            cose_protected_header_find(protected, |key| key == &Label::Text(REF_KEY.to_string()))
        {
            if let Ok(doc_ref) = DocumentRef::try_from(cbor_doc_ref) {
                extra.doc_ref = Some(doc_ref);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header doc ref",
                    &format!("{cbor_doc_ref:?}"),
                    "Expected a CBOR DocumentRef",
                    &format!("{CONTEXT}, DocumentRef"),
                );
            };
        }

        if let Some(cbor_doc_template) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(TEMPLATE_KEY.to_string())
        }) {
            if let Ok(doc_template) = DocumentRef::try_from(cbor_doc_template) {
                extra.template = Some(doc_template);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header document template",
                    &format!("{cbor_doc_template:?}"),
                    "Expected a CBOR DocumentRef",
                    &format!("{CONTEXT}, DocumentRef"),
                );
            }
        }

        if let Some(cbor_doc_reply) =
            cose_protected_header_find(protected, |key| key == &Label::Text(REPLY_KEY.to_string()))
        {
            if let Ok(doc_reply) = DocumentRef::try_from(cbor_doc_reply) {
                extra.reply = Some(doc_reply);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header document reply",
                    &format!("{cbor_doc_reply:?}"),
                    "Expected a CBOR DocumentRef",
                    &format!("{CONTEXT}, DocumentRef"),
                );
            }
        }

        if let Some(cbor_doc_section) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(SECTION_KEY.to_string())
        }) {
            if let Ok(section) = Section::try_from(cbor_doc_section) {
                extra.section = Some(section);
            } else {
                error_report.conversion_error(
                    "COSE protected header document section",
                    &format!("{cbor_doc_section:?}"),
                    "Must be a valid CBOR encoded String JSON Path",
                    &format!("{CONTEXT}, converting document section to String JSON Path"),
                );
            }
        }

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
                                &format!("{CONTEXT}, converting collaborator to String"),
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
                    &format!("{CONTEXT}, converting collaborators to Array"),
                );
            };
        }

        if let Some(cbor_doc_brand_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(BRAND_ID_KEY.to_string())
        }) {
            if let Ok(brand_id) = DocumentRef::try_from(cbor_doc_brand_id) {
                extra.brand_id = Some(brand_id);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header brand ID",
                    &format!("{cbor_doc_brand_id:?}"),
                    "Expected a CBOR UUID",
                    &format!("{CONTEXT}, decoding CBOR UUID for brand ID"),
                );
            }
        }

        if let Some(cbor_doc_campaign_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(CAMPAIGN_ID_KEY.to_string())
        }) {
            if let Ok(campaign_id) = DocumentRef::try_from(cbor_doc_campaign_id) {
                extra.campaign_id = Some(campaign_id);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header campaign ID",
                    &format!("{cbor_doc_campaign_id:?}"),
                    "Expected a CBOR UUID",
                    &format!("{CONTEXT}, decoding CBOR UUID for campaign ID"),
                );
            }
        }

        if let Some(cbor_doc_election_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(ELECTION_ID_KEY.to_string())
        }) {
            if let Ok(election_id) = decode_cbor_uuid(cbor_doc_election_id.clone()) {
                extra.election_id = Some(election_id);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header election ID",
                    &format!("{cbor_doc_election_id:?}"),
                    "Expected a CBOR UUID",
                    &format!("{CONTEXT}, decoding CBOR UUID for election ID"),
                );
            }
        }

        if let Some(cbor_doc_category_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(CATEGORY_ID_KEY.to_string())
        }) {
            if let Ok(category_id) = DocumentRef::try_from(cbor_doc_category_id) {
                extra.category_id = Some(category_id);
            } else {
                error_report.conversion_error(
                    "CBOR COSE protected header category ID",
                    &format!("{cbor_doc_category_id:?}"),
                    "Expected a CBOR UUID",
                    &format!("{CONTEXT}, decoding CBOR UUID for category ID"),
                );
            }
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
