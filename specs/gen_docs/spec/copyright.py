"""Copyright Specification."""

import datetime

from pydantic import BaseModel, ConfigDict

from spec.change_log_entry import ChangeLogEntry


class Copyright(BaseModel):
    """Copyright Deserialized Specification."""

    copyright: str
    license: str
    created: datetime.date
    versions: list[ChangeLogEntry]

    model_config = ConfigDict(extra="forbid")
