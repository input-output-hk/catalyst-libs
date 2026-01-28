use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    proposal_doc,
    doc_types::PROPOSAL,
    Json,
    template: &DocumentRef,
    parameters: &DocumentRef
);
