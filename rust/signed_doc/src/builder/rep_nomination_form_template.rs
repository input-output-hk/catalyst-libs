use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    rep_nomination_form_template_doc,
    doc_types::REP_NOMINATION_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
