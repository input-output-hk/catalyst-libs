#![allow(missing_docs, unused_imports, clippy::missing_docs_in_private_items)]

uniffi::setup_scaffolding!();

mod catalyst_signed_doc;
mod ffi_check;

type Result<T> = core::result::Result<T, crate::Error>;

type Json = String;
type CatalystId = String;
type Uuid = String;
type Ed25519SigningKey = String;

#[derive(uniffi::Error, Debug)]
#[uniffi(flat_error)]
enum Error {
    Anyhow(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Anyhow(e) => write!(f, "{e}"),
        }
    }
}
