"""Form Template Definition."""

import json
import typing
from functools import cached_property
from typing import Any

import jsonschema
from pydantic import BaseModel, Field, RootModel, computed_field

from spec.forms.element.element import Element


class FormTemplateElements(RootModel[dict[str, Element]]):
    """Template Json Schema Definitions."""

    root: dict[str, Element]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for def_name, value in self.root.items():
            value.set_name(def_name)

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
            print(json.dumps(schema, indent=4))
            raise

        return schema

    def basic(self) -> dict[str, typing.Any]:
        """Generate a `form_template_basic.schema.json` file from the definitions."""
        schema = self.root

        # Just ensure the generated example is valid.
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
        except Exception:
            print(json.dumps(schema, indent=4))
            raise

        return schema


class FormTemplate(BaseModel):
    """Template Json Schema Definitions."""

    elements: FormTemplateElements
    generic_schema: TemplateGenericSchema = Field(alias="schema")
