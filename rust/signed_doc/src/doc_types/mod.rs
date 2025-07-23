//! An implementation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

use crate::DocType;

/// helper macro by evaluating `DocType::try_from_uuid(catalyst_types::uuid::uuid!())`
/// expression
macro_rules! doc_type_init {
    ($s:literal) => {
        match DocType::try_from_uuid(catalyst_types::uuid::uuid!($s)) {
            Ok(v) => v,
            Err(_) => panic!("invalid uuid v4 value"),
        }
    };
}

/// -------------- Document Types --------------
/// Brand document type.
pub const BRAND_PARAMETERS: DocType = doc_type_init!("3e4808cc-c86e-467b-9702-d60baa9d1fca");

/// Campaign Parameters document type.
pub const CAMPAIGN_PARAMETERS: DocType = doc_type_init!("0110ea96-a555-47ce-8408-36efe6ed6f7c");

/// Category Parameters document type.
pub const CATEGORY_PARAMETERS: DocType = doc_type_init!("48c20109-362a-4d32-9bba-e0a9cf8b45be");

/// Proposal document type.
pub const PROPOSAL: DocType = doc_type_init!("7808d2ba-d511-40af-84e8-c0d1625fdfdc");

/// Proposal comment document type.
pub const PROPOSAL_COMMENT: DocType = doc_type_init!("b679ded3-0e7c-41ba-89f8-da62a17898ea");

/// Proposal action document type.
pub const PROPOSAL_SUBMISSION_ACTION: DocType =
    doc_type_init!("5e60e623-ad02-4a1b-a1ac-406db978ee48");

/// Proposal Comment Template document type.
pub const PROPOSAL_COMMENT_FORM_TEMPLATE: DocType =
    doc_type_init!("0b8424d4-ebfd-46e3-9577-1775a69d290c");

/// Proposal Template document type.
pub const PROPOSAL_FORM_TEMPLATE: DocType = doc_type_init!("0ce8ab38-9258-4fbc-a62e-7faa6e58318f");
