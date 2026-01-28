use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    rep_profile_form_template_doc,
    doc_types::REP_PROFILE_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
