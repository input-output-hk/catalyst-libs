"""Payload Specification."""

import json
import urllib
import urllib.request
from typing import Any

import jsonschema
import rich
from pydantic import BaseModel, ConfigDict, Field, HttpUrl

from spec.example import JsonExample

DRAFT7_SCHEMA = "https://json-schema.org/draft-07/schema"
DRAFT202012_SCHEMA = "https://json-schema.org/draft/2020-12/schema"


class SchemaValidationError(Exception):
    """Something is wrong with payload schema validation."""


class Payload(BaseModel):
    """Payload Deserialized Specification."""

    description: str
    nil: bool
    doc_schema: HttpUrl | dict[str, Any] | None = Field(default=None, alias="schema")
    examples: list[JsonExample] = Field(default_factory=JsonExample.default)

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: Any) -> None:  # noqa: ANN401
        """Validate the examples against the schema."""
        schema = None
        validator = None
        if isinstance(self.doc_schema, HttpUrl):
            if f"{self.doc_schema}" == DRAFT7_SCHEMA:
                schema = jsonschema.Draft7Validator.META_SCHEMA
            elif f"{self.doc_schema}" == DRAFT202012_SCHEMA:
                schema = jsonschema.Draft202012Validator.META_SCHEMA
            else:
                rich.print(f"Downloading Schema from: {self.doc_schema}")
                with urllib.request.urlopen(f"{self.doc_schema}") as response:  # noqa: S310
                    schema = json.loads(response.read())
        elif isinstance(self.doc_schema, dict):
            schema = self.doc_schema

        if schema is not None:
            # Check that its valid jsonschema Draft 7 or 202012.
            try:
                jsonschema.Draft7Validator.check_schema(schema)
                validator = jsonschema.Draft7Validator(schema, format_checker=jsonschema.draft7_format_checker)
            except:  # noqa: E722
                jsonschema.Draft202012Validator.check_schema(schema)
                validator = jsonschema.Draft202012Validator(
                    schema, format_checker=jsonschema.draft202012_format_checker
                )

        for example in self.examples:
            if validator is None:
                msg = "No schema to validate payload examples."
                raise SchemaValidationError(msg)
            validator.validate(instance=example.example)  # type: ignore  # noqa: PGH003

        return super().model_post_init(context)
