"""Cose CDDL Definitions."""

import typing
from enum import Enum
from functools import cached_property

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, RootModel, computed_field

from spec.metadata_formats import MetadataFormats
from spec.optional import OptionalField


class HeaderType(Enum):
    """All header Types."""

    DOCUMENT = 1
    SIGNATURE = 2
    METADATA = 3


class GenericHeader(BaseModel):
    """Generic Cose/Metadata header."""

    cose_label: str | int | None = Field(default=None, alias="coseLabel")
    description: str
    required: OptionalField
    format: str

    _name: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    @computed_field
    @property
    def name(self) -> str:
        """Get headers name."""
        return self._name

    @computed_field
    @name.setter
    def name(self, name: str) -> None:
        """Set headers name."""
        self._name = name

    @computed_field
    @property
    def label(self) -> str:
        """Get headers name."""
        if self.cose_label is None:
            return f'"{self._name}"'
        if isinstance(self.cose_label, str):
            return f'"{self.cose_label}"'
        return f"{self.cose_label}"


class CoseHeader(GenericHeader):
    """Cose Header Deserialized Specification."""

    value: str | list[str] | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")


class CoseHeaders(RootModel[dict[str, CoseHeader]]):
    """Cose Headers."""

    root: dict[str, CoseHeader]

    _order: list[str] | None = PrivateAttr(default=None)

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set Cose Header Names
        for name, header in self.root.items():
            header.name = name

    def get(self, name: str) -> CoseHeader:
        """Get a Cose Header by its name."""
        return self.root[name]

    @computed_field
    @cached_property
    def all(self) -> list[CoseHeader]:
        """Get all Cose Headers sorted and in a list."""
        return [self.get(name) for name in self.names]

    @computed_field
    @property
    def names(self) -> list[str]:
        """Get ordered list of all defined Cose Header Names."""
        if self._order is not None:
            return self._order
        return list(self.root.keys())

    def order(self, order: list[str]) -> None:
        """Set the order of fields."""
        tmp_order = order
        # Add in any unlisted headers
        for name in self.root:
            if name not in tmp_order:
                tmp_order.append(name)
        self._order = []
        # remove any listed headers that are not used.
        for name in tmp_order:
            if name in self.root:
                self._order.append(name)


class CoseDefinitions(BaseModel):
    """Definitions of our COSE Format usage."""

    header_formats: MetadataFormats = Field(alias="headerFormats")
    headers: CoseHeaders
    headers_order: list[str] = Field(alias="headersOrder")  # dont use directly
    signature_headers: CoseHeaders

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set Cose Header Order
        self.headers.order(self.headers_order)
        self.signature_headers.order(self.headers_order)
