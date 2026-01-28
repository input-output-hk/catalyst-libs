use crate::{Chain, DocumentRef, builder::doc_builder, doc_types};

doc_builder!(
    contest_ballot_checkpoint_doc,
    doc_types::CONTEST_BALLOT_CHECKPOINT,
    Cbor,
    r#ref: &DocumentRef,
    parameters: &DocumentRef,
    chain: &Chain
);
