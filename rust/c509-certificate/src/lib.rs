//! CBOR Encoded X.509 Certificate (C509 Certificate) library
//!
//! This crate provides a functionality for generating C509 Certificate.
//!
//! Please refer to the [C509 Certificate](https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/) for more information.
//!
//! ## C509 certificate structure
//!
//! A C509 certificate is a CBOR encoded X.509 certificate. It consists of two main parts:
//! 1. `TBSCertificate`
//! 2. `issuerSignatureValue`
//!
//! In order to generate an unsigned C509 certificate, the TBS Certificate must be
//! provided. Then the unsigned C509 certificate will then be used to calculate the
//! issuerSignatureValue.
//!
//! # TBS Certificate
//!
//! The To Be Sign Certificate contains the following fields:
//!    * c509CertificateType: A certificate type, where 2 indicates a natively signed C509
//!      certificate following X.509 v3 or 3 indicates CBOR re-encoded X.509 v3 DER
//!      certificate.
//!    * certificateSerialNumber: A unique serial number for the certificate.
//!    * subjectPublicKeyAlgorithm: Specifies the cryptographic algorithm used for the
//!      `subjectPublicKey`.
//!    * issuer: The entity that issued the certificate. In the case of a self-signed
//!      certificate, the issuer is identical to the subject.
//!    * validityNotBefore: The duration for which the Certificate Authority (CA)
//!      guarantees it will retain information regarding the certificate's status on which
//!      the period begins.
//!    * validityNotAfter: The duration for which the Certificate Authority (CA)
//!      guarantees it will retain information regarding the certificate's status on which
//!      the period ends. This can be set to no expiry date.
//!    * subject: The entity associated with the public key stored in the subject public
//!      key field.
//!    * subjectPublicKey: The public key of the subject.
//!    * extensions: A list of extensions defined for X.509 v3 certificate, providing
//!      additional attributes for users or public keys, and for managing relationships
//!      between Certificate Authorities (CAs).
//!    * issuerSignatureAlgorithm: The algorithm used to sign the certificate (must be the
//!      algorithm uses to create `IssuerSignatureValue`).
//

use anyhow::anyhow;
use c509::C509;
use cert_tbs::TbsCert;
use minicbor::{Decode, Encode};
use signing::{PrivateKey, PublicKey};

pub use crate::extensions::extension::data::C509ExtensionType;

pub mod algorithm_identifier;
pub mod attributes;
pub mod big_uint;
pub mod c509;
pub mod cert_tbs;
pub mod extensions;
pub mod general_names;
mod helper;
pub mod issuer_sig_algo;
pub mod name;
pub mod oid;
pub mod signing;
pub mod subject_pub_key_algo;
mod tables;
pub mod time;
pub mod wasm_binding;

/// Generate a signed or unsigned C509 certificate.
///
/// # Arguments
/// - `tbs_cert` - A TBS certificate.
/// - `private_key` - An optional private key, if provided certificate is signed.
///
/// # Returns
/// Returns a signed or unsigned C509 certificate.
///
/// # Errors
///
/// Returns an error if the generated data is invalid.

pub fn generate(tbs_cert: &TbsCert, private_key: Option<&PrivateKey>) -> anyhow::Result<Vec<u8>> {
    // Encode the TbsCert
    let encoded_tbs = {
        let mut buffer = Vec::new();
        let mut encoder = minicbor::Encoder::new(&mut buffer);
        tbs_cert.encode(&mut encoder, &mut ())?;
        buffer
    };
    let sign_data = private_key.map(|pk| pk.sign(&encoded_tbs));

    // Encode the whole C509 certificate including `TbSCert` and `issuerSignatureValue`
    let encoded_c509 = {
        let mut buffer = Vec::new();
        let mut encoder = minicbor::Encoder::new(&mut buffer);
        let c509 = C509::new(tbs_cert.clone(), sign_data);
        c509.encode(&mut encoder, &mut ())?;
        buffer
    };
    Ok(encoded_c509)
}

/// Verify the signature of a C509 certificate.
///
/// # Arguments
/// - `c509` - The cbor encoded C509 certificate to verify.
/// - `public_key` - The public key used to verify the certificate.
///
/// # Errors
/// Returns an error if the `issuer_signature_value` is invalid or the signature cannot be
/// verified.
pub fn verify(c509: &[u8], public_key: &PublicKey) -> anyhow::Result<()> {
    let mut d = minicbor::Decoder::new(c509);
    let c509 = C509::decode(&mut d, &mut ())?;
    let mut encoded_tbs = Vec::new();
    let mut encoder = minicbor::Encoder::new(&mut encoded_tbs);
    c509.tbs_cert().encode(&mut encoder, &mut ())?;
    let issuer_sig = c509.issuer_signature_value().clone().ok_or(anyhow!(
        "Signature verification failed, No issuer signature"
    ))?;
    public_key.verify(&encoded_tbs, &issuer_sig)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use cert_tbs::test_tbs_cert::tbs_1;
    use signing::tests::private_key_str;

    use super::*;

    #[test]
    fn test_generate_and_verify_signed_c509_cert() {
        let (tbs_cert, _) = tbs_1();

        let private_key = FromStr::from_str(&private_key_str()).expect(
            "Cannot create
private key",
        );

        let signed_c509 = generate(&tbs_cert, Some(&private_key))
            .expect("Failed to generate signed C509 certificate");

        assert!(verify(&signed_c509, &private_key.public_key()).is_ok());
    }
}
