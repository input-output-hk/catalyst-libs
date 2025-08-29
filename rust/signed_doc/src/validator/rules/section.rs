//! `section` rule type impl.

use crate::CatalystSignedDocument;

/// `section` field validation rule
#[derive(Debug)]
pub(crate) enum SectionRule {
    /// Is 'section' specified
    #[allow(dead_code)]
    Specified {
        /// optional flag for the `section` field
        optional: bool,
    },
    /// 'section' is not specified
    NotSpecified,
}

impl SectionRule {
    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        if let Self::Specified { optional } = self {
            if doc.doc_meta().section().is_none() && !optional {
                doc.report()
                    .missing_field("section", "Document must have a section field");
                return Ok(false);
            }
        }
        if let Self::NotSpecified = self {
            if let Some(section) = doc.doc_meta().section() {
                doc.report().unknown_field(
                    "section",
                    &section.to_string(),
                    "Document does not expect to have a section field",
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[tokio::test]
    async fn section_rule_specified_test() {
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        let rule = SectionRule::Specified { optional: false };
        assert!(rule.check(&doc).await.unwrap());

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: true };
        assert!(rule.check(&doc).await.unwrap());

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: false };
        assert!(!rule.check(&doc).await.unwrap());
    }

    #[tokio::test]
    async fn section_rule_not_specified_test() {
        let rule = SectionRule::NotSpecified;

        let doc = Builder::new().build();
        assert!(rule.check(&doc).await.unwrap());

        let doc = Builder::new()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        assert!(!rule.check(&doc).await.unwrap());
    }
}
