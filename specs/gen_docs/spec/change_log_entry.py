"""Change Log Entry Specification."""

import datetime

from pydantic import BaseModel, ConfigDict, Field


class ChangeLogEntry(BaseModel):
    """Change Log Entry Deserialized Specification."""

    version: str
    modified: datetime.date
    changes: str

    model_config = ConfigDict(extra="forbid")
