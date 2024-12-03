//! Catalyst documents signing cli example

#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    dead_code,
    unused_variables
)]

use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    path::PathBuf,
};

use clap::Parser;
use coset::CborSerializable;
use ed25519_dalek::{
    ed25519::signature::Signer,
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};

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
        /// Path to the json schema (Draft 7) to validate document agains it
        schema: PathBuf,
        /// Path to the output COSE file to store.
        output: PathBuf,
    },
    /// Adds a signature to already formed COSE document
    Sign {
        /// Path to the secret key in PEM format
        sk: PathBuf,
        /// Path to the formed (could be empty, without any signatures) COSE document
        /// This exact file would be modified and new signature would be added
        doc: PathBuf,
        /// Signer kid
        kid: String,
    },
    /// Verifies COSE document
    Verify {
        /// Path to the public key in PEM format
        pk: PathBuf,
        /// Path to the fully formed (should has at least one signature) COSE document
        doc: PathBuf,
        /// Path to the json schema (Draft 7) to validate document agains it
        schema: PathBuf,
    },
}

impl Cli {
    fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Build {
                doc,
                schema,
                output,
            } => {
                let schema = load_schema_from_file(&schema)?;
                let json_doc = load_json_doc_from_file(&doc)?;
                validate_json_doc(&json_doc, &schema)?;
                let compressed_doc = brotli_compress_doc(&json_doc)?;
                let empty_cose_sign = build_empty_cose_doc(compressed_doc);
                store_cose_into_file(empty_cose_sign, &output)?;
            },
            Self::Sign { sk, doc, kid } => {
                let sk = load_secret_key_from_file(&sk)?;
                let mut cose = load_cose_doc_from_file(&doc)?;
                add_signature_to_cose_doc(&mut cose, &sk, kid);
                store_cose_into_file(cose, &doc)?;
            },
            Self::Verify { pk, doc, schema } => {
                let pk = load_public_key_from_file(&pk)?;
                let cose = load_cose_doc_from_file(&doc)?;
            },
        }
        Ok(())
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

fn load_json_doc_from_file(doc_path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let doc_file = File::open(doc_path)?;
    let doc_json = serde_json::from_reader(doc_file)?;
    Ok(doc_json)
}

fn validate_json_doc(
    doc: &serde_json::Value, schema: &jsonschema::JSONSchema,
) -> anyhow::Result<()> {
    schema.validate(doc).map_err(|err| {
        let mut validation_error = String::new();
        for e in err {
            validation_error.push_str(&format!("\n - {e}"));
        }
        anyhow::anyhow!("{validation_error}")
    })?;
    Ok(())
}

fn brotli_compress_doc(doc: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    let brotli_params = brotli::enc::BrotliEncoderParams::default();
    let doc_bytes = serde_json::to_vec(&doc)?;
    let mut buf = Vec::new();
    brotli::BrotliCompress(&mut doc_bytes.as_slice(), &mut buf, &brotli_params)?;
    Ok(buf)
}

fn build_empty_cose_doc(doc_bytes: Vec<u8>) -> coset::CoseSign {
    let protected_header =
        coset::HeaderBuilder::new().content_format(coset::iana::CoapContentFormat::Json);

    coset::CoseSignBuilder::new()
        .protected(protected_header.build())
        .payload(doc_bytes)
        .build()
}

fn load_cose_doc_from_file(cose_path: &PathBuf) -> anyhow::Result<coset::CoseSign> {
    let mut cose_file = File::open(cose_path)?;
    let mut cose_file_bytes = Vec::new();
    cose_file.read_to_end(&mut cose_file_bytes)?;
    let cose = coset::CoseSign::from_slice(&cose_file_bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(cose)
}

fn store_cose_into_file(cose: coset::CoseSign, output: &PathBuf) -> anyhow::Result<()> {
    let mut cose_file = File::create(output)?;
    let cose_bytes = cose.to_vec().map_err(|e| anyhow::anyhow!("{e}"))?;
    cose_file.write_all(&cose_bytes)?;
    Ok(())
}

fn load_json_doc_from_cose(cose: &coset::CoseSign) -> anyhow::Result<serde_json::Value> {
    let Some(payload) = &cose.payload else {
        anyhow::bail!("COSE document missing payload field with the JSON content in it");
    };
    let doc = serde_json::from_slice(payload)?;
    Ok(doc)
}

fn load_secret_key_from_file(sk_path: &PathBuf) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let sk_str = read_to_string(sk_path)?;
    let sk = ed25519_dalek::SigningKey::from_pkcs8_pem(&sk_str)?;
    Ok(sk)
}

fn load_public_key_from_file(pk_path: &PathBuf) -> anyhow::Result<ed25519_dalek::VerifyingKey> {
    let pk_str = read_to_string(pk_path)?;
    let pk = ed25519_dalek::VerifyingKey::from_public_key_pem(&pk_str)?;
    Ok(pk)
}

fn add_signature_to_cose_doc(
    cose: &mut coset::CoseSign, sk: &ed25519_dalek::SigningKey, kid: String,
) {
    let protected_header = coset::HeaderBuilder::new().key_id(kid.into_bytes());
    let mut signature = coset::CoseSignatureBuilder::new()
        .protected(protected_header.build())
        .build();
    let data_to_sign = cose.tbs_data(&[], &signature);
    signature.signature = sk.sign(&data_to_sign).to_vec();
    cose.signatures.push(signature);
}
