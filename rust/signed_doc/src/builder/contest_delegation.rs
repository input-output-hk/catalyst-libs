use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    contest_delegation_doc,
    doc_types::CONTEST_DELEGATION,
    Json,
    r#ref: &DocumentRef,
    parameters: &DocumentRef
);
