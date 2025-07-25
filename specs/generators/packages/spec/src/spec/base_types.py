"""Base Document Type Identifiers."""

from pydantic import RootModel, computed_field
from pydantic.types import UUID4


class DocTypeId(RootModel[UUID4]):
    """Document Type ID is a UUIDv4."""

    root: UUID4  # name: uuid_str

    class Config:
        """Config."""

        frozen = True

    @computed_field
    @property
    def as_cbor(self) -> str:
        """DocType in CBOR Diagnostic Notation."""
        return f"#6.37(h'{self.root.hex}')"

    @computed_field
    @property
    def as_uuid_str(self) -> str:
        """DocType in CBOR Diagnostic Notation."""
        return str(self.root)
