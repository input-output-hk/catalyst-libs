//! C509 certificate CLI

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use asn1_rs::{oid, Oid};
use c509_certificate::{
    attributes::attribute::Attribute,
    big_uint::UnwrappedBigUint,
    cert_tbs::TbsCert,
    extensions::Extensions,
    issuer_sig_algo::IssuerSignatureAlgorithm,
    name::{Name, NameValue},
    signing::{PrivateKey, PublicKey},
    subject_pub_key_algo::SubjectPubKeyAlgorithm,
    time::Time,
};
use chrono::{DateTime, Utc};
use clap::Parser;
use hex::ToHex;
use minicbor::Decode;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Commands for C509 certificate generation, verification and decoding
#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Cli {
    /// Generate C509 certificate, if private key is provided, self-signed certificate
    /// will be generated.
    Generate {
        /// JSON file with information to create C509 certificate.
        #[clap(short = 'f', long)]
        json_file: PathBuf,
        /// Optional output path that the generated C509 will be written to.
        #[clap(short, long)]
        output: Option<PathBuf>,
        /// Optional private key file, if provided, self-signed certificate will be
        /// generated. Currently support only PEM format.
        #[clap(long)]
        private_key: Option<PathBuf>,
        #[clap(long)]
        /// Optional key type.
        key_type: Option<String>,
    },

    /// C509 certificate signature verification.
    Verify {
        /// C509 certificate file
        #[clap(short, long)]
        file: PathBuf,
        /// Public key file. Currently support only PEM format.
        #[clap(long)]
        public_key: PathBuf,
    },

    /// Decode C509 certificate back to JSON.
    Decode {
        /// C509 certificate file.
        #[clap(short, long)]
        file: PathBuf,
        /// Optional output path of C509 certificate information in JSON format.
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}

impl Cli {
    /// Function to execute the commands.
    pub(crate) fn exec() -> anyhow::Result<()> {
        let cli = Cli::parse();

        match cli {
            Cli::Generate {
                json_file,
                output,
                private_key,
                key_type,
            } => {
                let sk = match private_key {
                    Some(key) => Some(PrivateKey::from_file(key)?),
                    None => None,
                };

                generate(&json_file, output, sk.as_ref(), key_type)
            },
            Cli::Verify { file, public_key } => verify(&file, public_key),
            Cli::Decode { file, output } => decode(&file, output),
        }
    }
}

/// A struct representing the JSON format of C509 certificate.
#[derive(Deserialize, Serialize)]
struct C509Json {
    /// Indicate whether the certificate is self-signed.
    self_signed: bool,
    /// Optional certificate type, if not provided, set to 2 as self-signed.
    certificate_type: Option<u8>,
    /// Optional serial number of the certificate,
    /// if not provided, a random number will be generated.
    serial_number: Option<UnwrappedBigUint>,
    /// Optional issuer signature algorithm of the certificate,
    /// if not provided, set to Ed25519.
    issuer_signature_algorithm: Option<IssuerSignatureAlgorithm>,
    /// Optional issuer of the certificate,
    /// if not provided, issuer is the same as subject.
    issuer: Option<Vec<Attribute>>,
    /// Optional validity not before date,
    /// if not provided, set to current time.
    validity_not_before: Option<String>,
    /// Optional validity not after date,
    /// if not provided, set to no expire date 9999-12-31T23:59:59+00:00.
    validity_not_after: Option<String>,
    /// Attributes of the subject.
    subject: Vec<Attribute>,
    /// Optional subject public key algorithm of the certificate,
    /// if not provided, set to Ed25519.
    subject_public_key_algorithm: Option<SubjectPubKeyAlgorithm>,
    /// A path to the public key file.
    /// Currently support only PEM format.
    subject_public_key: String,
    /// Extensions of the certificate.
    extensions: Extensions,
    /// Optional issuer signature value of the certificate.
    #[serde(skip_deserializing)]
    issuer_signature_value: Option<Vec<u8>>,
}

/// Ed25519 oid and parameter - default algorithm.
const ED25519: (Oid, Option<String>) = (oid!(1.3.101 .112), None);

