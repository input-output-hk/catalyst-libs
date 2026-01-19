# ruff: noqa: S101, D100, D103

from catalyst_python.catalyst_python_ffi import ffi_check


def test_ffi_integration() -> None:
    assert ffi_check(flag=True)
    assert not ffi_check(flag=False)
