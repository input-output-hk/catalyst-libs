"""Individual Document Specification."""

import typing

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr

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


def empty_string_list() -> list[str]:
    """Get an empty string list."""
    return []


class Document(BaseModel):
    """Document Data Definition."""

    type: list[str]
    description: str | None = Field(default=None)
    validation: str | None = Field(default=None)
    business_logic: DocumentBusinessLogic = Field(
        default_factory=DocumentBusinessLogic,
    )
    notes: list[str]
    headers: dict[str, CoseHeader]
    metadata: dict[str, Metadata]
    payload: Payload | None = Field(default=None)
    signers: Signers
    authors: dict[str, str]
    versions: list[ChangeLogEntry]

    _name: str | None = PrivateAttr(default=None)
    _all_refs: list[str] = PrivateAttr(default_factory=empty_string_list)
    _refed_by: list[str] = PrivateAttr(default_factory=empty_string_list)

    doc_name: str | None = Field(default=None)  # Set when wwe get a document

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set all the documents this references.
        all_refs: list[str] = []
        for meta in self.metadata.values():
            if meta.format == "Document Reference":
                all_refs.extend(meta.type)
        self._all_refs = list(set(all_refs))

    def set_name(self, doc_name: str) -> None:
        """Set the name properties."""
        self.doc_name = doc_name

        for name, meta in self.metadata.items():
            meta.set_name(name, doc_name)

    def add_referer(self, doc_name: str) -> None:
        """Set the name properties."""
        if doc_name not in self._refed_by:
            self._refed_by.append(doc_name)

    @property
    def all_references(self) -> list[str]:
        """Get a list of all documents this document references."""
        return self._all_refs

    @property
    def name(self) -> str:
        """Get name of this document."""
        return self._name if self._name is not None else "Unknown"

    @property
    def all_docs_referencing(self) -> list[str]:
        """Get name of all documents which reference this document."""
        return self._refed_by

    @property
    def content_type(self) -> str | list[str]:
        """Get document content type."""
        content_type = self.headers.get("content type")
        if content_type is not None:
            content_type = content_type.value
        if content_type is None:
            content_type = "Undefined"
        return content_type
