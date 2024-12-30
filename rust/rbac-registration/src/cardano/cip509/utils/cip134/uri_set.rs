//! A set of [`Cip0134Uri`].

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context, Result};
use c509_certificate::{
    extensions::{alt_name::GeneralNamesOrText, extension::ExtensionValue},
    general_names::general_name::{GeneralNameTypeRegistry, GeneralNameValue},
    C509ExtensionType,
};
use der_parser::der::parse_der_sequence;
use tracing::debug;
use x509_cert::der::{oid::db::rfc5912::ID_CE_SUBJECT_ALT_NAME, Decode};

use crate::{
    cardano::cip509::{
        rbac::certs::{C509Cert, X509DerCert},
        utils::Cip0134Uri,
        validation::URI,
    },
    utils::general::decode_utf8,
};

/// A mapping from a certificate index to URIs contained within.
type UrisMap = HashMap<usize, Box<[Cip0134Uri]>>;

/// A set of [`Cip0134Uri`] contained in both x509 and c509 certificates stored in the
/// metadata part of [`Cip509`](crate::cardano::cip509::Cip509).
///
/// This structure uses [`Arc`] internally, so it is cheap to clone.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct Cip0134UriSet(Arc<Cip0134UriSetInner>);

/// Internal `Cip0134UriSet` data.
#[derive(Debug, Eq, PartialEq)]
struct Cip0134UriSetInner {
    /// URIs from x509 certificates.
    x_uris: UrisMap,
    /// URIs from c509 certificates.
    c_uris: UrisMap,
}

impl Cip0134UriSet {
    /// Creates a new `Cip0134UriSet` instance from the given certificates.
    ///
    /// # Errors
    /// - Invalid certificate.
    pub fn new(x509_certs: &[X509DerCert], c509_certs: &[C509Cert]) -> Result<Self> {
        let x_uris =
            extract_x509_uris(x509_certs).with_context(|| "Error processing X509 certificates")?;
        let c_uris =
            extract_c509_uris(c509_certs).with_context(|| "Error processing C509 certificates")?;
        Ok(Self(Arc::new(Cip0134UriSetInner { x_uris, c_uris })))
    }

    /// Returns a mapping from the x509 certificate index to URIs contained within.
    #[must_use]
    pub fn x_uris(&self) -> &UrisMap {
        &self.0.x_uris
    }

    /// Returns a mapping from the c509 certificate index to URIs contained within.
    #[must_use]
    pub fn c_uris(&self) -> &UrisMap {
        &self.0.c_uris
    }

    /// Returns `true` if both x509 and c509 certificate maps are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.x_uris().is_empty() && self.c_uris().is_empty()
    }
}

/// Iterates over X509 certificates and extracts CIP-0134 URIs.
fn extract_x509_uris(certificates: &[X509DerCert]) -> Result<UrisMap> {
    let mut result = UrisMap::new();

    for (index, cert) in certificates.iter().enumerate() {
        let X509DerCert::X509Cert(cert) = cert else {
            continue;
        };
        let cert = x509_cert::Certificate::from_der(cert)
            .with_context(|| "Failed to decode X509 certificate from DER")?;
        // Find the "subject alternative name" extension.
        let Some(extension) = cert
            .tbs_certificate
            .extensions
            .iter()
            .flatten()
            .find(|e| e.extn_id == ID_CE_SUBJECT_ALT_NAME)
        else {
            continue;
        };
        let (_, der) = parse_der_sequence(extension.extn_value.as_bytes()).with_context(|| {
            format!("Failed to parse DER sequence for Subject Alternative Name ({extension:?})")
        })?;

        let mut uris = Vec::new();
        for data in der.ref_iter() {
            if data.header.raw_tag() != Some(&[URI]) {
                continue;
            }
            let content = data
                .content
                .as_slice()
                .with_context(|| "Unable to process content for {data:?}")?;
            let address = match decode_utf8(content) {
                Ok(a) => a,
                Err(e) => {
                    // X.509 doesn't restrict the "alternative name" extension to be utf8 only, so
                    // we cannot treat this as error.
                    debug!("Ignoring {e:?}");
                    continue;
                },
            };
            let uri = match Cip0134Uri::parse(&address) {
                Ok(u) => u,
                Err(e) => {
                    // Same as above - simply skip non-confirming values.
                    debug!("Ignoring invalid CIP-0134 address: {e:?}");
                    continue;
                },
            };
            uris.push(uri);
        }

        if !uris.is_empty() {
            result.insert(index, uris.into_boxed_slice());
        }
    }

    Ok(result)
}

