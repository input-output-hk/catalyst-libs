//! Catalyst signed document cli example

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    path::PathBuf,
};

use catalyst_signed_doc::{CatalystSignedDocument, Decode, Decoder, KidUri, Metadata};
use clap::Parser;
use coset::{iana::CoapContentFormat, CborSerializable};
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

const CONTENT_ENCODING_KEY: &str = "Content-Encoding";
const UUID_CBOR_TAG: u64 = 37;

fn encode_cbor_uuid(uuid: &uuid::Uuid) -> coset::cbor::Value {
    coset::cbor::Value::Tag(
        UUID_CBOR_TAG,
        coset::cbor::Value::Bytes(uuid.as_bytes().to_vec()).into(),
    )
}

impl Cli {
    fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Build {
                doc,
                schema,
                output,
                meta,
            } => {
                let doc_schema = load_schema_from_file(&schema)?;
                let json_doc = load_json_from_file(&doc)?;
                let json_meta = load_json_from_file(&meta)
                    .map_err(|e| anyhow::anyhow!("Failed to load metadata from file: {e}"))?;
                println!("{json_meta}");
                validate_json(&json_doc, &doc_schema)?;
                let compressed_doc = brotli_compress_json(&json_doc)?;
                let empty_cose_sign = build_empty_cose_doc(compressed_doc, &json_meta);
                store_cose_file(empty_cose_sign, &output)?;
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
            println!("This is a valid Catalyst Signed Document.");
            println!("{cat_signed_doc}");
        },
        Err(e) => eprintln!("Invalid Cataylyst Signed Document, err: {e}"),
    }
}

fn load_schema_from_file(schema_path: &PathBuf) -> anyhow::Result<jsonschema::JSONSchema> {
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

fn validate_json(doc: &serde_json::Value, schema: &jsonschema::JSONSchema) -> anyhow::Result<()> {
    schema.validate(doc).map_err(|err| {
        let mut validation_error = String::new();
        for e in err {
            validation_error.push_str(&format!("\n - {e}"));
        }
        anyhow::anyhow!("{validation_error}")
    })?;
    Ok(())
}

fn brotli_compress_json(doc: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    let brotli_params = brotli::enc::BrotliEncoderParams::default();
    let doc_bytes = serde_json::to_vec(&doc)?;
    let mut buf = Vec::new();
    brotli::BrotliCompress(&mut doc_bytes.as_slice(), &mut buf, &brotli_params)?;
    Ok(buf)
}

fn build_empty_cose_doc(doc_bytes: Vec<u8>, meta: &Metadata) -> coset::CoseSign {
    let mut builder =
        coset::HeaderBuilder::new().content_format(CoapContentFormat::from(meta.content_type()));

    if let Some(content_encoding) = meta.content_encoding() {
        builder = builder.text_value(
            CONTENT_ENCODING_KEY.to_string(),
            format!("{content_encoding}").into(),
        );
    }
    let mut protected_header = builder.build();

    protected_header.rest.push((
        coset::Label::Text("type".to_string()),
        encode_cbor_uuid(&meta.doc_type()),
    ));
    protected_header.rest.push((
        coset::Label::Text("id".to_string()),
        encode_cbor_uuid(&meta.doc_id()),
    ));
    protected_header.rest.push((
        coset::Label::Text("ver".to_string()),
        encode_cbor_uuid(&meta.doc_ver()),
    ));
    let meta_rest = meta.extra().header_rest().unwrap_or_default();

    if !meta_rest.is_empty() {
        protected_header.rest.extend(meta_rest);
    }
    coset::CoseSignBuilder::new()
        .protected(protected_header)
        .payload(doc_bytes)
        .build()
}

fn load_cose_from_file(cose_path: &PathBuf) -> anyhow::Result<coset::CoseSign> {
    let mut cose_file = File::open(cose_path)?;
    let mut cose_file_bytes = Vec::new();
    cose_file.read_to_end(&mut cose_file_bytes)?;
    let cose = coset::CoseSign::from_slice(&cose_file_bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(cose)
}

fn store_cose_file(cose: coset::CoseSign, output: &PathBuf) -> anyhow::Result<()> {
    let mut cose_file = File::create(output)?;
    let cose_bytes = cose
        .to_vec()
        .map_err(|e| anyhow::anyhow!("Failed to Store COSE SIGN: {e}"))?;
    cose_file.write_all(&cose_bytes)?;
    Ok(())
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
