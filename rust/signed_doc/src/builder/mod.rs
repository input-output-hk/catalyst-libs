//! Catalyst Signed Document Builder.

#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items
)]

pub mod brand_parameters;
pub mod brand_parameters_form_template;
pub mod campaign_parameters;
pub mod campaign_parameters_form_template;
pub mod category_parameters;
pub mod category_parameters_form_template;
pub mod contest_ballot;
pub mod contest_ballot_checkpoint;
pub mod contest_delegation;
pub mod contest_parameters;
pub mod contest_parameters_form_template;
pub mod ed25519;
pub mod proposal;
pub mod proposal_comment;
pub mod proposal_comment_form_template;
pub mod proposal_form_template;
pub mod proposal_submission_action;
pub mod rep_nomination;
pub mod rep_nomination_form_template;
pub mod rep_profile;
pub mod rep_profile_form_template;

use std::io::Write;

pub use brand_parameters::brand_parameters_doc;
pub use brand_parameters_form_template::brand_parameters_form_template_doc;
pub use campaign_parameters::campaign_parameters_doc;
pub use campaign_parameters_form_template::campaign_parameters_form_template_doc;
use catalyst_types::catalyst_id::CatalystId;
pub use category_parameters::category_parameters_doc;
pub use category_parameters_form_template::category_parameters_form_template_doc;
use cbork_utils::with_cbor_bytes::WithCborBytes;
pub use contest_ballot::contest_ballot_doc;
pub use contest_ballot_checkpoint::contest_ballot_checkpoint_doc;
pub use contest_delegation::contest_delegation_doc;
pub use contest_parameters::contest_parameters_doc;
pub use contest_parameters_form_template::contest_parameters_form_template_doc;
pub use proposal::proposal_doc;
pub use proposal_comment::proposal_comment_doc;
pub use proposal_comment_form_template::proposal_comment_form_template_doc;
pub use proposal_form_template::proposal_form_template_doc;
pub use proposal_submission_action::proposal_submission_action_doc;
pub use rep_nomination::rep_nomination_doc;
pub use rep_nomination_form_template::rep_nomination_form_template_doc;
pub use rep_profile::rep_profile_doc;
pub use rep_profile_form_template::rep_profile_form_template_doc;

use crate::{
    CatalystSignedDocument, Content, ContentType, Metadata, Signatures,
    signature::{Signature, tbs_data},
};

/// Catalyst Signed Document Builder.
/// Its a type sage state machine which iterates type safely during different stages of
/// the Catalyst Signed Document build process:
/// Setting Metadata -> Setting Content -> Setting Signatures
pub type Builder = MetadataBuilder;

/// Only `metadata` builder part
pub struct MetadataBuilder {
    /// metadata
    metadata: Metadata,
}

/// Only `content` builder part
pub struct ContentBuilder {
    /// metadata
    metadata: Metadata,
    /// content
    content: Content,
}

/// Only `Signatures` builder part
pub struct SignaturesBuilder {
    /// metadata
    metadata: WithCborBytes<Metadata>,
    /// content
    content: Content,
    /// signatures
    signatures: Signatures,
}

impl MetadataBuilder {
    /// Start building a signed document
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            metadata: Metadata::default(),
        }
    }

    /// Set document metadata in JSON format
    /// Collect problem report if some fields are missing.
    ///
    /// # Errors
    /// - Fails if it is invalid metadata fields JSON object.
    pub fn with_json_metadata(
        mut self,
        json: serde_json::Value,
    ) -> anyhow::Result<ContentBuilder> {
        self.metadata = Metadata::from_json(json)?;
        Ok(ContentBuilder {
            metadata: self.metadata,
            content: Content::default(),
        })
    }
}

impl ContentBuilder {
    /// Prepares a `SignaturesBuilder` from the current `ContentBuilder`
    fn into_signatures_builder(self) -> anyhow::Result<SignaturesBuilder> {
        Ok(SignaturesBuilder {
            metadata: WithCborBytes::new(self.metadata, &mut ())?,
            content: self.content,
            signatures: Signatures::default(),
        })
    }

