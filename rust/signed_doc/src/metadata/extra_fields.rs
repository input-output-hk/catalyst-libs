//! Catalyst Signed Document Extra Fields.

use anyhow::anyhow;
use coset::{cbor::Value, Label, ProtectedHeader};

use super::{cose_protected_header_find, decode_cbor_uuid, encode_cbor_uuid, DocumentRef, UuidV4};

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
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExtraFields {
    /// Reference to the latest document.
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub(super) doc_ref: Option<DocumentRef>,
    /// Reference to the document template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) template: Option<DocumentRef>,
    /// Reference to the document reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reply: Option<DocumentRef>,
    /// Reference to the document section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) section: Option<String>,
    /// Reference to the document collaborators. Collaborator type is TBD.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) collabs: Vec<String>,
    /// Unique identifier for the brand that is running the voting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) brand_id: Option<UuidV4>,
    /// Unique identifier for the campaign of voting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) campaign_id: Option<UuidV4>,
    /// Unique identifier for the election.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) election_id: Option<UuidV4>,
    /// Unique identifier for the voting category as a collection of proposals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) category_id: Option<UuidV4>,
}

impl ExtraFields {
    /// Fill the COSE hedear `ExtraFields` data into the header builder.
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
            builder = builder.text_value(SECTION_KEY.to_string(), Value::Text(section.clone()));
        }

        if !self.collabs.is_empty() {
            builder = builder.text_value(
                COLLABS_KEY.to_string(),
                Value::Array(self.collabs.iter().cloned().map(Value::Text).collect()),
            );
        }
        if let Some(brand_id) = &self.brand_id {
            builder = builder.text_value(BRAND_ID_KEY.to_string(), encode_cbor_uuid(brand_id)?);
        }

        if let Some(campaign_id) = &self.campaign_id {
            builder =
                builder.text_value(CAMPAIGN_ID_KEY.to_string(), encode_cbor_uuid(campaign_id)?);
        }

        if let Some(election_id) = &self.election_id {
            builder =
                builder.text_value(ELECTION_ID_KEY.to_string(), encode_cbor_uuid(election_id)?);
        }

        if let Some(category_id) = &self.category_id {
            builder =
                builder.text_value(CATEGORY_ID_KEY.to_string(), encode_cbor_uuid(*category_id)?);
        }
        Ok(builder)
    }
}

impl TryFrom<&ProtectedHeader> for ExtraFields {
    type Error = crate::error::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(protected: &ProtectedHeader) -> Result<Self, Self::Error> {
        let mut extra = ExtraFields::default();
        let mut errors = Vec::new();

        if let Some(cbor_doc_ref) =
            cose_protected_header_find(protected, |key| key == &Label::Text(REF_KEY.to_string()))
        {
            match DocumentRef::try_from(cbor_doc_ref) {
                Ok(doc_ref) => {
                    extra.doc_ref = Some(doc_ref);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `ref` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_template) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(TEMPLATE_KEY.to_string())
        }) {
            match DocumentRef::try_from(cbor_doc_template) {
                Ok(doc_template) => {
                    extra.template = Some(doc_template);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `template` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_reply) =
            cose_protected_header_find(protected, |key| key == &Label::Text(REPLY_KEY.to_string()))
        {
            match DocumentRef::try_from(cbor_doc_reply) {
                Ok(doc_reply) => {
                    extra.reply = Some(doc_reply);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `reply` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_section) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(SECTION_KEY.to_string())
        }) {
            match cbor_doc_section.clone().into_text() {
                Ok(doc_section) => {
                    extra.section = Some(doc_section);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `section` field, err: {e:?}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_collabs) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(COLLABS_KEY.to_string())
        }) {
            match cbor_doc_collabs.clone().into_array() {
                Ok(collabs) => {
                    let mut c = Vec::new();
                    for (ids, collaborator) in collabs.iter().cloned().enumerate() {
                        match collaborator.into_text() {
                            Ok(collaborator) => {
                                c.push(collaborator);
                            },
                            Err(e) => {
                                errors.push(anyhow!(
                                    "Invalid Collaborator at index {ids} of COSE protected header `collabs` field, err: {e:?}"
                                ));
                            },
                        }
                    }
                    extra.collabs = c;
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `collabs` field, err: {e:?}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_brand_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(BRAND_ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(cbor_doc_brand_id.clone()) {
                Ok(brand_id) => {
                    extra.brand_id = Some(brand_id);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `brand_id` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_campaign_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(CAMPAIGN_ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(cbor_doc_campaign_id.clone()) {
                Ok(campaign_id) => {
                    extra.campaign_id = Some(campaign_id);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `campaign_id` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_election_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(ELECTION_ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(cbor_doc_election_id.clone()) {
                Ok(election_id) => {
                    extra.election_id = Some(election_id);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `election_id` field, err: {e}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_category_id) = cose_protected_header_find(protected, |key| {
            key == &Label::Text(CATEGORY_ID_KEY.to_string())
        }) {
            match decode_cbor_uuid(cbor_doc_category_id.clone()) {
                Ok(category_id) => {
                    extra.category_id = Some(category_id);
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `category_id` field, err: {e}"
                    ));
                },
            }
        }

        if errors.is_empty() {
            Ok(extra)
        } else {
            Err(errors.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{V4, V7};

    use super::*;

    #[test]
    fn extra_fields_json_serde_test() {
        let extra = ExtraFields::default();

        let json = serde_json::to_value(extra).unwrap();
        assert_eq!(json, serde_json::json!({}));

        let uuid_v7 = V7::new();
        let uuid_v4 = V4::new();
        let section = "some section".to_string();
        let collabs = vec!["collab1".to_string(), "collab2".to_string()];
        let extra = ExtraFields {
            doc_ref: Some(DocumentRef::Latest { id: uuid_v7 }),
            reply: Some(DocumentRef::WithVer {
                id: uuid_v7,
                ver: uuid_v7,
            }),
            template: Some(DocumentRef::Latest { id: uuid_v7 }),
            section: Some(section.clone()),
            collabs: collabs.clone(),
            campaign_id: Some(uuid_v4),
            election_id: Some(uuid_v4),
            brand_id: Some(uuid_v4),
            category_id: Some(uuid_v4),
        };

        let json = serde_json::to_value(extra).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "ref": {"id": uuid_v7.to_string()},
                "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
                "template": {"id": uuid_v7.to_string()},
                "section": section,
                "collabs": collabs,
                "campaign_id": uuid_v4.to_string(),
                "election_id":  uuid_v4.to_string(),
                "brand_id":  uuid_v4.to_string(),
                "category_id": uuid_v4.to_string(),
            })
        );
    }
}
