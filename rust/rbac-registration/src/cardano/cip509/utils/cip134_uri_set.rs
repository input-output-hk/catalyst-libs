//! A set of [`Cip0134Uri`].

use std::{collections::HashMap, sync::Arc};

use c509_certificate::{
    extensions::{alt_name::GeneralNamesOrText, extension::ExtensionValue},
    general_names::general_name::{GeneralNameTypeRegistry, GeneralNameValue},
    C509ExtensionType,
};
use cardano_blockchain_types::{Cip0134Uri, StakeAddress};
use catalyst_types::problem_report::ProblemReport;
use der_parser::der::parse_der_sequence;
use pallas::ledger::addresses::Address;
use tracing::debug;
use x509_cert::der::oid::db::rfc5912::ID_CE_SUBJECT_ALT_NAME;

use crate::cardano::cip509::{
    rbac::{C509Cert, Cip509RbacMetadata, X509DerCert},
    validation::URI,
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
#[derive(Debug, Clone, Eq, PartialEq)]
struct Cip0134UriSetInner {
    /// URIs from x509 certificates.
    x_uris: UrisMap,
    /// URIs from c509 certificates.
    c_uris: UrisMap,
}

impl Cip0134UriSet {
    /// Creates a new `Cip0134UriSet` instance from the given certificates.
    #[must_use]
    pub fn new(
        x509_certs: &[X509DerCert], c509_certs: &[C509Cert], report: &ProblemReport,
    ) -> Self {
        let x_uris = extract_x509_uris(x509_certs, report);
        let c_uris = extract_c509_uris(c509_certs, report);
        Self(Arc::new(Cip0134UriSetInner { x_uris, c_uris }))
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

    /// Returns a list of stake addresses by the given index.
    #[must_use]
    pub fn stake_addresses(&self, index: usize) -> Vec<StakeAddress> {
        let mut result = Vec::new();

        if let Some(uris) = self.x_uris().get(&index) {
            result.extend(convert_stake_addresses(uris));
        }
        if let Some(uris) = self.c_uris().get(&index) {
            result.extend(convert_stake_addresses(uris));
        }

        result
    }

    /// Return the updated URIs set.
    ///
    /// The resulting set includes all the data from both the original and a new one. In
    /// the following example for brevity we only consider one type of uris:
    /// ```text
    /// // Original data:
    /// 0: [uri_1]
    /// 1: [uri_2, uri_3]
    ///
    /// // New data:
    /// 0: undefined
    /// 1: deleted
    /// 2: [uri_4]
    ///
    /// // Resulting data:
    /// 0: [uri_1]
    /// 2: [uri_4]
    /// ```
    #[must_use]
    pub fn update(self, metadata: &Cip509RbacMetadata) -> Self {
        if self == metadata.certificate_uris {
            // Nothing to update.
            return self;
        }

        let Cip0134UriSetInner {
            mut x_uris,
            mut c_uris,
        } = Arc::unwrap_or_clone(self.0);

        for (index, cert) in metadata.x509_certs.iter().enumerate() {
            match cert {
                X509DerCert::Undefined => {
                    // The certificate wasn't changed - there is nothing to do.
                },
                X509DerCert::Deleted => {
                    x_uris.remove(&index);
                },
                X509DerCert::X509Cert(_) => {
                    if let Some(uris) = metadata.certificate_uris.x_uris().get(&index) {
                        x_uris.insert(index, uris.clone());
                    }
                },
            }
        }

        for (index, cert) in metadata.c509_certs.iter().enumerate() {
            match cert {
                C509Cert::Undefined => {
                    // The certificate wasn't changed - there is nothing to do.
                },
                C509Cert::Deleted => {
                    c_uris.remove(&index);
                },
                C509Cert::C509CertInMetadatumReference(_) => {
                    debug!("Ignoring unsupported metadatum reference");
                },
                C509Cert::C509Certificate(_) => {
                    if let Some(uris) = metadata.certificate_uris.c_uris().get(&index) {
                        c_uris.insert(index, uris.clone());
                    }
                },
            }
        }

        Self(Arc::new(Cip0134UriSetInner { x_uris, c_uris }))
    }
}

/// Iterates over X509 certificates and extracts CIP-0134 URIs.
fn extract_x509_uris(certificates: &[X509DerCert], report: &ProblemReport) -> UrisMap {
    let mut result = UrisMap::new();
    let context = "Extracting URIs from X509 certificates in Cip509 metadata";

    for (index, cert) in certificates.iter().enumerate() {
        let X509DerCert::X509Cert(cert) = cert else {
            continue;
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
        let Ok((_, der)) = parse_der_sequence(extension.extn_value.as_bytes()) else {
            report.other(
                &format!(
                    "Failed to parse DER sequence for Subject Alternative Name ({extension:?})"
                ),
                context,
            );
            continue;
        };

        let mut uris = Vec::new();
        for data in der.ref_iter() {
            if data.header.raw_tag() != Some(&[URI]) {
                continue;
            }
            let Ok(bytes) = data.content.as_slice() else {
                report.other(&format!("Unable to process content for {data:?}"), context);
                continue;
            };
            match Cip0134Uri::try_from(bytes) {
                Ok(u) => uris.push(u),
                Err(e) => {
                    // X.509 doesn't restrict the "alternative name" extension to be utf8 only, so
                    // we cannot treat this as error.
                    debug!("Ignoring invalid CIP-0134 address: {e:?}");
                    continue;
                },
            };
        }

        if !uris.is_empty() {
            result.insert(index, uris.into_boxed_slice());
        }
    }

    result
}

/// Iterates over C509 certificates and extracts CIP-0134 URIs.
fn extract_c509_uris(certificates: &[C509Cert], report: &ProblemReport) -> UrisMap {
    let mut result = UrisMap::new();
    let context = "Extracting URIs from C509 certificates in Cip509 metadata";

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
                report.other(
                    &format!("Unexpected extension value type for {extension:?}"),
                    context,
                );
                continue;
            };
            let GeneralNamesOrText::GeneralNames(gen_names) = alt_name.general_name() else {
                report.other(
                    &format!("Unexpected general name type: {extension:?}"),
                    context,
                );
                continue;
            };

            let mut uris = Vec::new();
            for name in gen_names.general_names() {
                if *name.gn_type() != GeneralNameTypeRegistry::UniformResourceIdentifier {
                    continue;
                }
                let GeneralNameValue::Text(address) = name.gn_value() else {
                    report.other(
                        &format!("Unexpected general name value format: {name:?}"),
                        context,
                    );
                    continue;
                };
                match Cip0134Uri::parse(address) {
                    Ok(u) => uris.push(u),
                    Err(e) => {
                        debug!("Ignoring invalid CIP-0134 address: {e:?}");
                        continue;
                    },
                };
            }

            if !uris.is_empty() {
                result.insert(index, uris.into_boxed_slice());
            }
        }
    }

    result
}