    /// Sets an empty content
    pub fn empty_content(self) -> anyhow::Result<SignaturesBuilder> {
        self.into_signatures_builder()
    }

    /// Sets the provided CBOR content, applying already set `content-encoding`.
    ///
    /// # Errors
    ///  - Verifies that `content-type` field is set to CBOR.
    ///  - Cannot encode provided CBOR.
    ///  - Compression failure.
    pub fn with_cbor_content<T: minicbor::Encode<()>>(
        mut self,
        content: T,
    ) -> anyhow::Result<SignaturesBuilder> {
        anyhow::ensure!(
            self.metadata.content_type() == Some(ContentType::Cbor),
            "Already set metadata field `content-type` is not CBOR value"
        );

        let mut buffer = Vec::new();
        let mut encoder = minicbor::Encoder::new(&mut buffer);
        content.encode(&mut encoder, &mut ())?;

        if let Some(encoding) = self.metadata.content_encoding() {
            self.content = encoding.encode(&buffer)?.into();
        } else {
            self.content = buffer.into();
        }

        self.into_signatures_builder()
    }

    /// Sets the provided raw CBOR content (bytes), applying already set
    /// `content-encoding`.
    ///
    /// # Errors
    ///  - Verifies that `content-type` field is set to CBOR.
    ///  - Compression failure.
    pub fn with_raw_cbor_content(
        mut self,
        content: &[u8],
    ) -> anyhow::Result<SignaturesBuilder> {
        anyhow::ensure!(
            self.metadata.content_type() == Some(ContentType::Cbor),
            "Already set metadata field `content-type` is not CBOR value"
        );

        if let Some(encoding) = self.metadata.content_encoding() {
            self.content = encoding.encode(content)?.into();
        } else {
            self.content = content.to_vec().into();
        }

        self.into_signatures_builder()
    }

    /// Set the provided JSON content, applying already set `content-encoding`.
    ///
    /// # Errors
    ///  - Verifies that `content-type` field is set to JSON
    ///  - Cannot serialize provided JSON
    ///  - Compression failure
    pub fn with_json_content(
        mut self,
        json: &serde_json::Value,
    ) -> anyhow::Result<SignaturesBuilder> {
        anyhow::ensure!(
            self.metadata.content_type() == Some(ContentType::Json)
                || self.metadata.content_type() == Some(ContentType::SchemaJson),
            "Already set metadata field `content-type` is not JSON value"
        );

        let content = serde_json::to_vec(&json)?;
        if let Some(encoding) = self.metadata.content_encoding() {
            self.content = encoding.encode(&content)?.into();
        } else {
            self.content = content.into();
        }

        self.into_signatures_builder()
    }
}

impl SignaturesBuilder {
    /// Add a signature to the document
    ///
    /// # Errors
    ///
    /// Fails if a `CatalystSignedDocument` cannot be created due to missing metadata or
    /// content, due to malformed data, or when the signed document cannot be
    /// converted into `coset::CoseSign`.
    pub fn add_signature(
        mut self,
        // TODO: use `Ed25519SigningKey` as an argument
        sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>,
        kid: CatalystId,
    ) -> anyhow::Result<Self> {
        if kid.is_id() {
            anyhow::bail!("Provided kid should be in a uri format, kid: {kid}");
        }

        self.signatures.push(build_signature(
            sign_fn,
            kid,
            &self.metadata,
            &self.content,
        )?);

        Ok(self)
    }

    /// Builds a document from the set `metadata`, `content` and `signatures`.
    ///
    /// # Errors:
    ///  - CBOR encoding/decoding failures
    pub fn build(self) -> anyhow::Result<CatalystSignedDocument> {
        let metadata_bytes = minicbor::to_vec(&self.metadata)?;
        let content_bytes = minicbor::to_vec(&self.content)?;
        let signature_bytes = minicbor::to_vec(&self.signatures)?;
        let doc = build_document(&metadata_bytes, &content_bytes, &signature_bytes)?;
        Ok(doc)
    }
}

