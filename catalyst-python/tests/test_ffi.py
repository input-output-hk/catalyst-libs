# ruff: noqa: S101, ERA001, D100, D103

from catalyst_python import catalyst_python_ffi


def test_ffi_integration() -> None:
    assert catalyst_python_ffi.ffi_check(True)
    assert not catalyst_python_ffi.ffi_check(False)
