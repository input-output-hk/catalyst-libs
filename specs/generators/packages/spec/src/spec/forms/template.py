"""Form Template Definition."""

import json
import typing
from functools import cached_property
from typing import Any

import jsonschema
import rich
from pydantic import BaseModel, ConfigDict, Field, RootModel, computed_field

from spec.forms.element.element import Element


class FormTemplateElements(RootModel[dict[str, Element]]):
    """Template Json Schema Definitions."""

    root: dict[str, Element]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for def_name, value in self.root.items():
            value.set_name(def_name)

    def names(self) -> list[str]:
        """Return a list of all element names."""
        return list(self.root.keys())

    def all(self) -> list[tuple[str, Element]]:
        """Return the name and value of all the elements, sorted by name."""
        elements = list(self.root.items())
        elements.sort(key=lambda element: element[0])
        return elements

    def get(self, name: str) -> Element:
        """Get the named element."""
        return self.root[name]

    @computed_field
    @cached_property
    def json_definition(self) -> dict[str, Any]:
        """Json Definition."""
        definitions: dict[str, Any] = {}

        for k, v in self.root.items():
            definitions[k] = v.json_definition

        return definitions

    @computed_field
    @cached_property
    def example(self) -> dict[str, Any]:
        """Generate an example of the definitions."""
        examples: dict[str, Any] = {}

        for v in self.root.values():
            for k_example, v_example in v.example.items():
                examples[k_example] = v_example  # noqa: PERF403

        return examples


class TemplateGenericSchema(RootModel[dict[str, Any]]):
    """Template Json Schema Definitions."""

    root: dict[str, typing.Any]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        validator = jsonschema.Draft202012Validator(
            jsonschema.Draft202012Validator.META_SCHEMA, format_checker=jsonschema.draft202012_format_checker
        )
        try:
            validator.validate(instance=self.root)  # type: ignore reportUnknownMemberType
        except Exception as e:
            msg = f"Generic Form Schema must be a valid Json Schema 2020-12. {e}"
            raise ValueError(msg) from e

    def example(self, properties: dict[str, typing.Any] | None = None) -> dict[str, typing.Any]:
        """Generate a `template_example.schema.json` file from the definitions."""
        schema = self.root
        if properties is not None:
            schema["properties"] = properties
        schema["title"] = "Example Template Schema."
        schema["description"] = "Example Template Schema showing all defined field types."

        # Just ensure the generated example is valid.
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
        except Exception:
            rich.print(json.dumps(schema, indent=4))
            raise

        return schema

    def basic(self) -> dict[str, typing.Any]:
        """Generate a `form_template_basic.schema.json` file from the definitions."""
        schema = self.root

        # Just ensure the generated example is valid.
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
        except Exception:
            rich.print(json.dumps(schema, indent=4))
            raise

        return schema


class FormTemplateAssetsIcons(RootModel[dict[str, str]]):
    """Template Assets Schema Definitions."""

    root: dict[str, str]

    @computed_field
    @property
    def all(self) -> list[str]:
        """Get all Icon names.

        Names are sorted alphabetically, but clustered
        based on the first component of the name,
        and how many component there are in the name.

        This keeps icon names with the same related purpose together
        when listed.
        """
        return sorted(self.root.keys(), key=lambda s: (s.split("-", maxsplit=1)[0], len(s.split("-")), s))

    def svg(self, name: str) -> str:
        """Return SVG icon data for the named icon."""
        return self.root[name]

    def check(self, items: list[str] | list[int]) -> bool:
        """Check if the items are a list of Icon names."""
        return set(self.all) == set(items)


class FormTemplateAssets(BaseModel):
    """Template Assets Schema Definitions."""

    icons: FormTemplateAssetsIcons

    model_config = ConfigDict(extra="forbid")


class FormTemplate(BaseModel):
    """Template Json Schema Definitions."""

    elements: FormTemplateElements
    generic_schema: TemplateGenericSchema = Field(alias="schema")
    assets: FormTemplateAssets

    model_config = ConfigDict(extra="forbid")
