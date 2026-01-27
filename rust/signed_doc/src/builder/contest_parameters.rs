use crate::{
    builder::{doc_builder,}, doc_types, DocumentRef
};

doc_builder!(contest_parameters_doc, doc_types::CONTEST_PARAMETERS, Json, template: &DocumentRef, parameters: &DocumentRef);

