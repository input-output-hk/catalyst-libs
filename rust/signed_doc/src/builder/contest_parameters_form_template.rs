use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    contest_parameters_form_template_doc,
    doc_types::CONTEST_PARAMETERS_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
