#![allow(missing_docs)]

fn main() {
    uniffi::generate_scaffolding("src/catalyst_ffi.udl").unwrap();
}
