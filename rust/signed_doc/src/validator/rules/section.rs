//! `section` rule type impl.

use futures::FutureExt;

use crate::{
    CatalystSignedDocument, providers::CatalystProvider,
    validator::CatalystSignedDocumentValidationRule,
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
    fn check<'a>(
        &'a self,
        doc: &'a CatalystSignedDocument,
        _provider: &'a dyn CatalystProvider,
    ) -> futures::future::BoxFuture<'a, anyhow::Result<bool>> {
        async { self.check_inner(doc) }.boxed()
    }
}

impl SectionRule {
    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        if let Self::Specified { optional } = self
            && doc.doc_meta().section().is_none()
            && !optional
        {
            doc.report()
                .missing_field("section", "Document must have a section field");
            return Ok(false);
        }
        if let Self::NotSpecified = self
            && let Some(section) = doc.doc_meta().section()
        {
            doc.report().unknown_field(
                "section",
                &section.to_string(),
                "Document does not expect to have a section field",
            );
            return Ok(false);
        }

        Ok(true)
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
        assert!(rule.check_inner(&doc).unwrap());

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: true };
        assert!(rule.check_inner(&doc).unwrap());

        let doc = Builder::new().build();
        let rule = SectionRule::Specified { optional: false };
        assert!(!rule.check_inner(&doc).unwrap());
    }

    #[test]
    fn section_rule_not_specified_test() {
        let rule = SectionRule::NotSpecified;

        let doc = Builder::new().build();
        assert!(rule.check_inner(&doc).unwrap());

        let doc = Builder::new()
            .with_metadata_field(SupportedField::Section("$".parse().unwrap()))
            .build();
        assert!(!rule.check_inner(&doc).unwrap());
    }
}
