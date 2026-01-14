#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use pyo3::prelude::*;

#[pymodule]
mod catalyst_python_ffi {
    use pyo3::prelude::*;
    #[pyfunction]
    fn ffi_check(flag: bool) -> bool {
        flag
    }

    
    // #[pymodule]
    // mod catalyst_signed_doc {
    //     use pyo3::prelude::*;
    //     #[pyclass]
    //     #[allow(dead_code)]
    //     struct CatalystSignedDocument(catalyst_signed_doc::CatalystSignedDocument);
    // }
}
