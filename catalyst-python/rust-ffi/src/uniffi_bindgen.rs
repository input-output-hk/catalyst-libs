//! UniFFI bindings generator.
//! Ideally you would then run the uniffi-bindgen binary from the uniffi crate to generate your bindings.
//! However, this is only available with Cargo nightly.

fn main() {
    uniffi::uniffi_bindgen_main()
}