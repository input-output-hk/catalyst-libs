//! Catalyst documents signing crate

mod builder;
mod content;
pub mod doc_types;
pub mod error;
mod metadata;
mod signature;
mod utils;
pub mod validator;

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    sync::Arc,
};

pub use builder::Builder;
use catalyst_types::problem_report::ProblemReport;
pub use catalyst_types::uuid::{Uuid, UuidV4, UuidV7};
pub use content::Content;
use coset::{CborSerializable, Header};
use ed25519_dalek::VerifyingKey;
use error::CatalystSignedDocError;
use metadata::{ContentEncoding, ContentType};
pub use metadata::{DocumentRef, ExtraFields, Metadata};
use minicbor::{decode, encode, Decode, Decoder, Encode};
pub use signature::{IdUri, Signatures};
use utils::context::DecodeSignDocCtx;

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Debug, Clone)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Content
    content: Content,
    /// Signatures
    signatures: Signatures,
}

/// Keep all the contents private.
/// Better even to use a structure like this.  Wrapping in an Arc means we don't have to
/// manage the Arc anywhere else. These are likely to be large, best to have the Arc be
/// non-optional.
#[derive(Clone)]
pub struct CatalystSignedDocument {
    /// Catalyst Signed Document metadata, raw doc, with content errors.
    inner: Arc<InnerCatalystSignedDocument>,
}

impl Display for CatalystSignedDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{}", self.inner.metadata)?;
        writeln!(f, "Payload Size: {} bytes", self.inner.content.size())?;
        writeln!(f, "Signature Information")?;
        if self.inner.signatures.is_empty() {
            writeln!(f, "  This document is unsigned.")?;
        } else {
            for kid in &self.inner.signatures.kids() {
                writeln!(f, "  Signature Key ID: {kid}")?;
            }
        }
        Ok(())
    }
}

impl From<InnerCatalystSignedDocument> for CatalystSignedDocument {
    fn from(inner: InnerCatalystSignedDocument) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}

impl CatalystSignedDocument {
    // A bunch of getters to access the contents, or reason through the document, such as.

