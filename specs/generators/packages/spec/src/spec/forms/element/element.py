"""Form Template Definition."""

import re
from functools import cached_property
from typing import Any

from pydantic import BaseModel, ConfigDict, PrivateAttr, computed_field

from spec.forms.element.parameters import Parameters


class Element(BaseModel):
    """Specification of an individual Form Element."""

    description: str
    definition: dict[str, Any]  # Raw definition from JSON
    parameters: Parameters
    parent: list[str]
    _name: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    @computed_field
    @property
    def name(self) -> str:
        """Name Of the Element."""
        return self._name

    @computed_field
    @property
    def snake_name(self) -> str:
        """Name Of the Element in snake case."""
        return re.sub(r"(?<!^)(?=[A-Z])", "_", self._name).lower()

    @computed_field
    @property
    def title_name(self) -> str:
        """Name Of the Element in title case."""
        return re.sub(r"(?<!^)(?=[A-Z])", " ", self._name).title()

    def set_name(self, val: str) -> None:
        """Set Name."""
        self._name = val
        self.parameters.set_element_name(val)

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
        example[example_name]["$ref"] = f"#/$defs/{name}"

        return example
