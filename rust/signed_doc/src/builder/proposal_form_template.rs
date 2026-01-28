use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    proposal_form_template_doc,
    doc_types::PROPOSAL_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
