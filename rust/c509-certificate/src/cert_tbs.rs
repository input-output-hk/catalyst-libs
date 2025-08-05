//! C509 certificate To Be Sign Certificate (TBS Certificate)

use minicbor::{encode::Write, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};

use crate::{
    big_uint::UnwrappedBigUint,
    extensions::Extensions,
    helper::{
        decode::{decode_bytes, decode_helper},
        encode::{encode_bytes, encode_helper},
    },
    issuer_sig_algo::IssuerSignatureAlgorithm,
    name::Name,
    subject_pub_key_algo::SubjectPubKeyAlgorithm,
    time::Time,
};

/// A struct represents a To Be Signed Certificate (TBS Certificate).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TbsCert {
    /// Certificate type.
    c509_certificate_type: u8,
    /// Serial number of the certificate.
    certificate_serial_number: UnwrappedBigUint,
    /// Issuer Signature Algorithm
    issuer_signature_algorithm: IssuerSignatureAlgorithm,
    /// Issuer
    issuer: Name,
    /// Validity not before.
    validity_not_before: Time,
    /// Validity not after.
    validity_not_after: Time,
    /// Subject
    subject: Name,
    /// Subject Public Key Algorithm
    subject_public_key_algorithm: SubjectPubKeyAlgorithm,
    /// Subject Public Key value
    subject_public_key: Vec<u8>,
    /// Extensions
    extensions: Extensions,
}

impl TbsCert {
    /// Create a new instance of TBS Certificate.
    /// If issuer is not provided, it will use the subject as the issuer.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c509_certificate_type: u8,
        certificate_serial_number: UnwrappedBigUint,
        issuer_signature_algorithm: IssuerSignatureAlgorithm,
        issuer: Option<Name>,
        validity_not_before: Time,
        validity_not_after: Time,
        subject: Name,
        subject_public_key_algorithm: SubjectPubKeyAlgorithm,
        subject_public_key: Vec<u8>,
        extensions: Extensions,
    ) -> Self {
        Self {
            c509_certificate_type,
            certificate_serial_number,
            issuer_signature_algorithm,
            issuer: issuer.unwrap_or_else(|| subject.clone()),
            validity_not_before,
            validity_not_after,
            subject,
            subject_public_key_algorithm,
            subject_public_key,
            extensions,
        }
    }

    /// Get the certificate type.
    #[must_use]
    pub fn c509_certificate_type(&self) -> u8 {
        self.c509_certificate_type
    }

    /// Get the certificate serial number.
    #[must_use]
    pub fn certificate_serial_number(&self) -> &UnwrappedBigUint {
        &self.certificate_serial_number
    }

    /// Get the issuer signature algorithm.
    #[must_use]
    pub fn get_issuer_signature_algorithm(&self) -> &IssuerSignatureAlgorithm {
        &self.issuer_signature_algorithm
    }

    /// Get the issuer.
    #[must_use]
    pub fn issuer(&self) -> &Name {
        &self.issuer
    }

    /// Get the validity not before.
    #[must_use]
    pub fn validity_not_before(&self) -> &Time {
        &self.validity_not_before
    }

    /// Get the validity not after.
    #[must_use]
    pub fn validity_not_after(&self) -> &Time {
        &self.validity_not_after
    }

    /// Get the subject.
    #[must_use]
    pub fn subject(&self) -> &Name {
        &self.subject
    }

    /// Get the subject public key algorithm.
    #[must_use]
    pub fn subject_public_key_algorithm(&self) -> &SubjectPubKeyAlgorithm {
        &self.subject_public_key_algorithm
    }

    /// Get the subject public key.
    #[must_use]
    pub fn subject_public_key(&self) -> &[u8] {
        &self.subject_public_key
    }

    /// Get the extensions.
    #[must_use]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Get the issuer signature algorithm.
    #[must_use]
    pub fn issuer_signature_algorithm(&self) -> &IssuerSignatureAlgorithm {
        &self.issuer_signature_algorithm
    }

    /// Convert the TBS Certificate to CBOR.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    pub fn to_cbor<W: Write>(&self) -> Result<Vec<u8>, minicbor::encode::Error<W::Error>> {
        let mut buf = Vec::new();
        let mut e = Encoder::new(&mut buf);
        self.encode(&mut e, &mut ())
            .map_err(minicbor::encode::Error::message)?;
        Ok(buf)
    }
}

