//! `section` rule type impl.

use crate::{
    CatalystSignedDocument, providers::Provider, validator::CatalystSignedDocumentValidationRule,
};

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

impl CatalystSignedDocumentValidationRule for SectionRule {
    fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl SectionRule {
    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        if let Self::Specified { optional } = self
            && doc.doc_meta().section().is_none()
            && !optional
        {
            doc.report()
                .missing_field("section", "Document must have a section field");
            return false;
        }
        if let Self::NotSpecified = self
            && let Some(section) = doc.doc_meta().section()
        {
            doc.report().unknown_field(
                "section",
                &section.to_string(),
                "Document does not expect to have a section field",
            );
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[test]
    fn section_rule_specified_test() {
        let doc = Builder::new()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        let rule = SectionRule::Specified { optional: false };
        assert!(rule.check_inner(&doc));

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: true };
        assert!(rule.check_inner(&doc));

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: false };
        assert!(!rule.check_inner(&doc));
    }

    #[test]
    fn section_rule_not_specified_test() {
        let rule = SectionRule::NotSpecified;

        let doc = Builder::new().build();
        assert!(rule.check_inner(&doc));

        let doc = Builder::new()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        assert!(!rule.check_inner(&doc));
    }
}
