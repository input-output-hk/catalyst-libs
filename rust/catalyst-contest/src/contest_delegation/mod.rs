//! `Contest Delegation` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation/#contest-delegation

pub mod rule;

use anyhow::Context;
use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef, catalyst_id::CatalystId, doc_types::CONTEST_DELEGATION,
    problem_report::ProblemReport, providers::CatalystSignedDocumentProvider, uuid::UuidV7,
};

/// `Contest Delegation` document type.
#[derive(Debug, Clone)]
pub struct ContestDelegation {
    /// Document reference info
    doc_ref: DocumentRef,
    /// A corresponding `CatalystId` of the delegator (author of the document).
    delegator: Option<CatalystId>,
    /// Delegations
    delegations: Vec<Delegation>,
    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    report: ProblemReport,
}

/// Delegation type.
#[derive(Debug, Clone)]
pub struct Delegation {
    /// A weight is assigned to the representative, which is used to define their voting
    /// power value.
    pub weight: u32,
    /// Representative `CatalystId`
    pub rep_kid: CatalystId,
}

impl PartialEq for ContestDelegation {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.doc_ref.eq(&other.doc_ref)
    }
}

/// Content Delegation JSON payload type.
#[derive(Debug, Clone, Default, serde::Deserialize)]
struct ContestDelegationPayload {
    /// List of weights to apply to each delegate.
    /// This list is in the same order as the delegate references.
    /// If there are fewer entries than delegates, then the missing weights are set to
    /// `1`. If there are more weights, then the extra weights are ignored.
    /// If the array is empty, then the weights assigned is `1`.
    weights: Vec<u32>,
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

    /// Returns 'delegator'.
    ///
    /// # Errors
    ///  - Missing 'delegator'.
    pub fn delegator(&self) -> anyhow::Result<&CatalystId> {
        self.delegator
            .as_ref()
            .ok_or(anyhow::anyhow!("Missing 'delegator'"))
    }

    /// Returns delegations
    #[must_use]
    pub fn delegations(&self) -> &[Delegation] {
        &self.delegations
    }

    /// Returns `ProblemReport`
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }
}

