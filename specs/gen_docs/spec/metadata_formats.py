"""Metadata Formats Specification."""

from pydantic import BaseModel, ConfigDict


class MetadataFormats(BaseModel):
    """Metadata Formats Deserialized Specification."""

    description: str
    cddl: str

    model_config = ConfigDict(extra="forbid")
