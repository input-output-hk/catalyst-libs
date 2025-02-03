//! Catalyst Signed Document COSE Signature information.

use anyhow::bail;
pub use catalyst_types::kid_uri::KidUri;
use catalyst_types::problem_report::ProblemReport;
use coset::CoseSignature;

/// Catalyst Signed Document COSE Signature.
#[derive(Debug, Clone)]
pub struct Signature {
    /// Key ID
    kid: KidUri,
    /// COSE Signature
    signature: CoseSignature,
}

/// List of Signatures.
#[derive(Debug, Clone, Default)]
pub struct Signatures(Vec<Signature>);

impl Signatures {
    /// Creates an empty signatures list.
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// List of signature Key IDs.
    #[must_use]
    pub fn kids(&self) -> Vec<KidUri> {
        self.0.iter().map(|sig| sig.kid.clone()).collect()
    }

    /// List of signatures.
    #[must_use]
    pub fn cose_signatures(&self) -> Vec<CoseSignature> {
        self.0.iter().map(|sig| sig.signature.clone()).collect()
    }

    /// Add a new signature
    pub fn push(&mut self, kid: KidUri, signature: CoseSignature) {
        self.0.push(Signature { kid, signature });
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
    pub(crate) fn from_cose_sig(
        cose_sigs: &[CoseSignature], error_report: &ProblemReport,
    ) -> anyhow::Result<Self> {
        let mut signatures = Vec::new();

        cose_sigs
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(idx, signature)| {
                match KidUri::try_from(signature.protected.header.key_id.as_ref()) {
                    Ok(kid) => signatures.push(Signature { kid, signature }),
                    Err(e) => {
                        error_report.conversion_error(
                            &format!("COSE signature protected header key ID at id {idx}"),
                            &format!("{:?}", &signature.protected.header.key_id),
                            &format!("{e:?}"),
                            "Converting COSE signature header key ID to KidUri",
                        );
                    },
                }
            });
        if error_report.is_problematic() {
            bail!("Failed to convert COSE Signatures to Signatures");
        }
        Ok(Signatures(signatures))
    }
}
