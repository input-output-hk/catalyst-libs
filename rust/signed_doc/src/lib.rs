//! Catalyst documents signing crate

mod builder;
mod content;
pub mod doc_types;
mod metadata;
pub mod providers;
mod signature;
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
use metadata::{ContentEncoding, ContentType};
pub use metadata::{DocumentRef, ExtraFields, Metadata};
use minicbor::{decode, encode, Decode, Decoder, Encode};
use providers::VerifyingKeyProvider;
pub use signature::{IdUri, Signatures};

/// A problem report content string
const PROBLEM_REPORT_CTX: &str = "Catalyst Signed Document";

/// Inner type that holds the Catalyst Signed Document with parsing errors.
#[derive(Debug, Clone)]
struct InnerCatalystSignedDocument {
    /// Document Metadata
    metadata: Metadata,
    /// Document Content
    content: Content,
    /// Signatures
    signatures: Signatures,
    /// A comprehensive problem report, which could include a decoding errors along with
    /// the other validation errors
    report: ProblemReport,
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
    ///
    /// # Errros
    /// - Missing 'type' field.
    #[must_use]
    pub fn doc_type(&self) -> anyhow::Result<UuidV4> {
        self.inner.metadata.doc_type()
    }

    /// Return Document ID `UUIDv7`.
    ///
    /// # Errros
    /// - Missing 'id' field.
    #[must_use]
    pub fn doc_id(&self) -> anyhow::Result<UuidV7> {
        self.inner.metadata.doc_id()
    }

    /// Return Document Version `UUIDv7`.
    ///
    /// # Errros
    /// - Missing 'ver' field.
    #[must_use]
    pub fn doc_ver(&self) -> anyhow::Result<UuidV7> {
        self.inner.metadata.doc_ver()
    }

    /// Return document `Content`.
    #[must_use]
    pub fn doc_content(&self) -> &Content {
        &self.inner.content
    }

