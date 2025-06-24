"""Payload Specification."""

import json
import textwrap
import urllib
import urllib.request
from typing import Any

import jsonschema
import rich
from pydantic import BaseModel, ConfigDict, Field, HttpUrl


class PayloadExample(BaseModel):
    """An Example of the payload."""

    title: str
    description: str
    example: dict[str, Any]

    model_config = ConfigDict(extra="forbid")

    @classmethod
    def default(cls) -> list["PayloadExample"]:
        """Return Default list."""
        return []

    def __str__(self) -> str:
        """Get the example properly formatted as markdown."""
        example = json.dumps(self.example, indent=2, sort_keys=True)
        textwrap.indent(example, "    ")

        return f"""

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: {self.title}"

{textwrap.indent(self.description, "    ")}

    ```json
{textwrap.indent(example, "    ")}
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
""".strip()


class SchemaValidationError(Exception):
    """Something is wrong with payload schema validation."""


class Payload(BaseModel):
    """Payload Deserialized Specification."""

    description: str
    nil: bool = Field(default=False)
    doc_schema: HttpUrl | dict[str, Any] | None = Field(default=None, alias="schema")
    examples: list[PayloadExample] = Field(default_factory=PayloadExample.default)

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: Any) -> None:  # noqa: ANN401
        """Validate the examples against the schema."""
        schema = None
        validator = None
        if isinstance(self.doc_schema, HttpUrl):
            if f"{self.doc_schema}" == "https://json-schema.org/draft-07/schema":
                schema = jsonschema.Draft7Validator.META_SCHEMA
            elif f"{self.doc_schema}" == "https://json-schema.org/draft/2020-12/schema":
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

    def __str__(self) -> str:
        """Get the examples properly formatted as markdown."""
        docs = self.description + "\n"

        if self.nil:
            docs += """
This document has no payload.
It must be encoded as a CBOR `null (0xf6)`.
"""
            return docs

        schema = self.doc_schema
        if schema is not None:
            if isinstance(schema, HttpUrl):
                if schema == "https://json-schema.org/draft-07/schema":
                    docs += "\n**Must be a valid JSON Schema Draft 7 document.**"
                else:
                    docs += f"\nMust be a valid according to <{schema}>."
            else:
                docs += f"""\n### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract

{textwrap.indent(self.description, "    ")}

    ```json
{textwrap.indent(json.dumps(schema, indent=2, sort_keys=True), "    ")}
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
"""

        if len(self.examples) > 0:
            docs += "\n### Example"
            if len(self.examples) >= 2:  # noqa: PLR2004
                docs += "s"
            docs += "\n"
            for example in self.examples:
                docs += f"{example}\n"

        return docs.strip()
