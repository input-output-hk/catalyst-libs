//! Catalyst Signed Documents code generation macro's from the defined `signed_doc.json`
//! spec.

mod error;
mod rules;
mod signed_doc_spec;
mod types_consts;

use crate::error::process_error;

/// Defines consts for all Catalyst Signed Documents types values, which are
/// defined inside the `signed_doc.json` spec.
///
/// E.G.
/// ```ignore
/// pub const PROPOSAL: DocType = DocType::try_from_uuid(catalyst_types::uuid::uuid!(
///     "7808d2ba-d511-40af-84e8-c0d1625fdfdc"
/// ));
/// ```
#[proc_macro]
pub fn catalyst_signed_documents_types_consts(
    _: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    types_consts::catalyst_signed_documents_types_consts_impl()
        .unwrap_or_else(process_error)
        .into()
}

/// Defines `documents_rules` function which will return a defined
/// `catalyst_signed_doc::Rules` instances for each corresponding document type, which are
/// defined inside the `signed_doc.json` spec.
///
///  ```ignore
/// fn documents_rules() -> impl Iterator<Item = (DocType, Rules)>
/// ```
#[proc_macro]
pub fn catalyst_signed_documents_rules(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    rules::catalyst_signed_documents_rules_impl()
        .unwrap_or_else(process_error)
        .into()
}
