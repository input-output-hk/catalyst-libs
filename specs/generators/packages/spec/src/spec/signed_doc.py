"""Signed Document Specification."""

# Autogenerate Documentation Pages from the formal specification

import datetime
import typing
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, ValidationError
from rich.console import Console
from rich.table import Table

from spec.authors import Authors
from spec.cddl.cose import CoseDefinitions
from spec.cddl.definition import CDDLDefinitions
from spec.change_log_entry import ChangeLogEntry
from spec.content_types import ContentTypes, EncodingTypes
from spec.copyright import Copyright
from spec.doc_clusters import DocClusters
from spec.document import Documents
from spec.documentation_links import Documentation
from spec.forms.template import FormTemplate
from spec.metadata import Metadata, MetadataHeader
from spec.optional import OptionalField
from spec.presentation_templates.template import PresentationTemplate


class SignedDoc(BaseModel):
    """Signed Doc Deserialized Specification."""

    authors: Authors
    cddl_definitions: CDDLDefinitions = Field(alias="cddlDefinitions")
    content_types: ContentTypes = Field(alias="contentTypes")
    copyright: Copyright
    cose: CoseDefinitions
    doc_clusters: DocClusters
    docs: Documents
    documentation: Documentation
    encoding_types: EncodingTypes = Field(alias="encodingTypes")
    metadata: Metadata
    form_template: FormTemplate = Field(alias="formTemplate")
    presentation_template: PresentationTemplate = Field(alias="presentationTemplate")

    _file: str = PrivateAttr(default="Uninitialized")

    model_config = ConfigDict(extra="forbid")

    @staticmethod
    def validation_error(err: ValidationError) -> None:
        """Print validation errors nicely when they occur (helper)."""
        table = Table(
            title=f"{err.error_count()} Locations where Schema Data does not match the {err.title} Model.",
            caption="Model does not match Schema and needs updating.",
            min_width=120,
            expand=True,
        )
        table.add_column("Key", style="yellow", overflow="fold")
        table.add_column("Error", style="red")
        table.add_column("Input", no_wrap=True, max_width=30, style="grey37")
        table.add_column("Context", style="green")

        error_links: dict[str, str] = {}
        errors = err.errors()
        errors.sort(key=lambda x: [x["loc"], x["type"]])
        for error in errors:
            error_links[error["msg"]] = error["url"]  # type: ignore  # noqa: PGH003

            loc: list[str] = []
            for x in error["loc"]:
                if isinstance(x, int):
                    loc.append(f"[{x}]")
                else:
                    loc.append(f"{x}")

            table.add_row(
                ".".join(loc),
                f"{error['type']}: {error['msg']}",
                str(error["input"]).splitlines()[0],
                ", ".join(f"{k}={v}" for k, v in (error.get("ctx") or {}).items()),
            )

        console = Console(width=120, force_terminal=True)
        console.print(table)

        for msg, url in error_links.items():
            console.print(f"* {msg} : {url}")

    @classmethod
    def load(cls, spec_file: str) -> typing.Self:
        """Initialize the Signed Document Specification."""
        with Path(spec_file).open("r") as f:
            raw_json = f.read()
            doc = cls.model_validate_json(raw_json, strict=True)
            doc._file = spec_file  # noqa: SLF001
            return doc

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        self.metadata.set_name(None)

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
            doc_type = None
            if doc_name is not None:
                doc_type = self.docs.type(doc_name)
            metadata_def = self.get_metadata(field, doc_name)
            if doc_name is None or metadata_def.required != OptionalField.excluded:
                field_display += metadata_def.metadata_as_markdown(
                    doc_type=doc_type,
                )
        return field_display.strip()
