//! Catalyst signed document cli example

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    path::PathBuf,
};

use catalyst_signed_doc::{
    Builder, CatalystSignedDocument, Content, Decode, Decoder, KidUri, Metadata, Signatures,
};
use clap::Parser;
use coset::{CborSerializable, Header};
use ed25519_dalek::{ed25519::signature::Signer, pkcs8::DecodePrivateKey};

fn main() {
    if let Err(err) = Cli::parse().exec() {
        println!("{err}");
    }
}

/// Hermes cli commands
#[derive(clap::Parser)]
enum Cli {
    /// Builds a COSE document without signatures
    Build {
        /// Path to the document in the json format
        doc: PathBuf,
        /// Path to the json schema (Draft 7) to validate document against it
        schema: PathBuf,
        /// Path to the output COSE file to store.
        output: PathBuf,
        /// Document metadata, must be in JSON format
        meta: PathBuf,
    },
    /// Adds a signature to already formed COSE document
    Sign {
        /// Path to the secret key in PEM format
        sk: PathBuf,
        /// Path to the formed (could be empty, without any signatures) COSE document
        /// This exact file would be modified and new signature would be added
        doc: PathBuf,
        /// Signer kid
        kid: KidUri,
    },
    /// Inspects Catalyst Signed Document
    Inspect {
        /// Path to the fully formed (should has at least one signature) COSE document
        path: PathBuf,
    },
    /// Inspects Catalyst Signed Document from hex-encoded bytes
    InspectBytes {
        /// Hex-formatted COSE SIGN Bytes
        cose_sign_hex: String,
    },
}

impl Cli {
    fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Build {
                doc,
                schema: _,
                output,
                meta,
            } => {
                // Load Metadata from JSON file
                let metadata: Metadata = load_json_from_file(&meta)
                    .map_err(|e| anyhow::anyhow!("Failed to load metadata from file: {e}"))?;
                println!("{metadata}");
                // Load Document from JSON file
                let json_doc: serde_json::Value = load_json_from_file(&doc)?;
                // Possibly encode if Metadata has an encoding set.
                let payload_bytes = serde_json::to_vec(&json_doc)?;
                let payload = match metadata.content_encoding() {
                    Some(encoding) => encoding.encode(&payload_bytes)?,
                    None => payload_bytes,
                };
                let content = Content::new(
                    payload,
                    metadata.content_type(),
                    metadata.content_encoding(),
                )?;
                // Start with no signatures.
                let signatures = Signatures::try_from(&Vec::new())?;
                let signed_doc = Builder::new()
                    .content(content)
                    .metadata(metadata)
                    .signatures(signatures)
                    .build()?;
                let mut bytes: Vec<u8> = Vec::new();
                minicbor::encode(signed_doc, &mut bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to encode document: {e}"))?;

                write_bytes_to_file(&bytes, &output)?;
            },
            Self::Sign { sk, doc, kid } => {
                let sk = load_secret_key_from_file(&sk)
                    .map_err(|e| anyhow::anyhow!("Failed to load SK FILE: {e}"))?;
                let mut cose = load_cose_from_file(&doc)
                    .map_err(|e| anyhow::anyhow!("Failed to load COSE FROM FILE: {e}"))?;
                add_signature_to_cose(&mut cose, &sk, kid.to_string());
                store_cose_file(cose, &doc)?;
            },
            Self::Inspect { path } => {
                let mut cose_file = File::open(path)?;
                let mut cose_bytes = Vec::new();
                cose_file.read_to_end(&mut cose_bytes)?;
                decode_signed_doc(&cose_bytes);
            },
            Self::InspectBytes { cose_sign_hex } => {
                let cose_bytes = hex::decode(&cose_sign_hex)?;
                decode_signed_doc(&cose_bytes);
            },
        }
        println!("Done");
        Ok(())
    }
}

