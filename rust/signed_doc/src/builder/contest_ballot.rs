use crate::{DocumentRef, builder::doc_builder, doc_types};

doc_builder!(contest_ballot_doc, doc_types::CONTEST_BALLOT, Cbor, r#ref: &[DocumentRef], parameters: &DocumentRef);
