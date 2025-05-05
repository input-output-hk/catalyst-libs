"""Cose Header Specification."""

from pydantic import BaseModel, ConfigDict, Field

from spec.optional import OptionalField


class CoseHeader(BaseModel):
    """Cose Header Deserialized Specification."""

    cose_label: str | int = Field(alias="coseLabel")
    description: str
    required: OptionalField
    format: str
    value: str | list[str] | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")
