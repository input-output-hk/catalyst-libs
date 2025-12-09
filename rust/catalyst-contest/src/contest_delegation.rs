//! `Contest Delegation` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation/#contest-delegation

use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, UuidV7, doc_types::CONTEST_DELEGATION,
};

/// `Contest Delegation` document type.
pub struct ContestDelegation {
    /// Document reference info
    doc_ref: DocumentRef,
}

impl ContestDelegation {
    /// Returns 'id' metadata field
    #[must_use]
    pub fn id(&self) -> &UuidV7 {
        self.doc_ref.id()
    }

    /// Returns 'ver' metadata field
    #[must_use]
    pub fn ver(&self) -> &UuidV7 {
        self.doc_ref.ver()
    }
}

impl TryFrom<CatalystSignedDocument> for ContestDelegation {
    type Error = anyhow::Error;

    fn try_from(value: CatalystSignedDocument) -> Result<Self, Self::Error> {
        anyhow::ensure!(
            value.doc_type()? == &CONTEST_DELEGATION,
            "Document must be Contest Delegation type"
        );

        let doc_ref = value.doc_ref()?;

        Ok(ContestDelegation { doc_ref })
    }
}
