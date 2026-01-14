#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use pyo3::prelude::*;

#[pymodule]
mod catalyst_python {
    use pyo3::prelude::*;
    #[pyfunction]
    fn ffi_check(flag: bool) -> bool {
        flag
    }
}
