"""CDDL Definition Specification."""

from pydantic import BaseModel, ConfigDict, Field


class CDDLDefinition(BaseModel):
    """CDDL Definition Deserialized Specification."""

    definition: str = Field(alias="def")
    requires: list[str]
    description: str | None = Field(default=None)
    comment: str | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")
