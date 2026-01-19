#![allow(missing_docs, unused_imports, clippy::missing_docs_in_private_items)]

uniffi::include_scaffolding!("catalyst_ffi");

mod ffi_check;

use ffi_check::ffi_check;
