"""Metadata Formats Specification."""

import datetime

from pydantic import BaseModel, ConfigDict, Field


class MetadataFormats(BaseModel):
    """Metadata Formats Deserialized Specification."""

    description: str
    cddl: str

    model_config = ConfigDict(extra="forbid")