/// Build document from the provided **CBOR encoded** `metadata`, `content` and
/// `signatures`.
fn build_document(
    metadata_bytes: &[u8],
    content_bytes: &[u8],
    signatures_bytes: &[u8],
) -> anyhow::Result<CatalystSignedDocument> {
    let mut e = minicbor::Encoder::new(Vec::new());
    // COSE_Sign tag
    // <!https://datatracker.ietf.org/doc/html/rfc8152#page-9>
    e.tag(minicbor::data::Tag::new(98))?;
    e.array(4)?;
    // protected headers (metadata fields)
    e.bytes(metadata_bytes)?;
    // empty unprotected headers
    e.map(0)?;
    // content
    e.writer_mut().write_all(content_bytes)?;
    // signatures
    e.writer_mut().write_all(signatures_bytes)?;
    CatalystSignedDocument::try_from(e.into_writer().as_slice())
}

/// Builds a `Signature` object by signing provided `metadata_bytes`, `content_bytes` and
/// `kid` params.
fn build_signature(
    sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>,
    kid: CatalystId,
    metadata: &WithCborBytes<Metadata>,
    content: &Content,
) -> anyhow::Result<Signature> {
    let data_to_sign = tbs_data(&kid, metadata, content)?;
    let sign_bytes = sign_fn(data_to_sign);
    Ok(Signature::new(kid, sign_bytes))
}

impl TryFrom<&CatalystSignedDocument> for SignaturesBuilder {
    type Error = anyhow::Error;

    fn try_from(value: &CatalystSignedDocument) -> Result<Self, Self::Error> {
        Ok(Self {
            metadata: value.0.metadata.clone(),
            content: value.0.content.clone(),
            signatures: value.0.signatures.clone(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use catalyst_types::uuid::UuidV4;
    use cbork_utils::with_cbor_bytes::WithCborBytes;

    use crate::metadata::SupportedField;

    /// A test version of the builder, which allows to build a not fully valid catalyst
    /// signed document
    #[derive(Default)]
    pub(crate) struct Builder {
        /// metadata
        metadata: super::Metadata,
        /// content
        content: super::Content,
        /// signatures
        signatures: super::Signatures,
    }

    impl Builder {
        /// Start building a signed document.
        ///
        /// The `type` metadata field is added because it is required by the
        /// `Metadata::from_fields` method.
        #[must_use]
        pub(crate) fn new() -> Self {
            Self::default().with_metadata_field(SupportedField::Type(UuidV4::new().into()))
        }

        /// Add provided `SupportedField` into the `Metadata`.
        pub(crate) fn with_metadata_field(
            mut self,
            field: SupportedField,
        ) -> Self {
            self.metadata.add_field(field);
            self
        }

        /// Set the content (COSE payload) to the document builder.
        /// It will set the content as its provided, make sure by yourself that
        /// `content-type` and `content-encoding` fields are aligned with the
        /// provided content bytes.
        pub(crate) fn with_content(
            mut self,
            content: Vec<u8>,
        ) -> Self {
            self.content = content.into();
            self
        }

        /// Add a signature to the document
        pub(crate) fn add_signature(
            mut self,
            sign_fn: impl FnOnce(Vec<u8>) -> Vec<u8>,
            kid: super::CatalystId,
        ) -> anyhow::Result<Self> {
            let metadata = WithCborBytes::new(self.metadata, &mut ())?;
            self.signatures.push(super::build_signature(
                sign_fn,
                kid,
                &metadata,
                &self.content,
            )?);
            self.metadata = metadata.inner();
            Ok(self)
        }

        /// Build a signed document with the collected error report.
        /// Could provide an invalid document.
        pub(crate) fn build(self) -> super::CatalystSignedDocument {
            let metadata_bytes = minicbor::to_vec(self.metadata).unwrap();
            let content_bytes = minicbor::to_vec(self.content).unwrap();
            let signature_bytes = minicbor::to_vec(self.signatures).unwrap();
            super::build_document(&metadata_bytes, &content_bytes, &signature_bytes).unwrap()
        }
    }
}