    /// Return Document Type `UUIDv4`.
    #[must_use]
    pub fn doc_type(&self) -> UuidV4 {
        self.inner.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    #[must_use]
    pub fn doc_id(&self) -> UuidV7 {
        self.inner.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    #[must_use]
    pub fn doc_ver(&self) -> UuidV7 {
        self.inner.metadata.doc_ver()
    }

    /// Return document `Content`.
    #[must_use]
    pub fn doc_content(&self) -> &Content {
        &self.inner.content
    }

    /// Return document `ContentType`.
    #[must_use]
    pub fn doc_content_type(&self) -> ContentType {
        self.inner.metadata.content_type()
    }

    /// Return document `ContentEncoding`.
    #[must_use]
    pub fn doc_content_encoding(&self) -> Option<ContentEncoding> {
        self.inner.metadata.content_encoding()
    }

    /// Return document metadata content.
    #[must_use]
    pub fn doc_meta(&self) -> &ExtraFields {
        self.inner.metadata.extra()
    }

    /// Return a Document's signatures
    #[must_use]
    pub(crate) fn signatures(&self) -> &Signatures {
        &self.inner.signatures
    }

    /// Return a list of Document's Catalyst IDs.
    #[must_use]
    pub fn kids(&self) -> Vec<IdUri> {
        self.inner.signatures.kids()
    }

    /// Return a list of Document's author IDs (short form of Catalyst IDs).
    #[must_use]
    pub fn authors(&self) -> Vec<IdUri> {
        self.inner.signatures.authors()
    }

    /// Verify document signatures.
    ///
    /// # Errors
    ///
    /// Returns a report of verification failures and the source error.
    /// If `provider` returns error, fails fast and placed this error into
    /// `CatalystSignedDocError::error`.
    pub fn verify<P>(&self, provider: P) -> Result<(), CatalystSignedDocError>
    where P: Fn(&IdUri) -> anyhow::Result<Option<VerifyingKey>> {
        let report = ProblemReport::new("Catalyst Signed Document Verification");

        let cose_sign = match self.as_cose_sign() {
            Ok(cose_sign) => cose_sign,
            Err(e) => {
                report.other(
                    "Cannot build a COSE sign object",
                    "During encoding signed document as COSE SIGN",
                );
                return Err(CatalystSignedDocError::new(report, e));
            },
        };

        for (signature, kid) in self.signatures().cose_signatures_with_kids() {
            match provider(kid) {
                Ok(Some(pk)) => {
                    let tbs_data = cose_sign.tbs_data(&[], signature);
                    match signature.signature.as_slice().try_into() {
                        Ok(signature_bytes) => {
                            let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
                            if let Err(e) = pk.verify_strict(&tbs_data, &signature) {
                                report.functional_validation(
                                    &format!(
                                        "Verification failed for signature with Key ID {kid}: {e}"
                                    ),
                                    "During signature validation with verifying key",
                                );
                            }
                        },
                        Err(_) => {
                            report.invalid_value(
                                "cose signature",
                                &format!("{}", signature.signature.len()),
                                &format!("must be {}", ed25519_dalek::Signature::BYTE_SIZE),
                                "During encoding cose signature to bytes",
                            );
                        },
                    }
                },
                Ok(None) => {
                    report.other(
                        &format!("Missing public key for {kid}."),
                        "During public key extraction",
                    );
                },
                Err(e) => {
                    return Err(CatalystSignedDocError::new(report, e));
                },
            }
        }

        if report.is_problematic() {
            return Err(CatalystSignedDocError::new(
                report,
                anyhow::anyhow!("Signature validation for Catalyst Signed Document fails"),
            ));
        }

        Ok(())
    }

    /// Returns a signed document `Builder` pre-loaded with the current signed document's
    /// data.
    #[must_use]
    pub fn into_builder(self) -> Builder {
        Builder::new()
            .with_metadata(self.inner.metadata.clone())
            .with_decoded_content(self.inner.content.decoded_bytes().to_vec())
            .with_signatures(self.inner.signatures.clone())
    }

    /// Convert Catalyst Signed Document into `coset::CoseSign`
    fn as_cose_sign(&self) -> anyhow::Result<coset::CoseSign> {
        let protected_header = Header::try_from(&self.inner.metadata)
            .map_err(|e| anyhow::anyhow!("Failed to encode Document Metadata: {e}"))?;

        let mut builder = coset::CoseSignBuilder::new()
            .protected(protected_header)
            .payload(self.inner.content.encoded_bytes()?);

        for signature in self.signatures().cose_signatures() {
            builder = builder.add_signature(signature);
        }
        Ok(builder.build())
    }
}

impl TryFrom<&[u8]> for CatalystSignedDocument {
    type Error = CatalystSignedDocError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let error_report = ProblemReport::new("Catalyst Signed Document");
        let mut ctx = DecodeSignDocCtx { error_report };
        let decoded: CatalystSignedDocument = minicbor::decode_with(value, &mut ctx)
            .map_err(|e| CatalystSignedDocError::new(ctx.error_report, e.into()))?;
        Ok(decoded)
    }
}

impl Decode<'_, DecodeSignDocCtx> for CatalystSignedDocument {
    fn decode(d: &mut Decoder<'_>, ctx: &mut DecodeSignDocCtx) -> Result<Self, decode::Error> {
        let start = d.position();
        d.skip()?;
        let end = d.position();
        let cose_bytes = d
            .input()
            .get(start..end)
            .ok_or(minicbor::decode::Error::end_of_input())?;

        let cose_sign = coset::CoseSign::from_slice(cose_bytes).map_err(|e| {
            ctx.error_report.invalid_value(
                "COSE sign document bytes",
                &format!("{:?}", &cose_bytes),
                &format!("Cannot convert bytes to CoseSign {e:?}"),
                "Creating COSE Sign document",
            );
            minicbor::decode::Error::message(format!("Invalid COSE Sign document: {e}"))
        })?;

        let metadata = Metadata::from_protected_header(&cose_sign.protected, &ctx.error_report)
            .map_or_else(
                |e| {
                    ctx.error_report.conversion_error(
                        "COSE sign protected header",
                        &format!("{:?}", &cose_sign.protected),
                        &format!("Expected Metadata: {e:?}"),
                        "Converting COSE Sign protected header to Metadata",
                    );
                    None
                },
                Some,
            );
        let signatures = Signatures::from_cose_sig(&cose_sign.signatures, &ctx.error_report)
            .map_or_else(
                |e| {
                    ctx.error_report.conversion_error(
                        "COSE sign signatures",
                        &format!("{:?}", &cose_sign.signatures),
                        &format!("Expected Signatures {e:?}"),
                        "Converting COSE Sign signatures to Signatures",
                    );
                    None
                },
                Some,
            );

        if cose_sign.payload.is_none() {
            ctx.error_report
                .missing_field("COSE Sign Payload", "Missing document content (payload)");
        }

        match (cose_sign.payload, metadata, signatures) {
            (Some(payload), Some(metadata), Some(signatures)) => {
                let content = Content::from_encoded(
                    payload.clone(),
                    metadata.content_type(),
                    metadata.content_encoding(),
                )
                .map_err(|e| {
                    ctx.error_report.invalid_value(
                        "Document Content",
                        &format!(
                            "Given value {:?}, {:?}, {:?}",
                            payload,
                            metadata.content_type(),
                            metadata.content_encoding()
                        ),
                        &format!("{e:?}"),
                        "Creating document content",
                    );
                    minicbor::decode::Error::message("Failed to create Document Content")
                })?;

                Ok(InnerCatalystSignedDocument {
                    metadata,
                    content,
                    signatures,
                }
                .into())
            },
            _ => {
                Err(minicbor::decode::Error::message(
                    "Failed to decode Catalyst Signed Document",
                ))
            },
        }
    }
}

