"""Base Document Type Identifiers."""

import typing

from pydantic import RootModel, computed_field
from pydantic.types import UUID4


class DocTypeId(RootModel[UUID4]):
    """Document Type ID is a UUIDv4."""

    root: UUID4  # name: uuid_str

    class Config:
        frozen = True

    @computed_field
    @property
    def as_cbor(self) -> str:
        """DocType in CBOR Diagnostic Notation."""
        return f"37(h'{self.root.hex}')"

    @computed_field
    @property
    def as_uuid_str(self) -> str:
        """DocType in CBOR Diagnostic Notation."""
        return str(self.root)


class BaseTypes(RootModel[dict[str, DocTypeId]]):
    """Authors of the Specifications."""

    root: dict[str, DocTypeId]  # name: uuid_str

    _for_uuid: dict[DocTypeId, str]  # reverse lookup

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set all the documents this references.
        self._for_uuid = {}
        for k, v in self.root.items():
            self._for_uuid[v] = k

    def uuid(self, name: str) -> DocTypeId:
        """Get UUID for Base Type Name."""
        return self.root[name]

    def name(self, uuid: DocTypeId) -> str:
        """Get Base Type Name for the given UUID."""
        return self._for_uuid[uuid]

    @computed_field
    @property
    def all(self) -> list[str]:
        """Get All Base Types."""
        return sorted(self.root.keys())
