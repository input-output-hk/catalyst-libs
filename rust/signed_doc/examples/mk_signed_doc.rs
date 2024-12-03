//! Catalyst documents signing cli example

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{fs::File, path::PathBuf};

use clap::Parser;

fn main() {
    if let Err(err) = Cli::parse().exec() {
        println!("{err}");
    }
}

/// `mk_signed_docs` cli commands
#[derive(clap::Parser)]
struct Cli {
    /// Path to the document in the json format
    doc: PathBuf,
    /// Path to the json schema (Draft 7) to validate document agains it
    schema: PathBuf,
}

impl Cli {
    fn exec(self) -> anyhow::Result<()> {
        let schema = load_schema(&self.schema)?;
        let doc = load_doc(&self.doc)?;
        validate_doc(&doc, &schema)?;

        Ok(())
    }
}

fn load_schema(schema_path: &PathBuf) -> anyhow::Result<jsonschema::JSONSchema> {
    let schema_file = File::open(schema_path)?;
    let schema_json = serde_json::from_reader(schema_file)?;
    let schema = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(schema)
}

fn load_doc(doc_path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let doc_file = File::open(doc_path)?;
    let doc_json = serde_json::from_reader(doc_file)?;
    Ok(doc_json)
}

fn validate_doc(doc: &serde_json::Value, schema: &jsonschema::JSONSchema) -> anyhow::Result<()> {
    schema.validate(doc).map_err(|err| {
        let mut validation_error = String::new();
        for e in err {
            validation_error.push_str(&format!("\n - {e}"));
        }
        anyhow::anyhow!("{validation_error}")
    })?;
    Ok(())
}
