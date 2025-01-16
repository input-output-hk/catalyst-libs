//! Catalyst Signed Document COSE Signature information.

pub use catalyst_types::kid_uri::KidUri;
use coset::CoseSignature;

/// Catalyst Signed Document COSE Signature.
#[derive(Debug)]
pub struct Signature {
    /// Key ID
    kid: KidUri,
    /// COSE Signature
    #[allow(dead_code)]
    signature: CoseSignature,
}

/// List of Signatures.
#[derive(Debug)]
pub struct Signatures(Vec<Signature>);

impl Signatures {
    /// List of signature Key IDs.
    pub fn kids(&self) -> Vec<KidUri> {
        self.0.iter().map(|sig| sig.kid.clone()).collect()
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