impl ContestDelegation {
    /// Trying to build Contest Delegation document collecting all issues into the
    /// `report`.
    ///
    /// # Errors
    ///  - If provided document not a Contest Delegation type
    ///  - If provided document is invalid (`report().is_problematic()`)
    ///  - `provider` returns error
    pub fn new<Provider>(
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<Self>
    where
        Provider: CatalystSignedDocumentProvider,
    {
        if doc.report().is_problematic() {
            anyhow::bail!("Provided document is not valid {:?}", doc.report())
        }
        anyhow::ensure!(
            doc.doc_type()? == &CONTEST_DELEGATION,
            "Document must be Contest Delegation type"
        );

        let report = ProblemReport::new("Contest Delegation");

        let (delegator, _) = get_delegator(doc, &report);
        let (payload, _) = get_payload(doc, &report);
        let (delegations, _) = get_delegations(doc, payload, provider, &report)?;

        Ok(ContestDelegation {
            doc_ref: doc.doc_ref()?,
            delegator,
            report,
            delegations,
        })
    }
}

/// Get signer of the provided document, which defines as delegator.
/// Returns additional boolean flag, was it valid or not.
fn get_delegator(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> (Option<CatalystId>, bool) {
    let mut valid = true;
    let authors = doc.authors();
    if authors.len() != 1 {
        report.invalid_value(
            "signatures",
            &authors.len().to_string(),
            "1",
            "Contest Delegation document must have only one author/signer",
        );
        valid = false;
    }

    let delegator = authors.into_iter().next();
    (delegator, valid)
}

/// Get `CatalystSignedDocument` from the provided `CatalystSignedDocument`, fill the
/// provided `ProblemReport` if something goes wrong.
/// Returns additional boolean flag, was it valid or not.
fn get_payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> (ContestDelegationPayload, bool) {
    let mut valid = true;
    let payload = doc
            .decoded_content()
            .inspect_err(|_| {
                report.functional_validation(
                    "Invalid Document content, cannot get decoded bytes",
                    "Cannot get a document content during Contest Delegation document validation.",
                );
                valid = false;
            })
            .and_then(|v | Ok(serde_json::from_slice::<ContestDelegationPayload>(&v)?))
            .inspect_err(|_| {
                report.functional_validation(
                    "Invalid Document content, must be a valid JSON object compliant with the JSON schema.",
                    "Cannot get a document content during Contest Delegation document validation.",
                );
                valid = false;
            })
            .unwrap_or_default();

    (payload, valid)
}

/// Get a list of delegations
/// Returns additional boolean flag, was it valid or not.
fn get_delegations(
    doc: &CatalystSignedDocument,
    payload: ContestDelegationPayload,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<(Vec<Delegation>, bool)> {
    const DEFAULT_WEIGHT: u32 = 1;

    if let Some(ref_field) = doc.doc_meta().doc_ref() {
        let kids = ref_field
            .iter()
            .map(|doc_ref| get_author_kid(doc_ref, provider, report))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let valid = kids.iter().all(|(_, valid)| *valid);

        // If there are fewer entries than delegates, then the missing weights are set to
        // `1`. If there are more weights, then the extra weights are ignored.
        // If array is empty, then the weights assigned is `1`.
        let weights_iter = payload
            .weights
            .into_iter()
            .chain(std::iter::repeat(DEFAULT_WEIGHT))
            .take(ref_field.len());

        let delegations = kids
            .into_iter()
            .zip(weights_iter)
            .filter_map(|((kid, _), weight)| {
                Some(Delegation {
                    weight,
                    rep_kid: kid?,
                })
            })
            .collect();

        Ok((delegations, valid))
    } else {
        report.missing_field(
            "ref",
            "Contest Delegation document must have least one reference to Rep Nomination document.",
        );
        Ok((Vec::new(), false))
    }
}

/// Get a corresponding authors/signers `CatalystId` for the provided document reference.
/// Returns additional boolean flag, was it valid or not.
fn get_author_kid(
    doc_ref: &DocumentRef,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<(Option<CatalystId>, bool)> {
    let mut valid = true;
    let Some(ref_doc) = provider.try_get_doc(doc_ref)? else {
        report.functional_validation(
            &format!("Cannot get referenced document by reference: {doc_ref}"),
            "Missing representative reference document for the Contest Delegation document",
        );
        valid = false;
        return Ok((None, valid));
    };

    valid &= rep_nomination_ref_check(&ref_doc, provider, report)?;

    let rep_nomination_authors = ref_doc.authors();
    if rep_nomination_authors.len() != 1 {
        report.invalid_value(
            "signatures",
            &rep_nomination_authors.len().to_string(),
            "1",
            "Rep Nomination document must have only one author/signer",
        );
        valid = false;
    }

    let rep_kid = rep_nomination_authors.into_iter().next();
    Ok((rep_kid, valid))
}

/// Verifies that the corresponding 'Rep Nomination' document reference is valid:
/// - References to the latest version of 'Rep Nomination' document ever submitted to the
///   corresponding 'Contest Parameters' document.
/// -
fn rep_nomination_ref_check(
    ref_doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<bool> {
    let mut valid = true;

    if let Ok(ref_doc_ref) = ref_doc.doc_ref() {
        let latest_ref_doc = provider
            .try_get_last_doc(*ref_doc_ref.id())?
            .context("A latest version of the document must exist if a first version exists")?;

        let latest_ref_doc_ref = latest_ref_doc.doc_ref().context(
            "Cannot get document reference for the latest representative reference document",
        )?;
        if latest_ref_doc_ref != ref_doc_ref {
            report.functional_validation(
                "It must be the latest Rep Nomination document",
                "Content Delegation must reference to the latest version Rep Nomination document",
            );
            valid = false;
        }
    } else {
        report.missing_field(
            "document reference",
            "Cannot get document reference for the representative reference document",
        );
        valid = false;
    }

    Ok(valid)
}
