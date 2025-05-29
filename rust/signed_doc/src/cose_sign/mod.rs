//! Catalyst Signed Document Builder.

/// Encoding helpers.
mod helpers;

use std::{convert::Infallible, fmt::Debug, sync::Arc};

use catalyst_types::catalyst_id::CatalystId;

use helpers::{
    encode_cose_sign, encode_cose_signature, encode_headers, encode_tbs_data, encoed_kid_header,
};
use indexmap::IndexMap;

pub type VecEncodeError = minicbor::encode::Error<Infallible>;

/// [RFC9052-CoseSign] builder without unprotected fields.
///
/// [RFC9052-CoseSign]: https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si
#[derive(Debug, Default)]
pub struct CoseSignBuilder {
    /// Mapping from encoded keys to encoded values within COSE protected header.
    protected: IndexMap<Vec<u8>, Vec<u8>>,
    /// Encoded COSE payload.
    payload: Option<Arc<[u8]>>,
}

impl CoseSignBuilder {
    /// Start building a [`CoseSign`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets COSE payload bytes. If content is encoded, it should be aligned with the
    /// encoding algorithm from the `content-encoding` field.
    #[must_use]
    pub fn with_payload<T>(&mut self, payload: T) -> &mut Self
    where
        Arc<[u8]>: From<T>,
    {
        self.payload = Some(payload.into());
        self
    }

    /// Add a protected header.
    ///
    /// If the key is already present, the value is updated.
    ///
    /// # Errors
    ///
    /// - Fails if it the CBOR encoding fails.
    /// - Fails if the key is already present.
    pub fn add_protected_header<C, K, V>(
        &mut self, ctx: &mut C, key: K, v: V,
    ) -> Result<&mut Self, VecEncodeError>
    where
        K: minicbor::Encode<C> + Debug,
        V: minicbor::Encode<C>,
    {
        let (encoded_key, encoded_v) = (
            minicbor::to_vec_with(&key, ctx)?,
            minicbor::to_vec_with(v, ctx)?,
        );
        let indexmap::map::Entry::Vacant(entry) = self.protected.entry(encoded_key) else {
            return Err(VecEncodeError::message(format!(
                "Trying to build a CoseSign with duplicate protected keys (key: {key:?})"
            )));
        };
        entry.insert(encoded_v);
        Ok(self)
    }

    /// Wraps around [`Self::add_protected_header`].
    /// Doesn't add fields with values equal to [`Default::default`].
    pub fn add_protected_header_if_not_default<C, K, V>(
        &mut self, ctx: &mut C, key: K, v: V,
    ) -> Result<&mut Self, VecEncodeError>
    where
        K: minicbor::Encode<C> + Debug,
        V: minicbor::Encode<C> + Default + PartialEq,
    {
        if V::default() == v {
            Ok(self)
        } else {
            self.add_protected_header(ctx, key, v)
        }
    }

    /// Encode [`Self::metadata`] by [`make_metadata_header`] with fields in insertion order.
    // Question: maybe this should be cached (e.g. frozen once filled)?
    fn encode_protected_header(&self) -> Vec<u8> {
        // This iterates in insertion order.
        let metadata_fields = self
            .protected
            .iter()
            .map(|(key, v)| (key.as_slice(), v.as_slice()));
        encode_headers(metadata_fields)
    }

    fn to_cose_sign(&self) -> CoseSign {
        let protected = self.encode_protected_header();
        CoseSign {
            protected,
            payload: self.payload.clone(),
            signatures: vec![],
        }
    }

    /// Add a signature.
    ///
    /// Returns [`CoseSign`], which implements [`minicbor::Encode`].
    /// More signatures can then be added with [`CoseSign::add_signature`].
    ///
    /// # Errors
    ///
    /// - If CBOR encoding of the [`CatalystId`] fails.
    pub fn add_signature<F: FnOnce(Vec<u8>) -> Vec<u8>>(
        &self, kid: CatalystId, sign_fn: F,
    ) -> Result<CoseSign, VecEncodeError> {
        let mut signer = self.to_cose_sign();
        signer.add_signature(kid, sign_fn)?;
        Ok(signer)
    }
}

/// [RFC9052-CoseSign](https://datatracker.ietf.org/doc/html/rfc9052).
pub struct CoseSign {
    /// Encoded COSE protected header.
    protected: Vec<u8>,
    /// Encoded COSE payload.
    payload: Option<Arc<[u8]>>,
    /// Encoded COSE signatures.
    signatures: Vec<Vec<u8>>,
}

impl CoseSign {
    /// Start building a [`CoseSign`]. Shortcut for the [`CoseSignBuilder::new`].
    pub fn builder() -> CoseSignBuilder {
        CoseSignBuilder::new()
    }

    /// Add another signature to the [`CoseSign`].
    ///
    /// # Errors
    ///
    /// - If CBOR encoding of the [`CatalystId`] fails.
    pub fn add_signature<F: FnOnce(Vec<u8>) -> Vec<u8>>(
        &mut self, kid: CatalystId, sign_fn: F,
    ) -> Result<&mut Self, VecEncodeError> {
        let kid_str = kid.to_string().into_bytes();
        let signature_header = encoed_kid_header(kid_str.as_slice())?;

        let tbs_data =
            encode_tbs_data(&self.protected, &signature_header, self.payload.as_deref())?;
        let signature_bytes = sign_fn(tbs_data);

        // This shouldn't fail.
        let signature = encode_cose_signature(&signature_header, &signature_bytes)?;
        self.signatures.push(signature);

        Ok(self)
    }
}

impl<C> minicbor::Encode<C> for CoseSign {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, _: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_cose_sign(
            e,
            &self.protected,
            self.payload.as_deref(),
            &self.signatures,
        )
    }
}
