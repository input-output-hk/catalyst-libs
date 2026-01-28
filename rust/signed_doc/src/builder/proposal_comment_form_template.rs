use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    proposal_comment_form_template_doc,
    doc_types::PROPOSAL_COMMENT_FORM_TEMPLATE,
    SchemaJson,
    parameters: &DocumentRef
);
