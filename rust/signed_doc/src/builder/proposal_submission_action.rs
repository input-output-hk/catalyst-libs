use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    proposal_submission_action_doc,
    doc_types::PROPOSAL_SUBMISSION_ACTION,
    Json,
    r#ref: &DocumentRef,
    parameters: &DocumentRef
);
