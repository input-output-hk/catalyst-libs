"""Signed Document Specification."""

# Autogenerate Documentation Pages from the formal specification

import json
import textwrap
import typing
from enum import Enum
from pathlib import Path


class HeaderType(Enum):
    """All header Types."""

    DOCUMENT = 1
    SIGNATURE = 2
    METADATA = 3


class SignedDocSpec:
    """Signed Document Specification Data."""

    DOCS_KEY: typing.ClassVar = "docs"
    LINK_AKA_KEY: typing.ClassVar = "linkAKA"
    DOCUMENTATION_LINKS_KEY: typing.ClassVar = "documentationLinks"
    HEADERS: typing.ClassVar = {
        HeaderType.DOCUMENT: {"headers": "cose_headers", "order": "cose_headers_order", "format": "coseHeaderFormats"},
        HeaderType.SIGNATURE: {
            "headers": "cose_signature_headers",
            "order": "cose_signature_headers_order",
            "format": "coseHeaderFormats",
        },
        HeaderType.METADATA: {"headers": "metadata", "order": "metadata_order", "format": "metadataFormats"},
    }

    def __init__(self, spec_file: str) -> None:
        """Initialize the Signed Document Specification."""
        with Path(spec_file).open("r") as f:
            self._data: dict = json.load(f)
        self._file = spec_file

    def data(self) -> dict:
        """Return the raw spec data."""
        return self._data

    def document_names(self) -> list[str]:
        """Get all documents."""
        docs: dict = self._data[self.DOCS_KEY]
        return docs.keys()

    def format_names(self, header_type: HeaderType) -> list[str]:
        """Get a list of all metadata format names defined."""
        _, _, formats = self.headers_and_order(header_type=header_type)
        metadata_formats: dict = self._data[formats]
        return metadata_formats.keys()

    def link_aka(self, link_name: str) -> str | None:
        """Get a Link AKA for a link name, or None if it doesn't exist."""
        link_aka: dict = self._data[self.LINK_AKA_KEY]
        return link_aka.get(link_name)

    def link_names(self) -> list[str]:
        """Get a list of ALL link names, including AKAs.

        Sorted from longest Link name to shortest.
        """
        link_aka: list[str] = list(self._data[self.LINK_AKA_KEY].keys())
        primary_links: list[str] = list(self._data[self.DOCUMENTATION_LINKS_KEY].keys())

        return sorted(link_aka + primary_links, key=lambda x: -len(x))

    def link_for_link_name(self, link_name: str) -> str:
        """Get a link for a link name."""
        return self._data[self.DOCUMENTATION_LINKS_KEY][link_name]

    def header(self, header: str, header_type: HeaderType = HeaderType.DOCUMENT) -> dict:
        """Get Cose header definition."""
        headers, _, _ = self.headers_and_order(header_type)
        return headers[header]

    def content_type_description(self, content_type: str) -> str | None:
        """Get a description for a known content type."""
        description = self._data["contentTypes"].get(content_type)
        if description is not None:
            description = description.get("description")
        return description

    def encoding_type_description(self, encoding_type: str) -> str | None:
        """Get a description for a known content type."""
        description = self._data["encodingTypes"].get(encoding_type)
        if description is not None:
            description = description.get("description")
        return description

    def headers_and_order(self, header_type: HeaderType) -> tuple[dict, list[str], str]:
        """Get headers and their ordering for a header_type."""
        headers = self.HEADERS[header_type]["headers"]
        header_order = self.HEADERS[header_type]["order"]
        formats = self.HEADERS[header_type]["format"]

        headers: dict = self._data[headers]
        header_order: list[str] = self._data.get(header_order, [])

        # Make sure unordered headers get included in the documentation.
        for header in headers:
            if header not in header_order:
                header_order.append(header)

        return (headers, header_order, formats)

    def all_headers(self, header_type: HeaderType = HeaderType.DOCUMENT) -> list[str]:
        """Return a list of sorted Cose headers."""
        _, header_order, _ = self.headers_and_order(header_type)
        return header_order

    def cddl_type_for_metadata(self, name: str | None, header_type: str) -> str:
        """Get the CDDL type for a given Metadata field."""
        headers, _, formats = self.headers_and_order(header_type)

        cddl_def = headers.get(name)
        if cddl_def is not None:
            cddl_def = cddl_def.get("format")
        if cddl_def is not None:
            cddl_def = self._data[formats].get(cddl_def)
        if cddl_def is not None:
            cddl_def = cddl_def.get("cddl")
        return cddl_def

    def cddl_def(self, name: str) -> dict | None:  # noqa: C901
        """Get a cddl definition by name."""

        def synthetic_headers(defs: dict, header_type: HeaderType = HeaderType.METADATA) -> dict:
            """Generate a synthetic cddl def for this type.

            Needs to be generated from Metadata definitions.
            """
            cddl_def = ""
            defs["requires"] = []
            exclusives = []

            headers, header_names, _ = self.headers_and_order(header_type)

            for header in header_names:
                header_data = headers[header]
                optional = "" if header_data["required"] == "yes" else "?"

                exclusive: list[str] | None = header_data.get("exclusive")
                if exclusive is not None:
                    exclusive.append(header)
                    exclusive.sort()
                    if exclusive not in exclusives:
                        exclusives.append(exclusive)
                else:
                    cddl_type = self.cddl_type_for_metadata(header, header_type)
                    field_name = header_data.get("coseLabel", header)
                    if isinstance(field_name, str):
                        field_name = f'"{field_name}"'
                    cddl_def += f"{optional}{field_name} => {cddl_type}\n"
                    if cddl_type not in defs["requires"]:
                        defs["requires"].append(cddl_type)
            for exclusive_set in exclusives:
                # Exclusive sets are never required
                exclusive_fields = []
                for entry in exclusive_set:
                    cddl_type = self.cddl_type_for_metadata(entry, header_type)
                    field_name = headers[entry].get("coseLabel", entry)
                    if isinstance(field_name, str):
                        field_name = f'"{field_name}"'
                    exclusive_fields.append(f"{field_name} => {cddl_type}")
                    if cddl_type not in defs["requires"]:
                        defs["requires"].append(cddl_type)

                cddl_def += f"""? (
    {" //\n    ".join(exclusive_fields)}
)
""".strip()
            defs["def"] = f"(\n{textwrap.indent(cddl_def, '  ')})"
            return defs

        defs = self._data.get("cddlDefinitions")
        if defs is not None:
            defs = defs.get(name)
            if name == "Signed_Document_Metadata_Headers":
                defs = synthetic_headers(defs, HeaderType.METADATA)
            elif name == "COSE_Document_Standard_Headers":
                defs = synthetic_headers(defs, HeaderType.DOCUMENT)
            elif name == "COSE_Signature_Standard_Headers":
                defs = synthetic_headers(defs, HeaderType.SIGNATURE)
        return defs

    def copyright(self, document_name: str | None) -> tuple[dict[str, str], dict, list[dict], str]:
        """Get copyright information from the spec."""

        def get_latest_file_change(versions: list, doc_versions: list | None) -> str:
            """Get the largest document version date."""
            latest_date = ""
            for ver in versions:
                latest_date = max(latest_date, ver["modified"])

            if doc_versions is not None:
                for ver in doc_versions:
                    latest_date = max(latest_date, ver["modified"])

            return latest_date

        authors = self._data["authors"]
        copyright_data = self._data["copyright"]
        versions = copyright_data["versions"]

        doc_versions = None
        if document_name is not None:
            authors = self._data[self.DOCS_KEY][document_name]["authors"] | authors
            doc_versions = self._data[self.DOCS_KEY][document_name]["versions"]

        latest_change = get_latest_file_change(versions, doc_versions)
        if doc_versions is not None:
            versions = doc_versions

        return (authors, copyright_data, versions, latest_change)

    def base_document_types(self) -> dict[str, str]:
        """Get the base document types."""
        return self._data["base_types"]

    def document_type(self, doc_name: str) -> list[str]:
        """Get the types for a specific document."""
        docs = self._data[self.DOCS_KEY]
        return docs[doc_name]["type"]

    def doc_name_for_type(self, uuid: str) -> str:
        """Get the name for a document base type, given its uuid."""
        doc_types = self.base_document_types()
        for k, v in doc_types.items():
            if v == uuid:
                return k
        return "Unknown"
