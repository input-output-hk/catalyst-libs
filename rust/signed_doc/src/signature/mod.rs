//! Catalyst Signed Document COSE Signature information.

pub use catalyst_types::catalyst_id::CatalystId;
use catalyst_types::problem_report::ProblemReport;
use coset::CoseSignature;

/// Catalyst Signed Document COSE Signature.
#[derive(Debug, Clone)]
pub struct Signature {
    /// Key ID
    kid: CatalystId,
    /// Raw signature data
    signature: Vec<u8>,
}

impl Signature {
    /// Return `kid` field (`CatalystId`), identifier who made a signature
    pub fn kid(&self) -> &CatalystId {
        &self.kid
    }

    /// Return raw signature bytes itself
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Convert COSE Signature to `Signature`.
    pub(crate) fn from_cose_sig(signature: CoseSignature, report: &ProblemReport) -> Option<Self> {
        match CatalystId::try_from(signature.protected.header.key_id.as_ref()) {
            Ok(kid) if kid.is_uri() => {
                Some(Self {
                    kid,
                    signature: signature.signature,
                })
            },
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
    /// Return an iterator over the signatures
    pub fn iter(&self) -> impl Iterator<Item = &Signature> + use<'_> {
        self.0.iter()
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

impl<C> minicbor::Encode<C> for Signature {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(3)?;
        let mut p_headers = minicbor::Encoder::new(Vec::new());
        // protected headers (kid field)
        p_headers
            .map(1)
            .map_err(minicbor::encode::Error::message)?
            .u8(4)
            .map_err(minicbor::encode::Error::message)?
            .encode(&self.kid)
            .map_err(minicbor::encode::Error::message)?;
        e.bytes(p_headers.into_writer().as_slice())?;
        // empty unprotected headers
        e.map(0)?;
        e.bytes(&self.signature)?;
        Ok(())
    }
}

impl<C> minicbor::Encode<C> for Signatures {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(
            self.0
                .len()
                .try_into()
                .map_err(minicbor::encode::Error::message)?,
        )?;
        for sign in self.iter() {
            e.encode(sign)?;
        }

        Ok(())
    }
}