/// Integer indicate that certificate is self-signed.
/// 2 for Natively Signed C509 Certificate following X.509 v3
/// 3 for CBOR re-encoding of X.509 v3 Certificate        
const SELF_SIGNED_INT: u8 = 2;

// -------------------generate-----------------------

/// A function to generate C509 certificate.
fn generate(
    file: &PathBuf, output: Option<PathBuf>, private_key: Option<&PrivateKey>,
    key_type: Option<String>,
) -> anyhow::Result<()> {
    let data = fs::read_to_string(file)?;
    let c509_json: C509Json = serde_json::from_str(&data)?;

    validate_certificate_type(c509_json.self_signed, c509_json.certificate_type)?;

    let serial_number = parse_serial_number(c509_json.serial_number);

    let issuer = determine_issuer(
        c509_json.self_signed,
        c509_json.issuer,
        c509_json.subject.clone(),
    )?;

    // Parse validity dates or use defaults
    // Now for not_before date
    let now_timestamp: u64 = Utc::now()
        .timestamp()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Current timestamp is invalid"))?;

    let not_before = parse_or_default_date(c509_json.validity_not_before, now_timestamp)?;
    // Default as expire date for not_after
    // Expire date = 9999-12-31T23:59:59+00:00 as mention in the C509 document
    let not_after = parse_or_default_date(
        c509_json.validity_not_after,
        parse_or_default_date(Some("9999-12-31T23:59:59+00:00".to_string()), 0)?,
    )?;

    let public_key = parse_public_key(&c509_json.subject_public_key)?;

    let key_type = get_key_type(key_type)?;

    // Create TbsCert instance
    let tbs = TbsCert::new(
        c509_json.certificate_type.unwrap_or(SELF_SIGNED_INT),
        serial_number,
        c509_json
            .issuer_signature_algorithm
            .unwrap_or(IssuerSignatureAlgorithm::new(key_type.0.clone(), ED25519.1)),
        Some(Name::new(NameValue::Attribute(issuer))),
        Time::new(not_before),
        Time::new(not_after),
        Name::new(NameValue::Attribute(c509_json.subject)),
        c509_json
            .subject_public_key_algorithm
            .unwrap_or(SubjectPubKeyAlgorithm::new(key_type.0, key_type.1)),
        public_key.to_bytes(),
        c509_json.extensions.clone(),
    );

    let cert = c509_certificate::generate(&tbs, private_key)?;

    // If the output path is provided, write to the file
    if let Some(output) = output {
        write_to_output_file(output, &cert)?;
    };

    println!("Hex: {:?}", hex::encode(&cert));
    println!("Bytes: {:?}", &cert);

    Ok(())
}

