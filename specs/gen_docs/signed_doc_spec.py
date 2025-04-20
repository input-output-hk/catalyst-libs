#!/usr/bin/env python

# Autogenerate Documentation Pages from the formal specification

import json
import textwrap


class SignedDocSpec:
    """Signed Document Specification Data"""

    DOCS_KEY = "docs"
    METADATA_KEY = "metadata"
    METADATA_FORMATS_KEY = "metadataFormats"
    LINK_AKA_KEY = "linkAKA"
    DOCUMENTATION_LINKS_KEY = "documentationLinks"

    def __init__(self, spec_file: str):
        with open(spec_file) as f:
            self._data: dict = json.load(f)
        self._file = spec_file

    def data(self) -> dict:
        """Return the raw spec data"""
        return self._data

    def document_names(self) -> dict:
        """Get all documents."""
        docs: dict = self._data[self.DOCS_KEY]
        return docs.keys()

    def metadata_names(self) -> list[str]:
        """Get a list of all metadata names defined"""
        metadata_headers = self._data["metadata_order"]
        metadata: dict = self._data[self.METADATA_KEY]
        for header in metadata:
            if header not in metadata_headers:
                metadata_headers.append(header)
        return metadata_headers

    def standard_document_header_names(self) -> list[str | int]:
        """Get a list of all standard document header names defined"""
        headers = self._data["cose_headers_order"]
        header_data: dict = self._data["cose_headers"]
        for header in header_data:
            if header not in headers:
                headers.append(header)
        return headers

    def standard_signature_header_names(self) -> list[str | int]:
        """Get a list of all standard signature header names defined"""
        headers = []
        header_data: dict = self._data["cose_signature_headers"]
        for header in header_data:
            if header not in headers:
                headers.append(header)
        return headers

    def metadata_format_names(self) -> list[str]:
        """Get a list of all metadata format names defined"""
        metadata_formats: dict = self._data[self.METADATA_FORMATS_KEY]
        return metadata_formats.keys()

    def link_aka(self, link_name: str) -> str | None:
        """Get a Link AKA for a link name, or None if it doesn't exist."""
        linkAka: dict = self._data[self.LINK_AKA_KEY]
        return linkAka.get(link_name)

    def link_names(self) -> list[str]:
        """Get a list of ALL link names, including AKAs.
        Sorted from longest Link name to shortest.
        """
        linkAka: list[str] = list(self._data[self.LINK_AKA_KEY].keys())
        primaryLinks: list[str] = list(self._data[self.DOCUMENTATION_LINKS_KEY].keys())

        return sorted(linkAka + primaryLinks, key=lambda x: -len(x))

    def link_for_linkname(self, linkName: str):
        """Get a link for a linkname"""
        return self._data[self.DOCUMENTATION_LINKS_KEY][linkName]

    def cose_header(self, header: str) -> dict:
        """Get Cose header definition"""
        return self._data["cose_headers"][header]

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

    def all_cose_headers(self) -> list[str]:
        """Return a list of sorted Cose headers."""
        headers: dict = self._data["cose_headers"]
        header_order: list[str] = self._data["cose_headers_order"]
        # Make sure unordered headers get included in the documentation.
        for header in headers:
            if header not in header_order:
                header_order += header
        return header_order

    def cddl_type_for_metadata(self, name: str | None, header_type: str) -> str:
        """Get the CDDL type for a given Metadata field."""
        if header_type == "document":
            headers: dict = self._data["cose_headers"]
            formats = "coseHeaderFormats"
        elif header_type == "signature":
            headers: dict = self._data["cose_signature_headers"]
            formats = "coseHeaderFormats"
        else:
            headers: dict = self._data[self.METADATA_KEY]
            formats = self.METADATA_FORMATS_KEY
        cddl_def = headers.get(name)
        if cddl_def is not None:
            cddl_def = cddl_def.get("format")
        if cddl_def is not None:
            cddl_def = self._data[formats].get(cddl_def)
        if cddl_def is not None:
            cddl_def = cddl_def.get("cddl")
        return cddl_def

    def cddl_def(self, name: str) -> dict | None:
        """Get a cddl definition by name."""

        def synthetic_headers(defs: dict, header_type: str = "metadata") -> dict:
            """Generate a synthetic cddl def for this type.

            Needs to be generated from Metadata definitions.
            """
            cddl_def = ""
            defs["requires"] = []
            exclusives = []
            if header_type == "document":
                header_names = self.standard_document_header_names()
                header_map_key = "cose_headers"
            elif header_type == "signature":
                header_names = self.standard_signature_header_names()
                header_map_key = "cose_signature_headers"
            else:
                header_names = self.metadata_names()
                header_map_key = self.METADATA_KEY

            for datum in header_names:
                optional = (
                    ""
                    if self._data[header_map_key][datum]["required"] == "yes"
                    else "?"
                )

                exclusive: list[str] | None = self._data[header_map_key][datum].get(
                    "exclusive"
                )
                if exclusive is not None:
                    exclusive.append(datum)
                    exclusive.sort()
                    if exclusive not in exclusives:
                        exclusives.append(exclusive)
                else:
                    cddl_type = self.cddl_type_for_metadata(datum, header_type)
                    field_name = self._data[header_map_key][datum].get(
                        "coseLabel", datum
                    )
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
                    field_name = self._data[header_map_key][entry].get(
                        "coseLabel", entry
                    )
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

            print(defs)
            return defs

        defs = self._data.get("cddlDefinitions")
        if defs is not None:
            defs = defs.get(name)
            if name == "Signed_Document_Metadata_Headers":
                defs = synthetic_headers(defs)
            elif name == "COSE_Document_Standard_Headers":
                defs = synthetic_headers(defs, "document")
            elif name == "COSE_Signature_Header_Map":
                defs = synthetic_headers(defs, "signature")
        return defs

    def copyright(
        self, document_name: str | None
    ) -> tuple[dict[str, str], dict, list[dict], str]:
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
        copyright = self._data["copyright"]
        versions = copyright["versions"]

        doc_versions = None
        if document_name is not None:
            authors = self._data[self.DOCS_KEY][document_name]["authors"] | authors
            doc_versions = self._data[self.DOCS_KEY][document_name]["versions"]

        latest_change = get_latest_file_change(versions, doc_versions)
        if doc_versions is not None:
            versions = doc_versions

        return (authors, copyright, versions, latest_change)
