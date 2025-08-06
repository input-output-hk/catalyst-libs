"""Form Template Definition."""

import typing
from functools import cached_property
from typing import Any

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, RootModel, computed_field

from spec.optional import OptionalField


class Parameter(BaseModel):
    """Specification of a single Json Schema Template Definition.

    Models `specs/definitions/form_template/parameters.cue:#parameter`
    """

    optional_property_type: str | None = Field(default=None, alias="property")
    description: str
    required: OptionalField
    type: str
    items: "None | Parameter" = Field(default=None)
    choices: list[str] | list[int] | None = Field(default=None)
    format: str | None = Field(default=None)
    content_media_type: str | None = Field(default=None, alias="contentMediaType")
    pattern: str | None = Field(default=None)
    min_length: int | None = Field(default=None)
    minimum: int | None = Field(default=None)
    maximum: int | None = Field(default=None)
    example: Any = Field(default=None)

    _name: str = PrivateAttr(default="Unknown")
    _element_name: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    @computed_field
    @property
    def element_name(self) -> str:
        """Name Of the Parameters Element Type."""
        return self._element_name

    def set_element_name(self, val: str) -> None:
        """Set Element Name."""
        self._element_name = val

    @computed_field
    @property
    def property_type(self) -> str:
        """Name Of the Property."""
        return self.optional_property_type if self.optional_property_type is not None else self.type

    @computed_field
    @property
    def name(self) -> str:
        """Name Of the Parameter."""
        return self._name

    def set_name(self, val: str) -> None:
        """Set Name."""
        self._name = val

    def example_kv(self, index: int = 0) -> tuple[str, Any]:
        """Generate an example value of the parameter."""
        value: dict[Any, Any] | str = {}
        if self.type == "string":
            value = f"An Example String {index}"

        return (self.property_type, value)


class Parameters(RootModel[dict[str, "Parameter | Parameters"]]):
    """All Parameters defined for an Element."""

    root: dict[str, "Parameter | Parameters"]
    _element_name: str = PrivateAttr(default="Unknown")

    @computed_field
    @property
    def all(self) -> list["Parameter | Parameters"]:
        """All the Parameters of an Element Type."""
        return [self.root[prop] for prop in sorted(self.root.keys())]

    @computed_field
    @property
    def element_name(self) -> str | None:
        """Name Of the Parameters Element Type."""
        return self._element_name

    def set_element_name(self, val: str) -> None:
        """Set Element Name."""
        self._element_name = val
        for name, value in self.root.items():
            value.set_element_name(name)

    def set_name(self) -> None:
        """Set Element Name."""
        for name, value in self.root.items():
            if isinstance(value, Parameter):
                value.set_name(name)
            else:
                value.set_name()

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        self.set_name()

    @computed_field
    @cached_property
    def example(self) -> dict[str, Any]:
        """Generate an example of the definition."""
        example: dict[str, Any] = {}

        for name, value in self.root.items():
            if value.example is not None:
                example[name] = value.example
        return example
