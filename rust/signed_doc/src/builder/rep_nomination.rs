use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    rep_nomination_doc,
    doc_types::REP_NOMINATION,
    Json,
    r#ref: &DocumentRef,
    template: &DocumentRef,
    parameters: &DocumentRef
);