/// Converts a list of `Cip0134Uri` to a list of stake addresses.
fn convert_stake_addresses(uris: &[Cip0134Uri]) -> Vec<StakeAddress> {
    uris.iter()
        .filter_map(|uri| {
            match uri.address() {
                Address::Stake(a) => Some(a.clone().into()),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use pallas::ledger::addresses::{Address, Network};

    use crate::{cardano::cip509::Cip509, utils::test};

    #[test]
    fn set_new() {
        let data = test::block_1();
        let cip509 = Cip509::new(&data.block, data.txn_index, &[])
            .unwrap()
            .unwrap();
        assert!(
            !cip509.report().is_problematic(),
            "Failed to decode Cip509: {:?}",
            cip509.report()
        );

        let set = cip509.certificate_uris().unwrap();
        assert!(!set.is_empty());
        assert!(set.c_uris().is_empty());

        let x_uris = set.x_uris();
        assert_eq!(x_uris.len(), 1);

        let uris = x_uris.get(&0).unwrap();
        assert_eq!(uris.len(), 1);

        let uri = uris.first().unwrap();
        assert_eq!(
            uri.uri(),
            format!("web+cardano://addr/{}", data.stake_addr.unwrap())
        );
        let Address::Stake(address) = uri.address() else {
            panic!("Unexpected address type");
        };
        assert_eq!(Network::Testnet, address.network());
        assert_eq!(
            "e075be10ec5c575caffb68b08c31470666d4fe1aeea07c16d6473903",
            address.payload().as_hash().to_string()
        );
    }
}
