"""Payload Specification."""

from typing import Any

from pydantic import BaseModel, ConfigDict, Field, HttpUrl


class Payload(BaseModel):
    """Payload Deserialized Specification."""

    description: str
    schema: HttpUrl | dict[str, Any] | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")
