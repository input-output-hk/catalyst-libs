//! Inspect a Catalyst Signed Document.
use std::{
    fs::{
        // read_to_string,
        File,
    },
    io::{
        Read,
        // Write
    },
    path::PathBuf,
};

use clap::Parser;
use signed_doc::CatalystSignedDocument;

/// Hermes cli commands
#[derive(clap::Parser)]
enum Cli {
    /// Inspects COSE document
    Inspect {
        /// Path to the fully formed (should has at least one signature) COSE document
        cose_sign: PathBuf,
        /// Path to the json schema (Draft 7) to validate document against it
        doc_schema: PathBuf,
    },
}

impl Cli {
    /// Execute Cli command
    fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Inspect {
                cose_sign,
                doc_schema: _,
            } => {
                //
                let mut cose_file = File::open(cose_sign)?;
                let mut cose_file_bytes = Vec::new();
                cose_file.read_to_end(&mut cose_file_bytes)?;
                let cat_signed_doc: CatalystSignedDocument = cose_file_bytes.try_into()?;
                println!("{cat_signed_doc}");
                Ok(())
            },
        }
    }
}

fn main() {
    println!("Catalyst Signed Document");
    if let Err(err) = Cli::parse().exec() {
        println!("{err}");
    }
}
