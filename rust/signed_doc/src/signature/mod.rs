//! Catalyst Signed Document COSE Signature information.

pub use catalyst_types::kid_uri::KidUri;
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
#[derive(Debug, Clone)]
pub struct Signatures(pub(crate) Vec<Signature>);

impl Signatures {
    /// List of signature Key IDs.
    #[must_use]
    pub fn kids(&self) -> Vec<KidUri> {
        self.0.iter().map(|sig| sig.kid.clone()).collect()
    }

    /// List of signatures.
    #[must_use]
    pub fn signatures(&self) -> Vec<CoseSignature> {
        self.0.iter().map(|sig| sig.signature.clone()).collect()
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
}

impl TryFrom<&Vec<CoseSignature>> for Signatures {
    type Error = crate::error::Error;

    fn try_from(value: &Vec<CoseSignature>) -> Result<Self, Self::Error> {
        let mut signatures = Vec::new();
        let mut errors = Vec::new();
        value
            .iter()
            .cloned()
            .enumerate()
            .for_each(|(idx, signature)| {
                match KidUri::try_from(signature.protected.header.key_id.as_ref()) {
                    Ok(kid) => signatures.push(Signature { kid, signature }),
                    Err(e) => {
                        errors.push(anyhow::anyhow!(
                            "Signature at index {idx} has valid Catalyst Key Id: {e}"
                        ));
                    },
                }
            });

        if errors.is_empty() {
            Ok(Signatures(signatures))
        } else {
            Err(errors.into())
        }
    }
}
