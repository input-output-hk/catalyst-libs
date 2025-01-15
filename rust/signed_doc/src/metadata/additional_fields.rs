//! Catalyst Signed Document Additional Fields.

use anyhow::anyhow;
use coset::{cbor::Value, Label, ProtectedHeader};

use super::{cose_protected_header_find, decode_cbor_uuid, encode_cbor_value, DocumentRef, UuidV4};

/// Additional Metadata Fields.
///
/// These values are extracted from the COSE Sign protected header labels.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdditionalFields {
    /// Reference to the latest document.
    #[serde(rename = "ref")]
    pub(super) doc_ref: Option<DocumentRef>,
    /// Reference to the document template.
    pub(super) template: Option<DocumentRef>,
    /// Reference to the document reply.
    pub(super) reply: Option<DocumentRef>,
    /// Reference to the document section.
    pub(super) section: Option<String>,
    /// Reference to the document collaborators. Collaborator type is TBD.
    pub(super) collabs: Option<Vec<String>>,
    /// Unique identifier for the brand that is running the voting.
    pub(super) brand_id: Option<UuidV4>,
    /// Unique identifier for the campaign of voting.
    pub(super) campaign_id: Option<UuidV4>,
    /// Unique identifier for the election.
    pub(super) election_id: Option<UuidV4>,
    /// Unique identifier for the voting category as a collection of proposals.
    pub(super) category_id: Option<UuidV4>,
}

impl AdditionalFields {
    /// Returns the COSE Sign protected header REST fields.
    ///
    /// # Errors
    /// If any internal field cannot be converted into `Value`.
    pub fn header_rest(&self) -> anyhow::Result<Vec<(Label, Value)>> {
        self.try_into()
    }
}

impl TryFrom<&AdditionalFields> for Vec<(Label, Value)> {
    type Error = anyhow::Error;

    fn try_from(fields: &AdditionalFields) -> anyhow::Result<Self> {
        let mut vec = Vec::new();

        if let Some(doc_ref) = &fields.doc_ref {
            vec.push((Label::Text("ref".to_string()), doc_ref.try_into()?));
        }

        if let Some(template) = &fields.template {
            vec.push((Label::Text("template".to_string()), template.try_into()?));
        }

        if let Some(reply) = &fields.reply {
            vec.push((Label::Text("reply".to_string()), reply.try_into()?));
        }

        if let Some(section) = &fields.section {
            vec.push((
                Label::Text("section".to_string()),
                Value::Text(section.clone()),
            ));
        }

        if let Some(collabs) = &fields.collabs {
            if !collabs.is_empty() {
                vec.push((
                    Label::Text("collabs".to_string()),
                    Value::Array(collabs.iter().cloned().map(Value::Text).collect()),
                ));
            }
        }

        if let Some(brand_id) = &fields.brand_id {
            vec.push((
                Label::Text("brand_id".to_string()),
                encode_cbor_value(brand_id)?,
            ));
        }

        if let Some(campaign_id) = &fields.campaign_id {
            vec.push((
                Label::Text("campaign_id".to_string()),
                encode_cbor_value(campaign_id)?,
            ));
        }

        if let Some(election_id) = &fields.election_id {
            vec.push((
                Label::Text("election_id".to_string()),
                encode_cbor_value(election_id)?,
            ));
        }

        if let Some(category_id) = &fields.category_id {
            vec.push((
                Label::Text("category_id".to_string()),
                encode_cbor_value(*category_id)?,
            ));
        }

        Ok(vec)
    }
}

impl TryFrom<&ProtectedHeader> for AdditionalFields {
    type Error = crate::error::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(protected: &ProtectedHeader) -> Result<Self, Self::Error> {
        let mut extra = AdditionalFields::default();
        let mut errors = Vec::new();

        if let Some(cbor_doc_ref) =
            cose_protected_header_find(protected, |key| key == &Label::Text("ref".to_string()))
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

        if let Some(cbor_doc_template) =
            cose_protected_header_find(protected, |key| key == &Label::Text("template".to_string()))
        {
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
            cose_protected_header_find(protected, |key| key == &Label::Text("reply".to_string()))
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

        if let Some(cbor_doc_section) =
            cose_protected_header_find(protected, |key| key == &Label::Text("section".to_string()))
        {
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

        if let Some(cbor_doc_collabs) =
            cose_protected_header_find(protected, |key| key == &Label::Text("collabs".to_string()))
        {
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

                    if !c.is_empty() {
                        extra.collabs = Some(c);
                    }
                },
                Err(e) => {
                    errors.push(anyhow!(
                        "Invalid COSE protected header `collabs` field, err: {e:?}"
                    ));
                },
            }
        }

        if let Some(cbor_doc_brand_id) =
            cose_protected_header_find(protected, |key| key == &Label::Text("brand_id".to_string()))
        {
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
            key == &Label::Text("campaign_id".to_string())
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
            key == &Label::Text("election_id".to_string())
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
            key == &Label::Text("category_id".to_string())
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
