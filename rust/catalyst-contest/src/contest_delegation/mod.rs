//! `Contest Delegation` document.
//!
//! See the [documentation] for more information.
//!
//! [documentation]: https://docs.dev.projectcatalyst.io/libs/main/architecture/08_concepts/signed_doc/docs/contest_delegation

pub mod rule;

#[cfg(test)]
mod tests;

use anyhow::Context;
use catalyst_signed_doc::{
    CatalystSignedDocument, DocumentRef,
    catalyst_id::CatalystId,
    doc_types::{self, CONTEST_DELEGATION},
    problem_report::ProblemReport,
    providers::{
        CatalystIdSelector, CatalystSignedDocumentProvider, CatalystSignedDocumentSearchQuery,
        DocTypeSelector, DocumentRefSelector,
    },
    uuid::UuidV7,
};

use crate::contest_parameters::ContestParameters;

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

        let delegator = get_delegator(doc, &report);
        let payload = get_payload(doc, &report);
        contest_parameters_checks(doc, provider, &report)?;
        let delegations = get_delegations(doc, payload, provider, &report)?;

        Ok(ContestDelegation {
            doc_ref: doc.doc_ref()?,
            delegator,
            report,
            delegations,
        })
    }
}

/// Get signer of the provided document, which defines as delegator.
fn get_delegator(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> Option<CatalystId> {
    let authors = doc.authors();
    if authors.len() != 1 {
        report.invalid_value(
            "signatures",
            &authors.len().to_string(),
            "1",
            "Contest Delegation document must have only one author/signer",
        );
    }

    authors.into_iter().next()
}

/// Get `ContestDelegationPayload` from the provided `CatalystSignedDocument`, fill the
/// provided `ProblemReport` if something goes wrong.
fn get_payload(
    doc: &CatalystSignedDocument,
    report: &ProblemReport,
) -> ContestDelegationPayload {
    doc.decoded_content()
        .inspect_err(|_| {
            report.functional_validation(
                "Invalid Document content, cannot get decoded bytes",
                "Cannot get a document content during Contest Delegation document validation.",
            );
        })
        .and_then(|v| Ok(serde_json::from_slice::<ContestDelegationPayload>(&v)?))
        .inspect_err(|e| {
            report.functional_validation(
                &e.to_string(),
                "Cannot get a document content during Contest Delegation document validation.",
            );
        })
        .unwrap_or_default()
}

/// Get the 'Contest Parameters' document from the 'parameters' metadata field, applying
/// all necessary validations.
fn contest_parameters_checks(
    doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<()> {
    let Some(doc_ref) = doc.doc_meta().parameters().and_then(|v| v.first()) else {
        report.missing_field(
            "parameters",
            "Contest Delegation must have a 'parameters' metadata field",
        );
        return Ok(());
    };

    let Some(contest_parameters) = provider.try_get_doc(doc_ref)? else {
        report.functional_validation(
            &format!("Cannot get referenced document by reference: {doc_ref}"),
            "Missing 'Contest Parameters' document for the Contest Delegation document",
        );
        return Ok(());
    };

    let Ok(doc_ver) = doc.doc_ver() else {
        report.missing_field(
            "ver",
            "Missing 'ver' metadata field for 'Contest Delegation' document",
        );
        return Ok(());
    };

    ContestParameters::timeline_check(doc_ver, &contest_parameters, report, "Contest Delegation");

    Ok(())
}

/// Get a list of delegations
fn get_delegations(
    doc: &CatalystSignedDocument,
    payload: ContestDelegationPayload,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<Vec<Delegation>> {
    const DEFAULT_WEIGHT: u32 = 1;

    if let Some(ref_field) = doc.doc_meta().doc_ref() {
        let kids = ref_field
            .iter()
            .map(|doc_ref| get_author_kid(doc_ref, provider, report))
            .collect::<anyhow::Result<Vec<_>>>()?;

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
            .filter_map(|(kid, weight)| {
                Some(Delegation {
                    weight,
                    rep_kid: kid?,
                })
            })
            .collect();

        Ok(delegations)
    } else {
        report.missing_field(
            "ref",
            "Contest Delegation document must have least one reference to Rep Nomination document.",
        );
        Ok(Vec::new())
    }
}

/// Get a corresponding authors/signers `CatalystId` for the provided document reference.
fn get_author_kid(
    doc_ref: &DocumentRef,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<Option<CatalystId>> {
    let Some(ref_doc) = provider.try_get_doc(doc_ref)? else {
        report.functional_validation(
            &format!("Cannot get referenced document by reference: {doc_ref}"),
            "Missing 'Rep Nomination' document for the Contest Delegation document",
        );
        return Ok(None);
    };

    rep_nomination_ref_check(&ref_doc, provider, report)?;

    let rep_nomination_authors = ref_doc.authors();
    if rep_nomination_authors.len() != 1 {
        report.invalid_value(
            "signatures",
            &rep_nomination_authors.len().to_string(),
            "1",
            "Rep Nomination document must have only one author/signer",
        );
    }

    let rep_kid = rep_nomination_authors.into_iter().next();
    Ok(rep_kid)
}

/// Verifies that the corresponding 'Rep Nomination' document reference is valid by
/// filling provided report if something fails:
/// - References to the latest version of 'Rep Nomination' document ever submitted to the
///   corresponding 'Contest Parameters' document.
/// - A Representative MUST Delegate to their latest Nomination for a 'Contest
///   Parameters', otherwise their Nomination is invalid.
fn rep_nomination_ref_check(
    ref_doc: &CatalystSignedDocument,
    provider: &dyn CatalystSignedDocumentProvider,
    report: &ProblemReport,
) -> anyhow::Result<()> {
    // We could use 'Rep Nomination'->'parameters' field,
    // because it must be the same as the 'Contest Delegation' document according the
    // `ParametersRule::link_check` verification.
    let Some(parameters) = ref_doc.doc_meta().parameters() else {
        report.missing_field(
            "parameters",
            "Missing 'parameters' metadata field for the 'Rep Nomination' document during 'Content Delegation' validation"
        );
        return Ok(());
    };
    // Trying to find ALL available 'Rep Nomination' documents which reference to the `Contest
    // Parameters`
    let query = CatalystSignedDocumentSearchQuery {
        authors: Some(CatalystIdSelector::Eq(ref_doc.authors())),
        parameters: Some(DocumentRefSelector::Eq(parameters.clone())),
        doc_type: Some(DocTypeSelector::In(vec![doc_types::REP_NOMINATION])),
        ..Default::default()
    };
    let all_nominations = provider.try_search_docs(&query)?;

    let Ok(ref_doc_ref) = ref_doc.doc_ref() else {
        report.missing_field(
            "document reference",
            "Cannot get document reference for the 'Rep Nomination' document during 'Content Delegation' validation",
        );
        return Ok(());
    };

    let latest_ref_doc_ref = all_nominations
        .iter()
        .filter_map(|doc| doc.doc_ref().ok())
        .max()
        .context("A latest version of the document must exist if a first version exists")?;

    if latest_ref_doc_ref != ref_doc_ref {
        report.functional_validation(
            "It must be the latest Rep Nomination document",
            "Content Delegation must reference to the latest version Rep Nomination document",
        );
    }

    // Trying to find the latest 'Contest Delegation' submitted by the representative ('Rep
    // Nomination' author/signer).
    let query = CatalystSignedDocumentSearchQuery {
        authors: Some(CatalystIdSelector::Eq(ref_doc.authors())),
        parameters: Some(DocumentRefSelector::Eq(parameters.clone())),
        doc_ref: Some(DocumentRefSelector::Eq(vec![ref_doc_ref].into())),
        doc_type: Some(DocTypeSelector::In(vec![doc_types::CONTEST_DELEGATION])),
        ..Default::default()
    };
    if provider.try_search_docs(&query)?.is_empty() {
        report.functional_validation(
            "A Representative MUST Delegate to their latest Nomination for a 'Contest Parameters', otherwise their Nomination is invalid.", 
            "Fails to validate a 'Contest Delegation' referenced representative nomination"
        );
    }

    Ok(())
}
