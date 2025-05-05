"""Payload Specification."""

from typing import Any

from pydantic import BaseModel, ConfigDict, Field, HttpUrl


class Payload(BaseModel):
    """Payload Deserialized Specification."""

    description: str
    doc_schema: HttpUrl | dict[str, Any] | None = Field(default=None, alias="schema")

    model_config = ConfigDict(extra="forbid")
