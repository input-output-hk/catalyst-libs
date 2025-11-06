//! Catalyst Signed Documents validation logic

pub(crate) mod rules;

use std::{collections::HashMap, sync::LazyLock};

use rules::Rules;

use crate::{
    metadata::DocType,
    providers::{CatalystIdProvider, CatalystSignedDocumentProvider},
    CatalystSignedDocument,
};

/// A table representing a full set or validation rules per document id.
static DOCUMENT_RULES: LazyLock<HashMap<DocType, Rules>> = LazyLock::new(document_rules_init);

/// `DOCUMENT_RULES` initialization function
#[allow(clippy::expect_used)]
fn document_rules_init() -> HashMap<DocType, Rules> {
    let document_rules_map: HashMap<DocType, Rules> = Rules::documents_rules()
        .expect("cannot fail to initialize validation rules")
        .collect();

    document_rules_map
}

/// A comprehensive document type based validation of the `CatalystSignedDocument`.
/// Includes time based validation of the `id` and `ver` fields based on the provided
/// `future_threshold` and `past_threshold` threshold values (in seconds).
/// Return true if it is valid, otherwise return false.
///
/// # Errors
/// If `provider` returns error, fails fast throwing that error.
pub async fn validate<Provider>(
    doc: &CatalystSignedDocument,
    provider: &Provider,
) -> anyhow::Result<bool>
where
    Provider: CatalystSignedDocumentProvider + CatalystIdProvider,
{
    let Ok(doc_type) = doc.doc_type() else {
        doc.report().missing_field(
            "type",
            "Can't get a document type during the validation process",
        );
        return Ok(false);
    };

    let Some(rules) = DOCUMENT_RULES.get(doc_type) else {
        doc.report().invalid_value(
            "`type`",
            &doc.doc_type()?.to_string(),
            "Must be a known document type value",
            "Unsupported document type",
        );
        return Ok(false);
    };
    rules.check(doc, provider).await
}

#[cfg(test)]
mod tests {
    use crate::validator::document_rules_init;

    #[test]
    fn document_rules_init_test() {
        document_rules_init();
    }
}