fn decode_signed_doc(cose_bytes: &[u8]) {
    println!(
        "Decoding {} bytes: {}",
        cose_bytes.len(),
        hex::encode(cose_bytes)
    );
    match CatalystSignedDocument::decode(&mut Decoder::new(cose_bytes), &mut ()) {
        Ok(cat_signed_doc) => {
            println!("This is a valid Catalyst Document.");
            println!("{cat_signed_doc}");
        },
        Err(e) => eprintln!("Invalid Catalyst Document, err: {e}"),
    }
}

fn _load_schema_from_file(schema_path: &PathBuf) -> anyhow::Result<jsonschema::JSONSchema> {
    let schema_file = File::open(schema_path)?;
    let schema_json = serde_json::from_reader(schema_file)?;
    let schema = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(schema)
}

fn load_json_from_file<T>(path: &PathBuf) -> anyhow::Result<T>
where T: for<'de> serde::Deserialize<'de> {
    let file = File::open(path)?;
    let json = serde_json::from_reader(file)?;
    Ok(json)
}

fn _validate_json(doc: &serde_json::Value, schema: &jsonschema::JSONSchema) -> anyhow::Result<()> {
    schema.validate(doc).map_err(|err| {
        let mut validation_error = String::new();
        for e in err {
            validation_error.push_str(&format!("\n - {e}"));
        }
        anyhow::anyhow!("{validation_error}")
    })?;
    Ok(())
}

fn _brotli_compress_json(doc: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    let brotli_params = brotli::enc::BrotliEncoderParams::default();
    let doc_bytes = serde_json::to_vec(doc)?;
    let mut buf = Vec::new();
    brotli::BrotliCompress(&mut doc_bytes.as_slice(), &mut buf, &brotli_params)?;
    Ok(buf)
}

fn _build_empty_cose_doc(doc_bytes: Vec<u8>, meta: &Metadata) -> anyhow::Result<coset::CoseSign> {
    let protected_header = Header::try_from(meta)?;
    Ok(coset::CoseSignBuilder::new()
        .protected(protected_header)
        .payload(doc_bytes)
        .build())
}

fn load_cose_from_file(cose_path: &PathBuf) -> anyhow::Result<coset::CoseSign> {
    let cose_file_bytes = read_bytes_from_file(cose_path)?;
    let cose = coset::CoseSign::from_slice(&cose_file_bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(cose)
}

fn read_bytes_from_file(path: &PathBuf) -> anyhow::Result<Vec<u8>> {
    let mut file_bytes = Vec::new();
    File::open(path)?.read_to_end(&mut file_bytes)?;
    Ok(file_bytes)
}

fn write_bytes_to_file(bytes: &[u8], output: &PathBuf) -> anyhow::Result<()> {
    File::create(output)?
        .write_all(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to write to file {output:?}: {e}"))
}

fn store_cose_file(cose: coset::CoseSign, output: &PathBuf) -> anyhow::Result<()> {
    let cose_bytes = cose
        .to_vec()
        .map_err(|e| anyhow::anyhow!("Failed to Store COSE SIGN: {e}"))?;
    write_bytes_to_file(&cose_bytes, output)
}

fn load_secret_key_from_file(sk_path: &PathBuf) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let sk_str = read_to_string(sk_path)?;
    let sk = ed25519_dalek::SigningKey::from_pkcs8_pem(&sk_str)?;
    Ok(sk)
}

fn add_signature_to_cose(cose: &mut coset::CoseSign, sk: &ed25519_dalek::SigningKey, kid: String) {
    let protected_header = coset::HeaderBuilder::new()
        .key_id(kid.into_bytes())
        .algorithm(coset::iana::Algorithm::EdDSA);
    let mut signature = coset::CoseSignatureBuilder::new()
        .protected(protected_header.build())
        .build();
    let data_to_sign = cose.tbs_data(&[], &signature);
    signature.signature = sk.sign(&data_to_sign).to_vec();
    cose.signatures.push(signature);
}
