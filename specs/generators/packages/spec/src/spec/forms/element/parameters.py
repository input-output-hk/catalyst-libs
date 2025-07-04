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

    property: str | None = Field(default=None)
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

    def element_name(self) -> str:
        """Name Of the Parameters Element Type."""
        return self._element_name

    def set_element_name(self, val: str) -> None:
        """Set Element Name."""
        self._element_name = val

    def name(self) -> str:
        """Name Of the Parameter."""
        return self._name

    def set_name(self, val: str) -> None:
        """Set Name."""
        self._name = val

    def example_kv(self, index: int = 0) -> tuple[str, Any]:
        """Generate an example value of the parameter."""
        prop = self.property if self.property is not None else "Unknown"
        value: dict[Any, Any] | str = {}
        if self.type == "string":
            value = f"An Example String {index}"

        return (prop, value)


class Parameters(RootModel[dict[str, Parameter]]):
    """All Parameters defined for an Element."""

    root: dict[str, Parameter]
    _element_name: str = PrivateAttr(default="Unknown")

    def element_name(self) -> str | None:
        """Name Of the Parameters Element Type."""
        return self._element_name

    def set_element_name(self, val: str) -> None:
        """Set Element Name."""
        self._element_name = val
        for name, value in self.root.items():
            value.set_element_name(name)

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for name, value in self.root.items():
            value.set_name(name)

    @computed_field
    @cached_property
    def example(self) -> dict[str, Any]:
        """Generate an example of the definition."""
        example: dict[str, Any] = {}

        for name, value in self.root.items():
            example[name] = value.example
        return example
