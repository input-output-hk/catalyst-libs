use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    campaign_parameters_doc,
    doc_types::CAMPAIGN_PARAMETERS,
    Json,
    template: &DocumentRef,
    parameters: &DocumentRef
);
