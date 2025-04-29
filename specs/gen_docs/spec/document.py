"""Individual Document Specification."""

from dataclasses import dataclass
from typing import Any

from pydantic import AnyUrl, BaseModel, ConfigDict, Field

from spec.change_log_entry import ChangeLogEntry
from spec.cose_header import CoseHeader
from spec.metadata import Metadata
from spec.payload import Payload
from spec.signers import Signers


class DocumentBusinessLogic(BaseModel):
    """Document Business Logic Data Definition."""

    front_end: str | None = Field(default=None)
    back_end: str | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")


class Document(BaseModel):
    """Document Data Definition."""

    type: list[str]
    description: str | None = Field(default=None)
    validation: str | None = Field(default=None)
    business_logic: DocumentBusinessLogic | None = Field(default=None)
    headers: dict[str, CoseHeader]
    metadata: dict[str, Metadata]
    payload: Payload | None = Field(default=None)
    signers: Signers
    authors: dict[str, str]
    versions: list[ChangeLogEntry]

    model_config = ConfigDict(extra="forbid")
