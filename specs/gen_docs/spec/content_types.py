"""Content Types Specification."""

from pydantic import BaseModel, ConfigDict, Field


class ContentTypes(BaseModel):
    """Content Types Deserialized Specification."""

    description: str
    coap_type: int | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")


class EncodingTypes(BaseModel):
    """Encoding Types Deserialized Specification."""

    description: str

    model_config = ConfigDict(extra="forbid")
