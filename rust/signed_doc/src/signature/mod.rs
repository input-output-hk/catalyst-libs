//! Catalyst Signed Document COSE Signature information.

pub use catalyst_types::catalyst_id::CatalystId;
use catalyst_types::problem_report::ProblemReport;
use coset::CoseSignature;

/// Catalyst Signed Document COSE Signature.
#[derive(Debug, Clone)]
pub struct Signature {
    /// Key ID
    kid: CatalystId,
    /// COSE Signature
    signature: CoseSignature,
}

impl Signature {
    /// Convert COSE Signature to `Signature`.
    pub(crate) fn from_cose_sig(signature: CoseSignature, report: &ProblemReport) -> Option<Self> {
        match CatalystId::try_from(signature.protected.header.key_id.as_ref()) {
            Ok(kid) if kid.is_uri() => Some(Self { kid, signature }),
            Ok(kid) => {
                report.invalid_value(
                    "COSE signature protected header key ID",
                    &kid.to_string(),
                    &format!(
                        "COSE signature protected header key ID must be a Catalyst ID, missing URI schema {}", CatalystId::SCHEME
                    ),
                    "Converting COSE signature header key ID to CatalystId",
                );
                None
            },
            Err(e) => {
                report.conversion_error(
                    "COSE signature protected header key ID",
                    &format!("{:?}", &signature.protected.header.key_id),
                    &format!("{e:?}"),
                    "Converting COSE signature header key ID to CatalystId",
                );
                None
            },
        }
    }
}

/// List of Signatures.
#[derive(Debug, Clone, Default)]
pub struct Signatures(Vec<Signature>);

impl Signatures {
    /// Return a list of author IDs (short form of Catalyst IDs).
    #[must_use]
    pub(crate) fn authors(&self) -> Vec<CatalystId> {
        self.kids().into_iter().map(|k| k.as_short_id()).collect()
    }

    /// Return a list of Document's Catalyst IDs.
    #[must_use]
    pub(crate) fn kids(&self) -> Vec<CatalystId> {
        self.0.iter().map(|sig| sig.kid.clone()).collect()
    }

    /// Iterator of COSE signatures object with kids.
    pub(crate) fn cose_signatures_with_kids(
        &self,
    ) -> impl Iterator<Item = (&CoseSignature, &CatalystId)> + use<'_> {
        self.0.iter().map(|sig| (&sig.signature, &sig.kid))
    }

    /// List of COSE signatures object.
    pub(crate) fn cose_signatures(&self) -> impl Iterator<Item = CoseSignature> + use<'_> {
        self.0.iter().map(|sig| sig.signature.clone())
    }

    /// Add a `Signature` object into the list
    pub(crate) fn push(&mut self, sign: Signature) {
        self.0.push(sign);
    }

    /// Number of signatures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True if the document has no signatures.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert list of COSE Signature to `Signatures`.
    pub(crate) fn from_cose_sig_list(cose_sigs: &[CoseSignature], report: &ProblemReport) -> Self {
        let res = cose_sigs
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(idx, signature)| {
                let sign = Signature::from_cose_sig(signature, report);
                if sign.is_none() {
                    report.other(&format!("COSE signature protected header key ID at id {idx}"), "Converting COSE signatures list to Catalyst Signed Documents signatures list",);
                }
                sign
            }).collect();

        Self(res)
    }
}
