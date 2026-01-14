//! Catalyst signed document cli example

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use anyhow::Context;
use catalyst_signed_doc::{
    CatalystSignedDocument,
    builder::{Builder, ed25519::Ed25519SigningKey},
    catalyst_id::CatalystId,
};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    Cli::parse().exec()
}

/// Catalyst Sign Document CLI Commands
#[derive(clap::Parser)]
#[allow(clippy::large_enum_variant)]
enum Cli {
    /// Builds a COSE document without signatures
    Build {
        /// Path to the document in the json format
        doc: PathBuf,
        /// Path to the output COSE file to store.
        output: PathBuf,
        /// Document metadata, must be in JSON format
        meta: PathBuf,
    },
    /// Adds a signature to already formed COSE document
    Sign {
        /// Path to the formed (could be empty, without any signatures) COSE document
        /// This exact file would be modified and new signature would be added
        doc: PathBuf,
        /// secret key hex bytes
        sk_hex: String,
        /// Signer kid
        kid: CatalystId,
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
            Self::Build { doc, output, meta } => {
                // Load Metadata from JSON file
                let metadata: serde_json::Value =
                    load_json_from_file(&meta).context("Failed to load metadata from file")?;
                println!("{metadata}");
                // Load Document from JSON file
                let json_doc: serde_json::Value = load_json_from_file(&doc)?;
                // Start with no signatures.
                let signed_doc = Builder::new()
                    .with_json_metadata(metadata)?
                    .with_json_content(&json_doc)?
                    .build()?;
                println!("report {:?}", &signed_doc.report());
                save_signed_doc(signed_doc, &output)?;
            },
            Self::Sign { doc, sk_hex, kid } => {
                let sk = Ed25519SigningKey::from_str(&sk_hex)?;
                let cose_bytes = read_bytes_from_file(&doc)?;
                let signed_doc = signed_doc_from_bytes(cose_bytes.as_slice())?;

                let new_signed_doc = signed_doc
                    .into_builder()?
                    .add_signature(|message| sk.sign(&message), kid.clone())?
                    .build()?;
                save_signed_doc(new_signed_doc, &doc)?;
            },
            Self::Inspect { path } => {
                let cose_bytes = read_bytes_from_file(&path)?;
                inspect_signed_doc(&cose_bytes)?;
            },
            Self::InspectBytes { cose_sign_hex } => {
                let cose_bytes = hex::decode(&cose_sign_hex)?;
                inspect_signed_doc(&cose_bytes)?;
            },
        }
        println!("Done");
        Ok(())
    }
}

fn read_bytes_from_file(path: &PathBuf) -> anyhow::Result<Vec<u8>> {
    let mut cose_file = File::open(path)?;
    let mut cose_bytes = Vec::new();
    cose_file.read_to_end(&mut cose_bytes)?;
    Ok(cose_bytes)
}

fn inspect_signed_doc(cose_bytes: &[u8]) -> anyhow::Result<()> {
    println!(
        "Decoding {} bytes:\n{}",
        cose_bytes.len(),
        hex::encode(cose_bytes)
    );
    let cat_signed_doc = signed_doc_from_bytes(cose_bytes)?;
    println!("This is a valid Catalyst Document.");
    println!("{cat_signed_doc}");
    Ok(())
}

fn save_signed_doc(
    signed_doc: CatalystSignedDocument,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let mut bytes: Vec<u8> = Vec::new();
    minicbor::encode(signed_doc, &mut bytes).context("Failed to encode document")?;

    write_bytes_to_file(&bytes, path)
}

fn signed_doc_from_bytes(cose_bytes: &[u8]) -> anyhow::Result<CatalystSignedDocument> {
    cose_bytes.try_into().context("Invalid Catalyst Document")
}

fn load_json_from_file<T>(path: &PathBuf) -> anyhow::Result<T>
where T: for<'de> serde::Deserialize<'de> {
    let file = File::open(path)?;
    let json = serde_json::from_reader(file)?;
    Ok(json)
}

fn write_bytes_to_file(
    bytes: &[u8],
    output: &PathBuf,
) -> anyhow::Result<()> {
    File::create(output)?
        .write_all(bytes)
        .context(format!("Failed to write to file {}", output.display()))
}
