"""Individual Document Specification."""

import typing
from functools import cached_property

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, RootModel, computed_field

from spec.authors import Authors
from spec.base_types import DocTypeId
from spec.cddl.cose import CoseHeaders
from spec.change_log_entry import ChangeLogEntry
from spec.metadata import MetadataHeaders
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

    type: DocTypeId
    draft: bool
    description: str | None = Field(default=None)
    validation: str | None = Field(default=None)
    business_logic: DocumentBusinessLogic = Field(
        default_factory=DocumentBusinessLogic,
    )
    notes: list[str]
    headers: CoseHeaders
    metadata: MetadataHeaders
    payload: Payload
    signers: Signers
    authors: Authors
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
        for name in self.metadata.names:
            meta = self.metadata.get(name)
            if meta.format == "Document Reference":
                all_refs.extend(meta.type)
        self._all_refs = list(set(all_refs))

    def set_name(self, doc_name: str) -> None:
        """Set the name properties."""
        self.doc_name = doc_name
        self.metadata.set_name(doc_name)

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
        if self.headers is None:
            return "Undefined"
        content_type = self.headers.get("content type")
        if content_type.value is None:
            return "Undefined"
        return content_type.value


class Documents(RootModel[dict[str, Document]]):
    """All Documents."""

    root: dict[str, Document]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set the name and references for each document.
        for name, doc in self.root.items():
            doc.set_name(name)
            for ref_doc in doc.all_references:
                self.root[ref_doc].add_referer(name)

    def get(self, name: str) -> Document:
        """Get a document by its name."""
        return self.root[name]

    @computed_field
    @cached_property
    def names(self) -> list[str]:
        """Get all documents."""
        names = list(self.root.keys())
        names.sort()
        return names

    def type(self, doc_name: str) -> DocTypeId:
        """Get the types for a specific document."""
        return self.root[doc_name].type
