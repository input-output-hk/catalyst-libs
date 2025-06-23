//! Parameter linked reference rule impl.

use crate::{
    providers::CatalystSignedDocumentProvider, validator::utils::validate_doc_refs,
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

            return validate_doc_refs(
                param_link_ref,
                provider,
                doc.report(),
                param_link_ref_validator,
            )
            .await;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::uuid::{UuidV4, UuidV7};

    use crate::{
        builder::tests::Builder,
        metadata::SupportedField,
        providers::tests::TestCatalystSignedDocumentProvider,
        validator::rules::param_link_ref::{LinkField, ParameterLinkRefRule},
        DocLocator, DocumentRef,
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
                .with_metadata_field(SupportedField::Id(doc1_id))
                .with_metadata_field(SupportedField::Ver(doc1_ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .with_metadata_field(SupportedField::Parameters(
                    vec![
                        DocumentRef::new(category_id, category_ver, DocLocator::default()),
                        DocumentRef::new(campaign_id, campaign_ver, DocLocator::default()),
                    ]
                    .into(),
                ))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Doc being referenced - parameter does not match
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(doc2_id))
                .with_metadata_field(SupportedField::Ver(doc2_ver))
                .with_metadata_field(SupportedField::Type(doc_type.into()))
                .with_metadata_field(SupportedField::Parameters(
                    vec![DocumentRef::new(
                        campaign_id,
                        campaign_ver,
                        DocLocator::default(),
                    )]
                    .into(),
                ))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Category doc
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(category_id))
                .with_metadata_field(SupportedField::Ver(category_ver))
                .with_metadata_field(SupportedField::Type(category_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();

            // Campaign doc
            let doc = Builder::new()
                .with_metadata_field(SupportedField::Id(campaign_id))
                .with_metadata_field(SupportedField::Ver(campaign_ver))
                .with_metadata_field(SupportedField::Type(campaign_type.into()))
                .build();
            provider.add_document(None, &doc).unwrap();
        }

        // Use Ref as a linked reference
        let rule = ParameterLinkRefRule::Specified {
            field: LinkField::Ref,
        };
        // Parameter must match
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(doc1_id, doc1_ver, DocLocator::default())].into(),
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![
                    DocumentRef::new(category_id, category_ver, DocLocator::default()),
                    DocumentRef::new(campaign_id, campaign_ver, DocLocator::default()),
                ]
                .into(),
            ))
            .build();
        assert!(rule.check(&doc, &provider).await.unwrap());

        // Parameter does not match
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Ref(
                vec![DocumentRef::new(doc2_id, doc2_ver, DocLocator::default())].into(),
            ))
            .with_metadata_field(SupportedField::Parameters(
                vec![
                    DocumentRef::new(category_id, category_ver, DocLocator::default()),
                    DocumentRef::new(campaign_id, campaign_ver, DocLocator::default()),
                ]
                .into(),
            ))
            .build();
        assert!(!rule.check(&doc, &provider).await.unwrap());
    }
}
