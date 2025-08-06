"""Presentation Template Definition."""

import json
import typing
from typing import Any

import jsonschema
import rich
from pydantic import BaseModel, ConfigDict, Field, RootModel

from spec.presentation_templates.card import Card


class PresentationTemplateCards(RootModel[dict[str, Card]]):
    """Template Json Schema Definitions."""

    root: dict[str, Card]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for def_name, value in self.root.items():
            value.set_card_id(def_name)

    def names(self) -> list[str]:
        """Return a list of all element names."""
        return list(self.root.keys())

    def all(self) -> list[tuple[str, Card]]:
        """Return the card_id and value of all the presentation cards, sorted by card_id."""
        elements = list(self.root.items())
        elements.sort(key=lambda element: element[0])
        return elements

    def get(self, name: str) -> Card:
        """Get the named presentation card."""
        return self.root[name]


class PresentationTemplateSchema(RootModel[dict[str, Any]]):
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
            msg = f"Presentation Template Schema must be a valid Json Schema 2020-12. {e}"
            raise ValueError(msg) from e

    def template_schema(self) -> dict[str, typing.Any]:
        """Generate a `presentation_template.schema.json` file from the definitions."""
        schema = self.root

        # Just ensure the generated example is valid.
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
        except Exception:
            rich.print(json.dumps(schema, indent=4))
            raise

        return schema


class PresentationTemplate(BaseModel):
    """Template Json Schema Definitions."""

    cards: PresentationTemplateCards
    template_schema: PresentationTemplateSchema = Field(alias="schema")

    model_config = ConfigDict(extra="forbid")
