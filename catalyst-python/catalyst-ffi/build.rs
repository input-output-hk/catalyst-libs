#![allow(missing_docs)]

fn main() {
    uniffi::generate_scaffolding("src/ffi_check.udl").unwrap();
}