"""Jsonschema Def Definition."""

from typing import Any

from pydantic import BaseModel, ConfigDict, Field

from spec.optional import OptionalField


class JsonSchemaDefParameters(BaseModel):
    """Specification of a single Json Schema Template Definition."""

    property: str | None = Field(default=None)
    description: str
    required: OptionalField
    type: str
    items: "None | JsonSchemaDefParameters" = Field(default=None)
    choices: list[str] | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")


class JsonSchemaDefs(BaseModel):
    """Specification of an individual Json Schema Template Definition."""

    description: str
    definition: dict[str, Any]  # Raw definition from JSON
    parameters: dict[str, JsonSchemaDefParameters]
    parent: str | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")
