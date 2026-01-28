use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    brand_parameters_doc,
    doc_types::BRAND_PARAMETERS,
    Json,
    template: &DocumentRef
);
