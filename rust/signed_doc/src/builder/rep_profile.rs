use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    rep_profile_doc,
    doc_types::REP_PROFILE,
    Json,
    template: &DocumentRef,
    parameters: &DocumentRef
);
