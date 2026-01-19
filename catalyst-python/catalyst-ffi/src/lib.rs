#![allow(missing_docs, clippy::missing_docs_in_private_items)]

uniffi::include_scaffolding!("ffi_check");

fn ffi_check(flag: bool) -> bool {
    flag
}