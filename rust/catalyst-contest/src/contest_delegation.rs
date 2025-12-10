//! `Contest Delegation` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation/#contest-delegation

use catalyst_signed_doc::{
    CatalystId, CatalystSignedDocument, DocumentRef, ProblemReport, UuidV7,
    doc_types::CONTEST_DELEGATION,
};

/// `Contest Delegation` document type.
pub struct ContestDelegation {
    /// Document reference info
    doc_ref: DocumentRef,

    /// A corresponding `CatalystId` of the delegator (author of the document).
    delegator: Option<CatalystId>,

    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    #[allow(dead_code)]
    report: ProblemReport,
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

    /// Returns 'delegator'
    #[must_use]
    pub fn delegator(&self) -> anyhow::Result<&CatalystId> {
        self.delegator
            .as_ref()
            .ok_or(anyhow::anyhow!("Missing 'delegator'"))
    }

    /// Returns `ProblemReport`
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }
}

impl TryFrom<CatalystSignedDocument> for ContestDelegation {
    type Error = anyhow::Error;

    fn try_from(v: CatalystSignedDocument) -> Result<Self, Self::Error> {
        if v.problem_report().is_problematic() {
            anyhow::bail!("Provided document is not valid {:?}", v.problem_report())
        }
        anyhow::ensure!(
            v.doc_type()? == &CONTEST_DELEGATION,
            "Document must be Contest Delegation type"
        );

        let report = ProblemReport::new("Contest Delegation");
        let doc_ref = v.doc_ref()?;

        let authors = v.authors();
        let delegator = if let &[delegator] = authors.as_slice() {
            Some(delegator)
        } else {
            None
        };

        Ok(ContestDelegation {
            doc_ref,
            delegator,
            report,
        })
    }
}