/// Iterates over C509 certificates and extracts CIP-0134 URIs.
fn extract_c509_uris(certificates: &[C509Cert]) -> Result<UrisMap> {
    let mut result = UrisMap::new();

    for (index, cert) in certificates.iter().enumerate() {
        let cert = match cert {
            C509Cert::C509Certificate(c) => c,
            C509Cert::C509CertInMetadatumReference(_) => {
                debug!("Ignoring unsupported metadatum reference");
                continue;
            },
            _ => continue,
        };

        for extension in cert.tbs_cert().extensions().extensions() {
            if extension.registered_oid().c509_oid().oid()
                != &C509ExtensionType::SubjectAlternativeName.oid()
            {
                continue;
            }
            let ExtensionValue::AlternativeName(alt_name) = extension.value() else {
                return Err(anyhow!("Unexpected extension value type for {extension:?}"));
            };
            let GeneralNamesOrText::GeneralNames(gen_names) = alt_name.general_name() else {
                return Err(anyhow!("Unexpected general name type: {extension:?}"));
            };

            let mut uris = Vec::new();
            for name in gen_names.general_names() {
                if *name.gn_type() != GeneralNameTypeRegistry::UniformResourceIdentifier {
                    continue;
                }
                let GeneralNameValue::Text(address) = name.gn_value() else {
                    return Err(anyhow!("Unexpected general name value format: {name:?}"));
                };
                let uri = Cip0134Uri::parse(address)
                    .with_context(|| format!("Failed to parse CIP-0134 address ({address})"))?;
                uris.push(uri);
            }

            if !uris.is_empty() {
                result.insert(index, uris.into_boxed_slice());
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use minicbor::{Decode, Decoder};
    use pallas::{
        codec::utils::Nullable,
        ledger::{
            addresses::{Address, Network},
            traverse::{MultiEraBlock, MultiEraTx},
        },
    };

    use crate::cardano::{cip509::Cip509, transaction::raw_aux_data::RawAuxData};

    // This lint is disabled locally because the `allow-indexing-slicing-in-tests` was added
    // very recently and isn't present in the stable clippy yet. Also it is impossible to use
    // `get(n).unwrap()` instead because Clippy will still complain (clippy::get-unwrap).
    #[allow(clippy::indexing_slicing)]
    #[test]
    fn set_new() {
        let block =
            hex::decode(include_str!("../../../../test_data/cardano/conway_1.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();
        let tx = &block.txs()[3];
        let cip509 = cip509(tx);
        let set = cip509.metadata.certificate_uris;
        assert!(!set.is_empty());
        assert!(set.c_uris().is_empty());

        let x_uris = set.x_uris();
        assert_eq!(x_uris.len(), 1);

        let uris = x_uris.get(&0).unwrap();
        assert_eq!(uris.len(), 1);

        let uri = &uris[0];
        // cSpell:disable
        assert_eq!(
            uri.uri(),
            "web+cardano://addr/stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr"
        );
        // cSpell:enable
        let Address::Stake(address) = uri.address() else {
            panic!("Unexpected address type");
        };
        assert_eq!(Network::Testnet, address.network());
        assert_eq!(
            "e075be10ec5c575caffb68b08c31470666d4fe1aeea07c16d6473903",
            address.payload().as_hash().to_string()
        );
    }

    fn cip509(tx: &MultiEraTx) -> Cip509 {
        let Nullable::Some(data) = tx.as_conway().unwrap().clone().auxiliary_data else {
            panic!("Auxiliary data is missing");
        };
        let data = RawAuxData::new(data.raw_cbor());
        let metadata = data.get_metadata(509).unwrap();

        let mut decoder = Decoder::new(metadata.as_slice());
        Cip509::decode(&mut decoder, &mut ()).unwrap()
    }
}
