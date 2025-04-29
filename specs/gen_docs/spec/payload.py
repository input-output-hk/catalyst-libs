"""Payload Specification."""

from typing import Any

from pydantic import AnyUrl, BaseModel, ConfigDict, Field


class Payload(BaseModel):
    """Payload Deserialized Specification."""

    description: str
    schema: AnyUrl | dict[str, Any] | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")