impl Encode<()> for TbsCert {
    fn encode<W: Write>(
        &self,
        e: &mut Encoder<W>,
        ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        encode_helper(e, "Certificate type", ctx, &self.c509_certificate_type)?;
        self.certificate_serial_number.encode(e, ctx)?;
        self.issuer_signature_algorithm.encode(e, ctx)?;
        self.issuer.encode(e, ctx)?;
        self.validity_not_before.encode(e, ctx)?;
        self.validity_not_after.encode(e, ctx)?;
        self.subject.encode(e, ctx)?;
        self.subject_public_key_algorithm.encode(e, ctx)?;
        encode_bytes(e, "Subject Public Key", &self.subject_public_key)?;
        self.extensions.encode(e, ctx)?;
        Ok(())
    }
}

impl Decode<'_, ()> for TbsCert {
    fn decode(
        d: &mut Decoder<'_>,
        ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let cert_type = decode_helper(d, "Certificate type", ctx)?;
        let serial_number = UnwrappedBigUint::decode(d, ctx)?;
        let issuer_signature_algorithm = IssuerSignatureAlgorithm::decode(d, ctx)?;
        let issuer = Some(Name::decode(d, ctx)?);
        let not_before = Time::decode(d, ctx)?;
        let not_after = Time::decode(d, ctx)?;
        let subject = Name::decode(d, ctx)?;
        let subject_public_key_algorithm = SubjectPubKeyAlgorithm::decode(d, ctx)?;
        let subject_public_key = decode_bytes(d, "Subject Public Key")?;
        let extensions = Extensions::decode(d, ctx)?;

        Ok(TbsCert::new(
            cert_type,
            serial_number,
            issuer_signature_algorithm,
            issuer,
            not_before,
            not_after,
            subject,
            subject_public_key_algorithm,
            subject_public_key,
            extensions,
        ))
    }
}

// ------------------Test----------------------

// Notes
// - The test from https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
// currently uses `subject_public_key` id-ecPublicKey, which has special encoding and
// decoding that this crate does not yet support. Hence, this test has
// been modified to align with the current encoding and decoding.
// - Currently support natively signed c509 certificate, so all text strings
// are UTF-8 encoded and all attributeType SHALL be non-negative
// - Some Extension values are not supported yet.

#[cfg(test)]
pub(crate) mod test_tbs_cert {
    use asn1_rs::oid;

    use super::*;
    use crate::{
        attributes::attribute::{Attribute, AttributeValue},
        extensions::{
            alt_name::{AlternativeName, GeneralNamesOrText},
            extension::{Extension, ExtensionValue},
        },
        general_names::{
            general_name::{GeneralName, GeneralNameTypeRegistry, GeneralNameValue},
            other_name_hw_module::OtherNameHardwareModuleName,
            GeneralNames,
        },
        name::NameValue,
    };

    // Mnemonic: match mad promote group rival case
    const PUBKEY: [u8; 8] = [0x88, 0xD0, 0xB6, 0xB0, 0xB3, 0x7B, 0xAA, 0x46];

    // Test reference https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
    // A.1.  Example RFC 7925 profiled X.509 Certificate
    pub(crate) fn tbs_1() -> (TbsCert, String) {
        let tbs_certificate = (
            3,                             // c509_certificate_type
            128_269,                       // certificate_serial_number
            oid!(1.2.840 .10045 .4 .3 .2), // issuer_signature_algorithm (ecdsa-with-SHA256)
            (
                // issuer
                oid!(2.5.4 .3), // oid (commonName)
                "RFC test CA",  // value
                false,          // critical
            ),
            1_672_531_200, // validity_not_before
            1_767_225_600, // validity_not_after
            (
                // subject
                oid!(2.5.4 .3),            // oid (commonName)
                "01-23-45-FF-FE-67-89-AB", // value
                false,                     // critical
            ),
            oid!(1.2.840 .10045 .2 .1), /* subject_public_key_algorithm (id-ecPublicKey
                                         * prime256v1 P-256) */
            PUBKEY, // subject_public_key (modified from the example)
            (
                // extensions
                oid!(2.5.29 .15), // oid (keyUsage)
                1,                // value
                false,            // critical
            ),
        );

        let tbs_certificate_cbor = [
            "03",                       // c509_certificate_type
            "4301f50d",                 // certificate_serial_number
            "00",                       // issuer_signature_algorithm
            "6b5246432074657374204341", // issuer
            "1a63b0cd00",               // validity_not_before
            "1a6955b900",               // validity_not_after
            "47010123456789ab",         // subject
            "01",                       // subject_public_key_algorithm
            "4888d0b6b0b37baa46",       // subject_public_key
            "01",                       // extensions
        ];

        // Issuer
        let mut attr1 = Attribute::new(tbs_certificate.3 .0);
        attr1.add_value(AttributeValue::Text(tbs_certificate.3 .1.to_string()));
        let issuer = Name::new(NameValue::Attribute(vec![attr1]));

        // Subject
        let mut attr2 = Attribute::new(tbs_certificate.6 .0);
        attr2.add_value(AttributeValue::Text(tbs_certificate.6 .1.to_string()));
        let subject = Name::new(NameValue::Attribute(vec![attr2]));

        // Extensions
        let mut extensions = Extensions::new();
        extensions.add_extension(Extension::new(
            tbs_certificate.9 .0,
            ExtensionValue::Int(tbs_certificate.9 .1),
            tbs_certificate.9 .2,
        ));

        let data = TbsCert::new(
            tbs_certificate.0,
            UnwrappedBigUint::new(tbs_certificate.1),
            IssuerSignatureAlgorithm::new(tbs_certificate.2, None),
            Some(issuer),
            Time::new(tbs_certificate.4),
            Time::new(tbs_certificate.5),
            subject,
            SubjectPubKeyAlgorithm::new(tbs_certificate.7, None),
            tbs_certificate.8.to_vec(),
            extensions,
        );

        let concatenated: String = tbs_certificate_cbor.concat();

        (data, concatenated)
    }

