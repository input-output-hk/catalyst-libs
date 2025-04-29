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
    business_logic: DocumentBusinessLogic = Field(
        default_factory=DocumentBusinessLogic,
    )
    headers: dict[str, CoseHeader]
    metadata: dict[str, Metadata]
    payload: Payload | None = Field(default=None)
    signers: Signers
    authors: dict[str, str]
    versions: list[ChangeLogEntry]

    doc_name: str | None = Field(default=None)  # Set when wwe get a document

    model_config = ConfigDict(extra="forbid")

    def set_name(self, doc_name: str | None = None) -> None:
        """Set the name properties."""
        self.doc_name = doc_name

        for name, meta in self.metadata.items():
            meta.set_name(name, doc_name)
