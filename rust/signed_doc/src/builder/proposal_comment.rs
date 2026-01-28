use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    proposal_comment_doc,
    doc_types::PROPOSAL_COMMENT,
    Json,
    r#ref: &DocumentRef,
    template: &DocumentRef,
    parameters: &DocumentRef
);
