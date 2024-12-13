//! A list of [`Cip0134Uri`].

use std::sync::Arc;

use anyhow::{anyhow, Result};
use c509_certificate::{
    extensions::{alt_name::GeneralNamesOrText, extension::ExtensionValue},
    general_names::general_name::{GeneralNameTypeRegistry, GeneralNameValue},
    C509ExtensionType,
};
use der_parser::der::parse_der_sequence;
use pallas::ledger::traverse::MultiEraTx;
use tracing::debug;
use x509_cert::der::{oid::db::rfc5912::ID_CE_SUBJECT_ALT_NAME, Decode};

use crate::{
    cardano::cip509::{
        rbac::{
            certs::{C509Cert, X509DerCert},
            Cip509RbacMetadata,
        },
        utils::Cip0134Uri,
        validation::URI,
        Cip509,
    },
    utils::general::decode_utf8,
};

/// A list of [`Cip0134Uri`].
///
/// This structure uses [`Arc`] internally, so it is cheap to clone.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct Cip0134UriList {
    /// An internal list of URIs.
    uris: Arc<[Cip0134Uri]>,
}

impl Cip0134UriList {
    /// Creates a new `Cip0134UriList` instance from the given `Cip509`.
    ///
    /// # Errors
    /// - Unsupported transaction era.
    pub fn new(cip509: &Cip509, tx: &MultiEraTx) -> Result<Self> {
        if !matches!(tx, MultiEraTx::Conway(_)) {
            return Err(anyhow!("Unsupported transaction era ({})", tx.era()));
        }

        let metadata = &cip509.x509_chunks.0;
        let mut uris = process_x509_certificates(metadata);
        uris.extend(process_c509_certificates(metadata));

        Ok(Self { uris: uris.into() })
    }

    /// Returns an iterator over the contained Cip0134 URIs.
    pub fn iter(&self) -> impl Iterator<Item = &Cip0134Uri> {
        self.uris.iter()
    }

    /// Returns a slice with all URIs in the list.
    #[must_use]
    pub fn as_slice(&self) -> &[Cip0134Uri] {
        &self.uris
    }
}

/// Iterates over X509 certificates and extracts CIP-0134 URIs.
fn process_x509_certificates(metadata: &Cip509RbacMetadata) -> Vec<Cip0134Uri> {
    let mut result = Vec::new();

    for cert in metadata.x509_certs.iter().flatten() {
        let X509DerCert::X509Cert(cert) = cert else {
            continue;
        };
        let cert = match x509_cert::Certificate::from_der(cert) {
            Ok(cert) => cert,
            Err(e) => {
                debug!("Failed to decode x509 certificate DER: {e:?}");
                continue;
            },
        };
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
        let der = match parse_der_sequence(extension.extn_value.as_bytes()) {
            Ok((_, der)) => der,
            Err(e) => {
                debug!("Failed to parse DER sequence for Subject Alternative Name ({extension:?}): {e:?}");
                continue;
            },
        };
        for data in der.ref_iter() {
            if data.header.raw_tag() != Some(&[URI]) {
                continue;
            }
            let content = match data.content.as_slice() {
                Ok(c) => c,
                Err(e) => {
                    debug!("Unable to process content for {data:?}: {e:?}");
                    continue;
                },
            };
            let address = match decode_utf8(content) {
                Ok(a) => a,
                Err(e) => {
                    debug!("Failed to decode content of {data:?}: {e:?}");
                    continue;
                },
            };
            match Cip0134Uri::parse(&address) {
                Ok(a) => result.push(a),
                Err(e) => {
                    debug!("Failed to parse CIP-0134 address ({address}): {e:?}");
                },
            }
        }
    }

    result
}

/// Iterates over C509 certificates and extracts CIP-0134 URIs.
fn process_c509_certificates(metadata: &Cip509RbacMetadata) -> Vec<Cip0134Uri> {
    let mut result = Vec::new();

    for cert in metadata.c509_certs.iter().flatten() {
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
                debug!("Unexpected extension value type for {extension:?}");
                continue;
            };
            let GeneralNamesOrText::GeneralNames(gen_names) = alt_name.general_name() else {
                debug!("Unexpected general name type: {extension:?}");
                continue;
            };
            for name in gen_names.general_names() {
                if *name.gn_type() != GeneralNameTypeRegistry::UniformResourceIdentifier {
                    continue;
                }
                let GeneralNameValue::Text(address) = name.gn_value() else {
                    debug!("Unexpected general name value format: {name:?}");
                    continue;
                };
                match Cip0134Uri::parse(address) {
                    Ok(a) => result.push(a),
                    Err(e) => {
                        debug!("Failed to parse CIP-0134 address ({address}): {e:?}");
                    },
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use minicbor::{Decode, Decoder};
    use pallas::{
        codec::utils::Nullable,
        ledger::{
            addresses::{Address, Network},
            traverse::MultiEraBlock,
        },
    };

    use super::*;
    use crate::cardano::transaction::raw_aux_data::RawAuxData;

    // This lint is disabled locally because unfortunately there is no
    // `allow-indexing-slicing-in-tests` option. Also it is impossible to use
    // `get(n).unwrap()` instead because Clippy will still complain (clippy::get-unwrap).
    #[allow(clippy::indexing_slicing)]
    #[test]
    fn list_new() {
        let block =
            hex::decode(include_str!("../../../../test_data/cardano/conway_1.block")).unwrap();
        let block = MultiEraBlock::decode(&block).unwrap();
        let tx = &block.txs()[3];
        let cip509 = cip509(tx);

        let list = Cip0134UriList::new(&cip509, tx).unwrap();
        assert_eq!(list.as_slice().len(), 1);
        // cSpell:disable
        assert_eq!(
            list.as_slice()[0].uri(),
            "web+cardano://addr/stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr"
        );
        // cSpell:enable
        let Address::Stake(address) = list.as_slice()[0].address() else {
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
