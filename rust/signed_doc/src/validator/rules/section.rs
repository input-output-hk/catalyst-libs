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
        self.check_inner(doc);
        Ok(!doc.report().is_problematic())
    }
}

impl SectionRule {
    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) {
        if let Self::Specified { optional } = self
            && doc.doc_meta().section().is_none()
            && !optional
        {
            doc.report()
                .missing_field("section", "Document must have a section field");
        }
        if let Self::NotSpecified = self
            && let Some(section) = doc.doc_meta().section()
        {
            doc.report().unknown_field(
                "section",
                &section.to_string(),
                "Document does not expect to have a section field",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::tests::Builder, metadata::SupportedField};

    #[test]
    fn section_rule_specified_test() {
        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        let rule = SectionRule::Specified { optional: false };
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());

        let doc = Builder::with_required_fields().build();
        let rule = SectionRule::Specified { optional: true };
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());

        let doc = Builder::with_required_fields().build();
        let rule = SectionRule::Specified { optional: false };
        rule.check_inner(&doc);
        let report = format!("{:?}", doc.report());
        println!("{report}");
        assert!(doc.report().is_problematic());
        assert!(report.contains("Document must have a section field"));
        assert_eq!(1, doc.report().entries().count());
    }

    #[test]
    fn section_rule_not_specified_test() {
        let rule = SectionRule::NotSpecified;

        let doc = Builder::with_required_fields().build();
        rule.check_inner(&doc);
        assert!(!doc.report().is_problematic());

        let doc = Builder::with_required_fields()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        rule.check_inner(&doc);
        let report = format!("{:?}", doc.report());
        println!("{report}");
        assert!(doc.report().is_problematic());
        assert!(report.contains("Document does not expect to have a section field"));
        assert_eq!(1, doc.report().entries().count());
    }
}
