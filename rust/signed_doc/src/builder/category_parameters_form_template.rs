use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    category_parameters_form_template_doc,
    doc_types::CATEGORY_PARAMETERS_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
