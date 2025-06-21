"""Form Template Definition."""

from functools import cached_property
from typing import Any

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, computed_field

from spec.forms.element.parameters import Parameters


class Element(BaseModel):
    """Specification of an individual Form Element."""

    description: str
    definition: dict[str, Any]  # Raw definition from JSON
    parameters: Parameters
    parent: str | None = Field(default=None)
    _name: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    def name(self) -> str:
        """Name Of the Parameter."""
        return self._name

    def set_name(self, val: str) -> None:
        """Set Name."""
        self._name = val

    @computed_field
    @cached_property
    def json_definition(self) -> dict[str, Any]:
        """Json Definition."""
        return self.definition

    @computed_field
    @cached_property
    def example(self) -> dict[str, Any]:
        """Generate an example of the definition."""
        name = self._name
        example_name = "example" + name[0].upper() + name[1:]
        example: dict[str, Any] = {example_name: self.parameters.example}
        example["$ref"] = f"#/$defs/{name}"

        return example