/// Write a data to a file given an output path.
fn write_to_output_file(output: PathBuf, data: &[u8]) -> anyhow::Result<()> {
    let mut file = File::create(output).map_err(|e| anyhow::anyhow!(e))?;
    file.write_all(data).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

/// Determine issuer of the certificate.
/// If self-signed is true, issuer is the same as subject.
/// Otherwise, issuer must be present.
fn determine_issuer(
    self_signed: bool, issuer: Option<Vec<Attribute>>, subject: Vec<Attribute>,
) -> anyhow::Result<Vec<Attribute>> {
    if self_signed {
        Ok(subject)
    } else {
        issuer.ok_or_else(|| anyhow::anyhow!("Issuer must be present if self-signed is false"))
    }
}

/// Validate the certificate type.
fn validate_certificate_type(
    self_signed: bool, certificate_type: Option<u8>,
) -> anyhow::Result<()> {
    if self_signed && certificate_type.unwrap_or(SELF_SIGNED_INT) != SELF_SIGNED_INT {
        return Err(anyhow::anyhow!(
            "Certificate type must be {SELF_SIGNED_INT} if self-signed is true"
        ));
    }
    Ok(())
}

/// Parse public key from file path.
fn parse_public_key(public_key: &str) -> anyhow::Result<PublicKey> {
    let pk_path = PathBuf::from(public_key);
    PublicKey::from_file(pk_path)
}

/// Get the key type. Currently support only Ed25519.
fn get_key_type(key_type: Option<String>) -> anyhow::Result<(Oid<'static>, Option<String>)> {
    match key_type {
        None => Err(anyhow::anyhow!("Currently only support Ed25519")),
        Some(key_type) => {
            match key_type.as_str() {
                "ed25519" => Ok(ED25519),
                _ => Err(anyhow::anyhow!("Currently only support Ed25519")),
            }
        },
    }
}

/// Parse date string to u64.
fn parse_or_default_date(date_option: Option<String>, default: u64) -> Result<u64, anyhow::Error> {
    match date_option {
        Some(date) => {
            DateTime::parse_from_rfc3339(&date)
                .map(|dt| {
                    dt.timestamp()
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Timestamp is invalid"))
                })?
                .map_err(|e| anyhow::anyhow!("Failed to parse date {date}: {e}"))
        },
        None => Ok(default),
    }
}

/// Generate random serial number if not provided
fn parse_serial_number(serial_number: Option<UnwrappedBigUint>) -> UnwrappedBigUint {
    let random_number: u64 = rand::thread_rng().gen();
    serial_number.unwrap_or(UnwrappedBigUint::new(random_number))
}

// -------------------verify-----------------------

/// Verify the signature of the certificate given public key file path.
fn verify(file: &PathBuf, public_key: PathBuf) -> anyhow::Result<()> {
    let cert = fs::read(file)?;
    let pk = PublicKey::from_file(public_key)?;
    match c509_certificate::verify(&cert, &pk) {
        Ok(()) => println!("Signature verified!"),
        Err(e) => println!("Signature verification failed: {e}"),
    };
    Ok(())
}

// -------------------decode-----------------------

/// Decode the certificate to JSON.
fn decode(file: &PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
    let cert = fs::read(file)?;
    let mut d = minicbor::Decoder::new(&cert);
    let c509 = c509_certificate::c509::C509::decode(&mut d, &mut ())?;

    let tbs_cert = c509.tbs_cert();
    let is_self_signed = tbs_cert.c509_certificate_type() == SELF_SIGNED_INT;
    let c509_json = C509Json {
        self_signed: is_self_signed,
        certificate_type: Some(tbs_cert.c509_certificate_type()),
        serial_number: Some(tbs_cert.certificate_serial_number().clone()),
        issuer_signature_algorithm: Some(tbs_cert.issuer_signature_algorithm().clone()),
        issuer: Some(extract_attributes(tbs_cert.issuer())?),
        validity_not_before: Some(time_to_string(
            tbs_cert.validity_not_before().clone().into(),
        )?),
        validity_not_after: Some(time_to_string(
            tbs_cert.validity_not_after().clone().into(),
        )?),
        subject: extract_attributes(tbs_cert.subject())?,
        subject_public_key_algorithm: Some(tbs_cert.subject_public_key_algorithm().clone()),
        // Return a hex formation of the public key
        subject_public_key: tbs_cert.subject_public_key().encode_hex(),
        extensions: tbs_cert.extensions().clone(),
        issuer_signature_value: c509.issuer_signature_value().clone(),
    };

    let data = serde_json::to_string(&c509_json)?;
    // If the output path is provided, write to the file
    if let Some(output) = output {
        write_to_output_file(output, data.as_bytes())?;
    };

    println!("{data}");
    Ok(())
}

/// Extract a `Attributes` from a `Name`.
fn extract_attributes(name: &Name) -> anyhow::Result<Vec<Attribute>> {
    match name.value() {
        NameValue::Attribute(attrs) => Ok(attrs.clone()),
        _ => Err(anyhow::anyhow!("Expected Attributes")),
    }
}

/// Convert time in i64 to string.
fn time_to_string(time: u64) -> anyhow::Result<String> {
    // Attempt to convert the timestamp and handle errors if they occur
    let timestamp: i64 = time
        .try_into()
        .map_err(|e| anyhow::anyhow!("Failed to convert time: {:?}", e))?;

    // Convert the timestamp to a DateTime and handle any potential errors
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
    Ok(datetime.to_rfc3339())
}

// -------------------main-----------------------

fn main() -> anyhow::Result<()> {
    Cli::exec()
}