impl Encode<()> for CatalystSignedDocument {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut encode::Encoder<W>, _ctx: &mut (),
    ) -> Result<(), encode::Error<W::Error>> {
        let cose_sign = self.as_cose_sign().map_err(encode::Error::message)?;

        let cose_bytes = cose_sign.to_vec().map_err(|e| {
            minicbor::encode::Error::message(format!("Failed to encode COSE Sign document: {e}"))
        })?;

        e.writer_mut()
            .write_all(&cose_bytes)
            .map_err(|_| minicbor::encode::Error::message("Failed to encode to CBOR"))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ed25519_dalek::SigningKey;
    use metadata::{ContentEncoding, ContentType};
    use rand::rngs::OsRng;

    use super::*;

    fn test_metadata() -> anyhow::Result<(UuidV7, UuidV4, Metadata)> {
        let uuid_v7 = UuidV7::new();
        let uuid_v4 = UuidV4::new();
        let section = "some section".to_string();
        let collabs = vec!["Alex1".to_string(), "Alex2".to_string()];
        let content_type = ContentType::Json;
        let content_encoding = ContentEncoding::Brotli;

        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "content-type": content_type.to_string(),
            "content-encoding": content_encoding.to_string(),
            "type": uuid_v4.to_string(),
            "id": uuid_v7.to_string(),
            "ver": uuid_v7.to_string(),
            "ref": {"id": uuid_v7.to_string()},
            "reply": {"id": uuid_v7.to_string(), "ver": uuid_v7.to_string()},
            "template": {"id": uuid_v7.to_string()},
            "section": section,
            "collabs": collabs,
            "campaign_id": {"id": uuid_v7.to_string()},
            "election_id":  uuid_v4.to_string(),
            "brand_id":  {"id": uuid_v7.to_string()},
            "category_id": {"id": uuid_v7.to_string()},
        }))
        .map_err(|_| anyhow::anyhow!("Invalid example metadata. This should not happen."))?;
        Ok((uuid_v7, uuid_v4, metadata))
    }

    #[test]
    fn catalyst_signed_doc_cbor_roundtrip_test() {
        let (uuid_v7, uuid_v4, metadata) = test_metadata().unwrap();
        let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();

        let doc = Builder::new()
            .with_metadata(metadata.clone())
            .with_decoded_content(content.clone())
            .build()
            .unwrap();

        let bytes = minicbor::to_vec(doc).unwrap();
        let decoded: CatalystSignedDocument = bytes.as_slice().try_into().unwrap();

        assert_eq!(decoded.doc_type(), uuid_v4);
        assert_eq!(decoded.doc_id(), uuid_v7);
        assert_eq!(decoded.doc_ver(), uuid_v7);
        assert_eq!(decoded.doc_content().decoded_bytes(), &content);
        assert_eq!(decoded.doc_meta(), metadata.extra());
    }

    #[test]
    fn signature_verification_test() {
        let mut csprng = OsRng;
        let sk: SigningKey = SigningKey::generate(&mut csprng);
        let content = serde_json::to_vec(&serde_json::Value::Null).unwrap();
        let pk = sk.verifying_key();

        let kid_str = format!(
            "id.catalyst://cardano/{}/0/0",
            base64_url::encode(pk.as_bytes())
        );

        let kid = IdUri::from_str(&kid_str).unwrap();
        let (_, _, metadata) = test_metadata().unwrap();
        let signed_doc = Builder::new()
            .with_decoded_content(content)
            .with_metadata(metadata)
            .add_signature(sk.to_bytes(), kid.clone())
            .unwrap()
            .build()
            .unwrap();

        assert!(signed_doc.verify(|_| Ok(Some(pk))).is_ok());
        assert!(signed_doc.verify(|_| Ok(None)).is_err());
        assert!(signed_doc
            .verify(|_| Err(anyhow::anyhow!("some error")))
            .is_err());
    }
}
