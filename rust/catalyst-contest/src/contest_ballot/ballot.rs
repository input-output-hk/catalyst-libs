//! An individual Ballot cast in a Contest by a registered user.

use anyhow::{bail, ensure};
use catalyst_signed_doc::{
    CatalystSignedDocument, doc_types::CONTEST_BALLOT, providers::CatalystSignedDocumentProvider,
};

/// An individual Ballot cast in a Contest by a registered user.
pub struct ContestBallot {}

impl ContestBallot {
    pub async fn new<Provider>(
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<Self>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        if doc.report().is_problematic() {
            bail!("Provided document is not valid {:?}", doc.report())
        }
        ensure!(
            doc.doc_type()? == &CONTEST_BALLOT,
            "Document must be Contest Ballot type"
        );

        // TODO: FIXME:
        todo!()
    }
}
