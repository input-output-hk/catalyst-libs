//! Catalyst Signed Document Additional Fields.

use anyhow::anyhow;

use super::{cose_protected_header_find, DocumentRef};

/// Additional Metadata Fields.
///
/// These values are extracted from the COSE Sign protected header labels.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct AdditionalFields {
    /// Reference to the latest document.
    #[serde(rename = "ref")]
    pub(super) doc_ref: Option<DocumentRef>,
    /// Reference to the document template.
    pub(super) template: Option<DocumentRef>,
    /// Reference to the document reply.
    pub(super) reply: Option<DocumentRef>,
    /// Reference to the document section.
    pub(super) section: Option<String>,
}

impl TryFrom<&coset::ProtectedHeader> for AdditionalFields {
    type Error = Vec<anyhow::Error>;

    fn try_from(protected: &coset::ProtectedHeader) -> Result<Self, Self::Error> {
        let mut extra = AdditionalFields::default();
        let mut errors = Vec::new();

        if let Some(cbor_doc_ref) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("ref".to_string())
        }) {
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
            key == &coset::Label::Text("template".to_string())
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

        if let Some(cbor_doc_reply) = cose_protected_header_find(protected, |key| {
            key == &coset::Label::Text("reply".to_string())
        }) {
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
            key == &coset::Label::Text("section".to_string())
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

        if errors.is_empty() {
            Ok(extra)
        } else {
            Err(errors)
        }
    }
}
