use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    contest_parameters_doc,
    doc_types::CONTEST_PARAMETERS,
    Json,
    template: &DocumentRef,
    parameters: &DocumentRef
);