    #[test]
    fn encode_decode_tbs_cert_1() {
        let (tbs_cert, tbs_cert_cbor) = tbs_1();

        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        tbs_cert
            .encode(&mut encoder, &mut ())
            .expect("Failed to encode TBS Certificate");

        assert_eq!(hex::encode(buffer.clone()), tbs_cert_cbor);

        let mut decoder = Decoder::new(&buffer);
        let decoded_tbs =
            TbsCert::decode(&mut decoder, &mut ()).expect("Failed to decode TBS Certificate");
        assert_eq!(decoded_tbs, tbs_cert);
    }

    // Test reference https://datatracker.ietf.org/doc/draft-ietf-cose-cbor-encoded-cert/11/
    // A.2.  Example IEEE 802.1AR profiled X.509 Certificate
    #[allow(clippy::too_many_lines)]
    fn tbs_2() -> (TbsCert, String) {
        let tbs_certificate = (
            3,                             // c509_certificate_type
            9_112_578_475_118_446_130,     // certificate_serial_number
            oid!(1.2.840 .10045 .4 .3 .2), // issuer_signature_algorithm (ecdsa-with-SHA256)
            [
                // issuer
                (
                    oid!(2.5.4 .6), // oid (C: countryName)
                    "US",           // value
                    false,          // critical
                ),
                (
                    oid!(2.5.4 .8), // oid (ST: stateOrProvinceName)
                    "CA",           // value
                    false,          // critical
                ),
                (
                    oid!(2.5.4 .10), // oid (O: organizationName)
                    "Example Inc",   // value
                    false,           // critical
                ),
                (
                    oid!(2.5.4 .11), // oid (OU: organizationalUnitName)
                    "certification", // value
                    false,           // critical
                ),
                (
                    oid!(2.5.4 .3), // oid (CN: commonName)
                    "802.1AR CA",   // value
                    false,          // critical
                ),
            ],
            1_548_934_156,   // validity_not_before
            253_402_300_799, // validity_not_after
            [
                // subject
                (
                    oid!(2.5.4 .6), // oid (C: countryName)
                    "US",           // value
                    false,          // critical
                ),
                (
                    oid!(2.5.4 .8), // oid (ST: stateOrProvinceName)
                    "CA",           // value
                    false,          // critical
                ),
                (
                    oid!(2.5.4 .7), // oid (L: localityName)
                    "LA",           // value
                    false,          // critical
                ),
                (
                    oid!(2.5.4 .10), // oid (O: organizationName)
                    "example Inc",   // value
                    false,           // critical
                ),
                (
                    oid!(2.5.4 .11), // oid (OU: organizationalUnitName)
                    "IoT",           // value
                    false,           // critical
                ),
                (
                    oid!(2.5.4 .5), // oid (serialNumber)
                    "Wt1234",       // value
                    false,          // critical
                ),
            ],
            oid!(1.2.840 .10045 .2 .1), /* subject_public_key_algorithm (id-ecPublicKey
                                         * prime256v1 P-256) */
            PUBKEY, // subject_public_key (modified from the example)
            (
                // extensions
                (
                    oid!(2.5.29 .19), // oid (basicConstraints)
                    -2,               // value
                    false,            // critical
                ),
                (
                    oid!(2.5.29 .14), // oid (subjectKeyIdentifier)
                    [
                        0x96, 0x60, 0x0D, 0x87, 0x16, 0xBF, 0x7F, 0xD0, 0xE7, 0x52, 0xD0, 0xAC,
                        0x76, 0x07, 0x77, 0xAD, 0x66, 0x5D, 0x02, 0xA0,
                    ], // value
                    false,            // critical
                ),
                (
                    oid!(2.5.29 .15), // oid (keyUsage)
                    5,                // value
                    true,             // critical
                ),
                (
                    oid!(2.5.29 .17), // oid (subjectAltName)
                    (
                        oid!(1.3.6 .1 .4 .1 .6175 .10 .1), // hwType
                        [0x01, 0x02, 0x03, 0x04],          // hwSerialNum
                    ),
                    false, // critical
                ),
            ),
        );

        let tbs_certificate_cbor = [
                "03", // c509_certificate_type
                "487e7661d7b54e4632", // certificate_serial_number
                "00", // issuer_signature_algorithm
                "8a0462555306624341086b4578616d706c6520496e63096d63657274696669636174696f6e016a3830322e314152204341", // issuer
                "1a5c52dc0c", // validity_not_before
                "f6", // validity_not_after
                "8c046255530662434105624c41086b6578616d706c6520496e630963496f540366577431323334", // subject
                "01", // subject_public_key_algorithm
                "4888d0b6b0b37baa46", // subject_public_key
                "840421015496600d8716bf7fd0e752d0ac760777ad665d02a0210503822082492b06010401b01f0a014401020304", // extensions
            ];

        // Issuer
        let mut attributes_1 = Vec::new();
        for i in 0..tbs_certificate.3.len() {
            let mut attr = Attribute::new(tbs_certificate.3.get(i).unwrap().0.clone());
            attr.add_value(AttributeValue::Text(
                tbs_certificate.3.get(i).unwrap().1.to_string(),
            ));
            attributes_1.push(attr);
        }
        let issuer = Name::new(NameValue::Attribute(attributes_1));

        // Subject
        let mut attributes_2 = Vec::new();
        for i in 0..tbs_certificate.6.len() {
            let mut attr = Attribute::new(tbs_certificate.6.get(i).unwrap().0.clone());
            attr.add_value(AttributeValue::Text(
                tbs_certificate.6.get(i).unwrap().1.to_string(),
            ));
            attributes_2.push(attr);
        }
        let subject = Name::new(NameValue::Attribute(attributes_2));

        // Extensions
        let mut extensions = Extensions::new();
        extensions.add_extension(Extension::new(
            tbs_certificate.9 .0 .0,
            ExtensionValue::Int(tbs_certificate.9 .0 .1),
            tbs_certificate.9 .0 .2,
        ));
        extensions.add_extension(Extension::new(
            tbs_certificate.9 .1 .0,
            ExtensionValue::Bytes(tbs_certificate.9 .1 .1.to_vec()),
            tbs_certificate.9 .1 .2,
        ));
        extensions.add_extension(Extension::new(
            tbs_certificate.9 .2 .0,
            ExtensionValue::Int(tbs_certificate.9 .2 .1),
            tbs_certificate.9 .2 .2,
        ));
        let mut gns = GeneralNames::new();
        let hw = OtherNameHardwareModuleName::new(
            tbs_certificate.9 .3 .1 .0,
            tbs_certificate.9 .3 .1 .1.to_vec(),
        );
        gns.add_general_name(GeneralName::new(
            GeneralNameTypeRegistry::OtherNameHardwareModuleName,
            GeneralNameValue::OtherNameHWModuleName(hw),
        ));

        extensions.add_extension(Extension::new(
            tbs_certificate.9 .3 .0,
            ExtensionValue::AlternativeName(AlternativeName::new(
                GeneralNamesOrText::GeneralNames(gns),
            )),
            false,
        ));

        let data = TbsCert::new(
            tbs_certificate.0,
            UnwrappedBigUint::new(tbs_certificate.1),
            IssuerSignatureAlgorithm::new(tbs_certificate.2, None),
            Some(issuer),
            Time::new(tbs_certificate.4),
            Time::new(tbs_certificate.5),
            subject,
            SubjectPubKeyAlgorithm::new(tbs_certificate.7, None),
            tbs_certificate.8.to_vec(),
            extensions,
        );

        let concatenated: String = tbs_certificate_cbor.concat();

        (data, concatenated)
    }

    #[test]
    fn encode_decode_tbs_cert_2() {
        let (tbs_cert, tbs_cert_cbor) = tbs_2();

        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        tbs_cert
            .encode(&mut encoder, &mut ())
            .expect("Failed to encode TBS Certificate");
        assert_eq!(hex::encode(buffer.clone()), tbs_cert_cbor);

        let mut decoder = Decoder::new(&buffer);
        let decoded_tbs =
            TbsCert::decode(&mut decoder, &mut ()).expect("Failed to decode TBS Certificate");
        assert_eq!(decoded_tbs, tbs_cert);
    }
}
