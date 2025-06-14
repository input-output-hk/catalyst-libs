"""Form Template Definition."""

import typing
from functools import cached_property
from typing import Any

from pydantic import RootModel, computed_field

from spec.forms.element.element import Element


class FormTemplate(RootModel[dict[str, Element]]):
    """Template Json Schema Definitions."""

    root: dict[str, Element]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for def_name, value in self.root.items():
            value.name = def_name

    @computed_field()
    @cached_property
    def json_definition(self) -> dict[str, Any]:
        """Json Definition."""
        definitions: dict[str, Any] = {}

        for k, v in self.root.items():
            definitions[k] = v.json_definition

        return definitions

    @computed_field()
    @cached_property
    def example(self) -> dict[str, Any]:
        """Generate an example of the definitions."""
        examples: dict[str, Any] = {}

        for k, v in self.root.items():
            examples[k] = v.example

        return examples
