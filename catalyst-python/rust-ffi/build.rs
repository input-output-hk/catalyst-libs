#![allow(missing_docs)]

fn main() {
    uniffi::generate_scaffolding("src/catalyst_python_ffi.udl").unwrap();
}