//! `collaborators` rule type impl.

use crate::CatalystSignedDocument;

/// `collaborators` field validation rule
#[derive(Debug)]
pub(crate) enum CollaboratorsRule {
    /// Is 'collaborators' specified
    #[allow(dead_code)]
    Specified {
        /// optional flag for the `collaborators` field
        optional: bool,
    },
    /// 'collaborators' is not specified
    NotSpecified,
}

impl CollaboratorsRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        if let Self::Specified { optional } = self {
            if doc.doc_meta().collaborators().is_empty() && !optional {
                doc.report().missing_field(
                    "collaborators",
                    "Document must have at least one entry in 'collaborators' field",
                );
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if !doc.doc_meta().collaborators().is_empty() {
                doc.report().unknown_field(
                    "collaborators",
                    &format!(
                        "{:#?}",
                        doc.doc_meta()
                            .collaborators()
                            .iter()
                            .map(ToString::to_string)
                            .reduce(|a, b| format!("{a}, {b}"))
                    ),
                    "Document does not expect to have a 'collaborators' field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_types::catalyst_id::role_index::RoleId;
    use test_case::test_case;

    use super::*;
    use crate::{
        builder::tests::Builder, metadata::SupportedField,
        validator::rules::utils::create_dummy_key_pair,
    };

    #[test_case(
        || {
            Builder::new()
                .with_metadata_field(SupportedField::Collaborators(
                        vec![create_dummy_key_pair(RoleId::Role0).2].into()
                    ))
                .build()
        }
        => true
        ;
        "valid 'collaborators' field present"
    )]
    #[test_case(
        || {
            Builder::new().build()
        }
        => true
        ;
        "missing 'collaborators' field"
    )]
    #[tokio::test]
    async fn section_rule_specified_optional_test(
        doc_gen: impl FnOnce() -> CatalystSignedDocument
    ) -> bool {
        let rule = CollaboratorsRule::Specified { optional: true };

        let doc = doc_gen();
        rule.check(&doc).await.unwrap()
    }

    #[test_case(
        || {
            Builder::new()
                .with_metadata_field(SupportedField::Collaborators(
                        vec![create_dummy_key_pair(RoleId::Role0).2].into()
                    ))
                .build()
        }
        => true
        ;
        "valid 'collaborators' field present"
    )]
    #[test_case(
        || {
            Builder::new().build()
        }
        => false
        ;
        "missing 'collaborators' field"
    )]
    #[tokio::test]
    async fn section_rule_specified_not_optional_test(
        doc_gen: impl FnOnce() -> CatalystSignedDocument
    ) -> bool {
        let rule = CollaboratorsRule::Specified { optional: false };

        let doc = doc_gen();
        rule.check(&doc).await.unwrap()
    }

    #[test_case(
        || {
            Builder::new().build()
        }
        => true
        ;
        "missing 'collaborators' field"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_metadata_field(SupportedField::Collaborators(
                        vec![create_dummy_key_pair(RoleId::Role0).2].into()
                    ))
                .build()
        }
        => false
        ;
        "valid 'collaborators' field present"
    )]
    #[tokio::test]
    async fn section_rule_not_specified_test(
        doc_gen: impl FnOnce() -> CatalystSignedDocument
    ) -> bool {
        let rule = CollaboratorsRule::NotSpecified;

        let doc = doc_gen();
        rule.check(&doc).await.unwrap()
    }
}