    /// Return document `ContentType`.
    ///
    /// # Errros
    /// - Missing 'content-type' field.
    #[must_use]
    pub fn doc_content_type(&self) -> anyhow::Result<ContentType> {
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

    /// Returns a collected problem report for the document.
    /// It accumulates all kind of errors, collected during the decoding, type based
    /// validation and signature verification.
    ///
    /// This is method is only for the public API usage, do not use it internally inside
    /// this crate.
    #[must_use]
    pub fn problem_report(&self) -> ProblemReport {
        self.report().clone()
    }

    /// Returns an internal problem report
    #[must_use]
    pub(crate) fn report(&self) -> &ProblemReport {
        &self.inner.report
    }

    /// Verify document signatures.
    /// Return true if all signatures are valid, otherwise return false.
    /// Also it imediatly return false, if document is already invalid.
    ///
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    pub async fn verify(&self, provider: &impl VerifyingKeyProvider) -> anyhow::Result<bool> {
        if self.report().is_problematic() {
            return Ok(false);
        }

        let Ok(cose_sign) = self.as_cose_sign() else {
            self.report().other(
                "Cannot build a COSE sign object",
                "During encoding signed document as COSE SIGN",
            );
            return Ok(false);
        };

        for (signature, kid) in self.signatures().cose_signatures_with_kids() {
            if let Some(pk) = provider.try_get_key(kid).await? {
                let tbs_data = cose_sign.tbs_data(&[], signature);
                match signature.signature.as_slice().try_into() {
                    Ok(signature_bytes) => {
                        let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
                        if let Err(e) = pk.verify_strict(&tbs_data, &signature) {
                            self.report().functional_validation(
                                &format!(
                                    "Verification failed for signature with Key ID {kid}: {e}"
                                ),
                                "During signature validation with verifying key",
                            );
                        }
                    },
                    Err(_) => {
                        self.report().invalid_value(
                            "cose signature",
                            &format!("{}", signature.signature.len()),
                            &format!("must be {}", ed25519_dalek::Signature::BYTE_SIZE),
                            "During encoding cose signature to bytes",
                        );
                    },
                }
            } else {
                self.report().other(
                    &format!("Missing public key for {kid}."),
                    "During public key extraction",
                );
            }
        }

        Ok(!self.report().is_problematic())
    }

    /// Returns a signed document `Builder` pre-loaded with the current signed document's
    /// data.
    ///
    /// # Errors
    /// Could fails if the `CatalystSignedDocument` object is not valid.
    #[must_use]
    pub fn into_builder(self) -> anyhow::Result<Builder> {
        Ok(Builder::new()
            .with_metadata(self.inner.metadata.clone())
            .with_decoded_content(self.doc_content().decoded_bytes()?.to_vec())
            .with_signatures(self.signatures().clone()))
    }

    /// Convert Catalyst Signed Document into `coset::CoseSign`
    ///
    /// # Errors
    /// Could fails if the `CatalystSignedDocument` object is not valid.
    fn as_cose_sign(&self) -> anyhow::Result<coset::CoseSign> {
        let protected_header = Header::try_from(&self.inner.metadata)
            .map_err(|e| anyhow::anyhow!("Failed to encode Document Metadata: {e}"))?;

        let content = if let Some(content_encoding) = self.doc_content_encoding() {
            self.doc_content().encoded_bytes(content_encoding)?
        } else {
            self.doc_content().decoded_bytes()?.to_vec()
        };

        let mut builder = coset::CoseSignBuilder::new()
            .protected(protected_header)
            .payload(content);

        for signature in self.signatures().cose_signatures() {
            builder = builder.add_signature(signature);
        }
        Ok(builder.build())
    }
}

impl Decode<'_, ()> for CatalystSignedDocument {
    fn decode(d: &mut Decoder<'_>, _ctx: &mut ()) -> Result<Self, decode::Error> {
        let start = d.position();
        d.skip()?;
        let end = d.position();
        let cose_bytes = d
            .input()
            .get(start..end)
            .ok_or(minicbor::decode::Error::end_of_input())?;

        let cose_sign = coset::CoseSign::from_slice(cose_bytes).map_err(|e| {
            minicbor::decode::Error::message(format!("Invalid COSE Sign document: {e}"))
        })?;

        let report = ProblemReport::new(PROBLEM_REPORT_CTX);
        let metadata = Metadata::from_protected_header(&cose_sign.protected, &report);
        let signatures = Signatures::from_cose_sig(&cose_sign.signatures, &report);

        let content = if let Some(payload) = cose_sign.payload {
            Content::from_encoded(
                payload,
                metadata.content_type().ok(),
                metadata.content_encoding(),
                &report,
            )
        } else {
            report.missing_field("COSE Sign Payload", "Missing document content (payload)");
            Content::default()
        };

        Ok(InnerCatalystSignedDocument {
            metadata,
            content,
            signatures,
            report,
        }
        .into())
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

    use ed25519_dalek::{SigningKey, VerifyingKey};
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
        let decoded: CatalystSignedDocument = minicbor::decode(bytes.as_slice()).unwrap();

        assert_eq!(decoded.doc_type().unwrap(), uuid_v4);
        assert_eq!(decoded.doc_id().unwrap(), uuid_v7);
        assert_eq!(decoded.doc_ver().unwrap(), uuid_v7);
        assert_eq!(decoded.doc_content().decoded_bytes().unwrap(), &content);
        assert_eq!(decoded.doc_meta(), metadata.extra());
    }

    struct Provider(anyhow::Result<Option<VerifyingKey>>);
    impl VerifyingKeyProvider for Provider {
        async fn try_get_key(
            &self, _kid: &IdUri,
        ) -> anyhow::Result<Option<ed25519_dalek::VerifyingKey>> {
            let res = self.0.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(*res)
        }
    }

    #[tokio::test]
    async fn signature_verification_test() {
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

        assert!(signed_doc.verify(&Provider(Ok(Some(pk)))).await.is_ok());
        assert!(signed_doc.verify(&Provider(Ok(None))).await.is_err());
        assert!(signed_doc
            .verify(&Provider(Err(anyhow::anyhow!("some error"))))
            .await
            .is_err());
    }
}
