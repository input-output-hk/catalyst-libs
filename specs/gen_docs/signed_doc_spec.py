#!/usr/bin/env python

# Autogenerate Documentation Pages from the formal specification

import json


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
        metadata: dict = self._data[self.METADATA_KEY]
        return metadata.keys()

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
