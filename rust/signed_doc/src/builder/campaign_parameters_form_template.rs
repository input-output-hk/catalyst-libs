use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    campaign_parameters_form_template_doc,
    doc_types::CAMPAIGN_PARAMETERS_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
