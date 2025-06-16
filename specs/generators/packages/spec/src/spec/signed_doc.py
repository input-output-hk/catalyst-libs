"""Signed Document Specification."""

# Autogenerate Documentation Pages from the formal specification

import datetime
import json
import typing
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr

from spec.authors import Authors
from spec.base_types import BaseTypes
from spec.cddl.cose import CoseDefinitions
from spec.cddl.definition import CDDLDefinitions
from spec.change_log_entry import ChangeLogEntry
from spec.content_types import ContentTypes, EncodingTypes
from spec.copyright import Copyright
from spec.doc_clusters import DocClusters
from spec.document import Documents
from spec.documentation_links import DocumentationLinks, LinkAKA
from spec.forms.template import FormTemplate
from spec.metadata import Metadata, MetadataHeader
from spec.optional import OptionalField


class SignedDoc(BaseModel):
    """Signed Doc Deserialized Specification."""

    authors: Authors
    base_types: BaseTypes
    cddl_definitions: CDDLDefinitions = Field(alias="cddlDefinitions")
    content_types: ContentTypes = Field(alias="contentTypes")
    copyright: Copyright
    cose: CoseDefinitions
    doc_clusters: DocClusters
    docs: Documents
    documentation_links: DocumentationLinks = Field(alias="documentationLinks")
    encoding_types: EncodingTypes = Field(alias="encodingTypes")
    link_aka_dont_use: LinkAKA = Field(alias="linkAKA")  # dont use directly
    metadata: Metadata
    form_template: FormTemplate = Field(alias="formTemplate")

    _data: dict[str, typing.Any] = PrivateAttr(default_factory=dict[str, typing.Any])
    _file: str = PrivateAttr(default="Uninitialized")

    model_config = ConfigDict(extra="forbid")

    @classmethod
    def load(cls, spec_file: str) -> typing.Self:
        """Initialize the Signed Document Specification."""
        with Path(spec_file).open("r") as f:
            raw_json = f.read()
            doc = cls.model_validate_json(raw_json, strict=True)
            data: dict[str, typing.Any] = json.loads(raw_json)
            doc._data = data  # noqa: SLF001
            doc._file = spec_file  # noqa: SLF001
            return doc

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Put Base Document Types inside the individual doc types for easy reference.
        self.docs.set_base_types(self.base_types)
        self.metadata.set_name(None)

        # Associate the Link AKA with documentation links.
        self.documentation_links.set_link_aka(self.link_aka_dont_use)

        # Build dynamic CDDL Definitions from the defined headers.
        self.cddl_definitions.add(
            [
                Metadata.custom_metadata_header(
                    self.cddl_definitions.get("Signed_Document_Metadata_Headers"),
                    self.metadata.headers.all,
                    self.metadata.formats,
                ),
                Metadata.custom_metadata_header(
                    self.cddl_definitions.get("COSE_Document_Standard_Headers"),
                    self.cose.headers.all,
                    self.cose.header_formats,
                ),
                Metadata.custom_metadata_header(
                    self.cddl_definitions.get("COSE_Signature_Standard_Headers"),
                    self.cose.signature_headers.all,
                    self.cose.header_formats,
                ),
            ]
        )

    def data(self) -> dict[str, typing.Any]:
        """Return the raw spec data."""
        return self._data

    def get_copyright(
        self,
        document_name: str | None,
    ) -> tuple[Authors, Copyright, list[ChangeLogEntry], datetime.date]:
        """Get copyright information from the spec."""

        def get_latest_file_change(
            versions: list[ChangeLogEntry], doc_versions: list[ChangeLogEntry] | None
        ) -> datetime.date:
            """Get the largest document version date."""
            latest_date = datetime.date.fromtimestamp(0.0)  # noqa: DTZ012
            for ver in versions:
                latest_date = max(latest_date, ver.modified)

            if doc_versions is not None:
                for ver in doc_versions:
                    latest_date = max(latest_date, ver.modified)

            return latest_date

        authors = self.authors
        copyright_data = self.copyright
        versions = copyright_data.versions

        doc_versions = None
        if document_name is not None:
            doc = self.docs.get(document_name)
            authors = doc.authors.combine(authors)
            doc_versions = doc.versions

        latest_change = get_latest_file_change(versions, doc_versions)
        if doc_versions is not None:
            versions = doc_versions

        return (authors, copyright_data, versions, latest_change)

    def get_metadata(self, metadata_name: str, doc_name: str | None = None) -> MetadataHeader:
        """Get a metadata definition by name, and optionally for a document."""
        if doc_name is None:
            raw_metadata_def = self.metadata.headers.get(metadata_name)
        else:
            raw_metadata_def = self.docs.get(doc_name).metadata.get(metadata_name)
        raw_metadata_def.set_name(metadata_name, doc_name)
        return raw_metadata_def

    def get_metadata_as_markdown(self, doc_name: str | None = None) -> str:
        """Get metadata definitions in a markdown format."""
        fields = self.metadata.headers.names
        field_display = ""
        for field in fields:
            doc_types = None
            if doc_name is not None:
                doc_types = self.docs.type(doc_name)
            metadata_def = self.get_metadata(field, doc_name)
            if doc_name is None or metadata_def.required != OptionalField.excluded:
                field_display += metadata_def.metadata_as_markdown(
                    doc_types=doc_types,
                )
        return field_display.strip()
