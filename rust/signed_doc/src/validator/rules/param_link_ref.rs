//! Parameter linked reference rule impl.

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_provided_doc,
    CatalystSignedDocument,
};

/// Filed that is being used for linked ref
pub(crate) enum LinkField {
    /// Ref field
    Ref,
    /// Template field
    Template,
}

/// Parameter Link reference validation rule
pub(crate) enum ParameterLinkRefRule {
    /// Link ref specified
    Specified {
        /// Filed that is being used for linked ref
        field: LinkField,
    },
    /// Link ref is not specified
    #[allow(dead_code)]
    NotSpecified,
}

impl ParameterLinkRefRule {
    /// Validation rule
    pub(crate) async fn check<Provider>(
        &self, doc: &CatalystSignedDocument, provider: &Provider,
    ) -> anyhow::Result<bool>
    where Provider: CatalystSignedDocumentProvider {
        if let Self::Specified { field } = self {
            let param_link_ref_validator = |ref_doc: CatalystSignedDocument| {
                // The parameters MUST be the same
                doc.doc_meta().parameters() == ref_doc.doc_meta().parameters()
            };

            // Which field is use for linked reference
            let param_link_ref = match field {
                LinkField::Ref => doc.doc_meta().doc_ref(),
                LinkField::Template => doc.doc_meta().template(),
            };

            let Some(param_link_ref) = param_link_ref else {
                doc.report()
                    .missing_field("Link ref", "Invalid link reference");
                return Ok(false);
            };

            for dr in param_link_ref.doc_refs() {
                let result =
                    validate_provided_doc(dr, provider, doc.report(), param_link_ref_validator)
                        .await?;
                if !result {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use crate::{
        providers::tests::TestCatalystSignedDocumentProvider,
        validator::rules::param_link_ref::{LinkField, ParameterLinkRefRule},
        Builder,
    };
    #[tokio::test]
    async fn param_link_ref_specified_test() {
        let mut provider = TestCatalystSignedDocumentProvider::default();

        let doc1_id = UuidV7::new();
        let doc1_ver = UuidV7::new();
        let doc2_id = UuidV7::new();
        let doc2_ver = UuidV7::new();

        let doc_type = UuidV4::new();

        let category_id = UuidV7::new();
        let category_ver = UuidV7::new();
        let category_type = UuidV4::new();

        let campaign_id = UuidV7::new();
        let campaign_ver = UuidV7::new();
        let campaign_type = UuidV4::new();

        // Prepare provider documents
        {
            // Doc being referenced - parameter MUST match
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": doc1_id.to_string(),
                    "ver": doc1_ver.to_string(),
                    "type": doc_type.to_string(),
                    "parameters": [{"id": category_id.to_string(), "ver": category_ver.to_string(), "cid": "0x" }, {"id": campaign_id.to_string(), "ver": campaign_ver.to_string(), "cid": "0x" }]
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Doc being referenced - parameter does not match
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": doc2_id.to_string(),
                    "ver": doc2_ver.to_string(),
                    "type": doc_type.to_string(),
                    "parameters": [{"id": campaign_id.to_string(), "ver": campaign_ver.to_string(),  "cid": "0x" }]
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Category doc
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": category_id.to_string(),
                    "ver": category_ver.to_string(),
                    "type": category_type.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();

            // Campaign doc
            let doc = Builder::new()
                .with_json_metadata(serde_json::json!({
                    "id": campaign_id.to_string(),
                    "ver": campaign_ver.to_string(),
                    "type": campaign_type.to_string(),
                }))
                .unwrap()
                .build();
            provider.add_document(None, &doc).unwrap();
        }

        // Use Ref as a linked reference
        let rule = ParameterLinkRefRule::Specified {
            field: LinkField::Ref,
        };
        // Parameter must match
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": [{"id": doc1_id.to_string(), "ver": doc1_ver.to_string(), "cid": "0x" }],
                "parameters":
                [{"id": category_id.to_string(), "ver": category_ver.to_string(), "cid": "0x" }, {"id": campaign_id.to_string(), "ver": campaign_ver.to_string(), "cid": "0x" }]
            }))
            .unwrap()
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Parameter does not match
        let doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "ref": {"id": doc2_id.to_string(), "ver": doc2_ver.to_string()},
                "parameters":
                [{"id": category_id.to_string(), "ver": category_ver.to_string(), "cid": "0x" }, {"id": campaign_id.to_string(), "ver": campaign_ver.to_string(), "cid": "0x" }]
            }))
            .unwrap()
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
