use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    category_parameters_doc,
    doc_types::CATEGORY_PARAMETERS,
    Json,
    template: &DocumentRef,
    parameters: &DocumentRef
);
