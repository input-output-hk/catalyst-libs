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

use catalyst_signed_doc::CatalystSignedDocument;
use clap::Parser;

/// Hermes cli commands
#[derive(clap::Parser)]
enum Cli {
    /// Inspects COSE document
    Inspect {
        /// Path to the fully formed (should has at least one signature) COSE document
        cose_sign_path: PathBuf,
    },
    /// Inspect COSE document hex-formatted bytes
    InspectBytes {
        /// Hex-formatted COSE SIGN Bytes
        cose_sign_str: String,
    },
}

impl Cli {
    /// Execute Cli command
    fn exec(self) -> anyhow::Result<()> {
        let cose_bytes = match self {
            Self::Inspect { cose_sign_path } => {
                let mut cose_file = File::open(cose_sign_path)?;
                let mut cose_file_bytes = Vec::new();
                cose_file.read_to_end(&mut cose_file_bytes)?;
                cose_file_bytes
            },
            Self::InspectBytes { cose_sign_str } => hex::decode(&cose_sign_str)?,
        };
        println!("Bytes read:\n{}\n", hex::encode(&cose_bytes));
        let cat_signed_doc: CatalystSignedDocument = cose_bytes.as_slice().try_into()?;
        println!("{cat_signed_doc}");
        Ok(())
    }
}

fn main() {
    println!("Catalyst Signed Document");
    println!("------------------------");
    if let Err(err) = Cli::parse().exec() {
        println!("{err}");
    }
}
