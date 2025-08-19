"""Form Template Definition."""

import json
import typing
from copy import deepcopy
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

    def add_referenced_json_schema_defs(self, schema: dict[str, Any]) -> dict[str, Any]:
        """Defs can reference other defs, make sure they are included properly."""
        looking = True
        while looking:
            looking = False
            for this_def in list(schema["$defs"].values()):
                if "items" in this_def:
                    ref: str = this_def["items"]["$ref"]
                    ref_element = ref.removeprefix("#/$defs/")
                    if ref_element not in schema["$defs"]:
                        schema["$defs"][ref_element] = self.root[ref_element].json_definition
                        looking = True
        return schema

    @computed_field
    @cached_property
    def json_definition(self) -> dict[str, Any]:
        """Json Definition."""
        definitions: dict[str, Any] = {}

        for k, v in self.root.items():
            definitions[k] = v.json_definition
        return self.add_referenced_json_schema_defs(definitions)

    def example(self, element: str | None = None) -> dict[str, Any]:  # noqa: C901
        """Generate an json schema example of the definitions."""
        examples: dict[str, Any] = {}

        # Add the basic values to the json schema example.
        examples["$schema"] = "https://json-schema.org/draft/2020-12/schema"
        examples["title"] = "Example" if element is None else f"{element} Example"
        examples["description"] = (
            "An example of all Elements"
            if element is None
            else f"An example of the {element} Element, and it's parents."
        )
        examples["$defs"] = {}
        examples["type"] = "object"
        properties: dict[str, Any] = {}
        examples["properties"] = properties
        examples["additionalProperties"] = False

        # Get a list of all elements we will need.
        all_elements: list[str] = []
        if element is None:
            all_elements = list(self.root.keys())
        else:
            all_elements = [element]
            found = True
            while found:
                found = False
                parents: list[str] = []
                for this_element in all_elements:
                    parents.extend(self.root[this_element].parent)
                for parent in parents:
                    if parent not in all_elements and parent != "{}":
                        all_elements.append(parent)
                        found = True

        # Generate the $defs
        for this_element in all_elements:
            examples["$defs"][this_element] = self.root[this_element].json_definition
        examples = self.add_referenced_json_schema_defs(examples)

        def add_element(properties: dict[str, Any], parent: str, *, stop: bool = False) -> None:
            """Recursively add elements to their parents."""
            for this_element in all_elements:
                if parent in self.root[this_element].parent:
                    example = deepcopy(self.root[this_element].example)
                    if stop and this_element == parent:
                        continue
                    properties.update(example)
                    for property_name in example:
                        if "properties" in example[property_name]:
                            add_element(
                                properties[property_name]["properties"], this_element, stop=this_element == parent
                            )

        add_element(examples["properties"], "{}")

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
            schema_txt = json.dumps(self.root, indent=2)
            msg = f"Generic Form Schema must be a valid Json Schema 2020-12. {e}\n{schema_txt}"
            raise ValueError(msg) from e

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
