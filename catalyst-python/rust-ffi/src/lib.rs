use pyo3::prelude::*;

#[pymodule]
mod catalyst_python {
    use pyo3::prelude::*;
    #[pyfunction]
    fn ffi_check(flag: bool) -> bool {
        flag
    }

    // #[pyclass]
    // struct CatalystSignedDocument(catalyst_signed_doc::CatalystSignedDocument);
}
