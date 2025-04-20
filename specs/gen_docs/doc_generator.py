#!/usr/bin/env python

# Autogenerate Documentation Pages from the formal specification

import argparse
import difflib
import re
import typing
from pathlib import Path

from common import doc_ref_link, metadata_field_link, metadata_format_link
from signed_doc_spec import SignedDocSpec


class DocGenerator:
    """base class for individual document generators"""

    NO_FLAGS = 0
    HAS_MARKDOWN_LINKS = 1
    IS_METADATA_PRIMARY_SOURCE = 2

    def __init__(
        self,
        args: argparse.Namespace,
        spec: SignedDocSpec,
        filename: str,
        depth: int = 0,
        flags: int = HAS_MARKDOWN_LINKS,
    ):
        # Must be called BEFORE subclasses add any customization.
        self._args = args
        self._spec = spec
        self._filename = filename
        self._filepath = Path(args.output).joinpath(self._filename)
        self._generate = args.generate
        self._depth = depth
        self._filedata = ""
        self._has_markdown_links = flags & self.HAS_MARKDOWN_LINKS != 0
        self._is_metadata_primary_source = flags & self.IS_METADATA_PRIMARY_SOURCE != 0
        self._document_name = None

    def add_generic_markdown_links(
        self,
        field_names: list[str],
        link_fmt_func: typing.Callable[[str, int], str],
        primary_source=False,
    ):
        """Should NOT be used directly."""
        # Don't do this if the document does not have markdown style links
        if not self._has_markdown_links:
            return

        lines = self._filedata.splitlines()
        file_data = ""
        for line in lines:
            if not primary_source or not line.startswith("#"):
                for field_name in field_names:
                    fieldNameRegex = f"(^|\\s)`{field_name}`(\\.|\\s|$)"
                    replacement = f"\\1{link_fmt_func(field_name, self._depth)}\\2"
                    line = re.sub(
                        fieldNameRegex,
                        replacement,
                        line,
                        flags=re.IGNORECASE | re.MULTILINE,
                    )
            file_data += f"{line}\n"

        self._filedata = file_data

    def add_doc_ref_links(self, primary_source=False):
        """Add Individual Document Reference cross reference links to the document.
        All Document References in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.document_names(),
            doc_ref_link,
            primary_source,
        )

    def add_metadata_links(self):
        """Add metadata field documentation cross reference links to the document.
        All metadata fields in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.metadata_names(),
            metadata_field_link,
            self._is_metadata_primary_source,
        )

    def add_metadata_format_links(self):
        """Add metadata format documentation cross reference links to the document.
        All metadata formats in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.metadata_format_names(),
            metadata_format_link,
            self._is_metadata_primary_source,
        )

    def strip_end_whitespace(self):
        """Strip all whitespace from the end of any lines."""
        lines = self._filedata.splitlines()
        stripped_lines = [line.rstrip() for line in lines]
        self._filedata = "\n".join(stripped_lines).strip() + "\n"

    def code_block_aware_re_subn(self, linkNameRegex, replacement) -> bool:
        """Do a multiline regex replacement, but ignore anything inside a code block."""
        lines = self._filedata.splitlines()
        new_file_data = ""
        cnt = 0
        in_code_block = False
        for line in lines:
            if line.strip().startswith("```"):
                in_code_block = not in_code_block

            if in_code_block:
                this_cnt = 0
            else:
                (line, this_cnt) = re.subn(
                    linkNameRegex,
                    replacement,
                    line,
                    flags=re.IGNORECASE,
                )
            cnt += this_cnt
            new_file_data += line + "\n"

        self._filedata = new_file_data

        return cnt != 0

    def add_reference_links(self):
        """Add Markdown reference links to the document.
        Only Reference links found to be used by the document will be added.
        """
        # Don't do this if the document does not have markdown style links
        if not self._has_markdown_links:
            return

        self.strip_end_whitespace()

        allLinkNames = self._spec.link_names()

        actualLinksUsed = {}
        for linkName in allLinkNames:
            escLinkName = re.escape(linkName)
            linkNameRegex = f"(^|\\s)({escLinkName})(\\.|\\s|$)"
            aka = self._spec.link_aka(linkName)
            if aka is not None:
                replacement = f"\\1[\\2][{aka}]\\3"
                linkName = aka
            else:
                replacement = r"\1[\2]\3"

            if self.code_block_aware_re_subn(
                linkNameRegex,
                replacement,
            ):
                actualLinksUsed[linkName] = self._spec.link_for_linkname(linkName)

        for link in actualLinksUsed:
            self._filedata += f"\n[{link}]: {actualLinksUsed[link]}"

    def remove_tabs(self, tabstop: int = 4):
        """Replace tabs in the input text with spaces so that the text aligns on tab stops.

        Args:
            text (str): The input text containing tabs.
            tabstop (int): The number of characters per tab stop. Default is 8.

        Returns:
            str: Text with tabs replaced by spaces, aligned at each tab stop.

        """
        # Create a regex pattern to match any tab character
        pattern = "\t"

        def replace_tab(match):
            # Calculate the number of spaces needed to reach the next tab stop
            position = match.start()
            return " " * (tabstop - (position % tabstop))

        # Substitute tabs with spaces, using a custom replacement function
        no_tabs_text = re.sub(pattern, replace_tab, self._filedata)

        self._filedata = no_tabs_text

    def insert_copyright(self, changelog=True) -> str:
        """Generate a copyright notice into the given document data.

        document_name: Name of the signed document we also get copyright info from.
        """
        (authors, copyright, versions, global_last_modified) = self._spec.copyright(
            self._document_name
        )

        copyright_year = copyright["created"][:4]
        last_modified_year = global_last_modified[:4]
        if last_modified_year != copyright_year:
            copyright_year = f"{copyright_year}-{last_modified_year}"

        copyright_notice = (
            f"""
## Copyright

| Copyright | :copyright: {copyright_year} {copyright["copyright"]} |
| --- | --- |
| License | This document is licensed under {copyright["license"]} |
| Created | {copyright["created"]} |
| Modified | {global_last_modified} |
""".strip()
            + "\n"
        )

        author_title = " Authors "
        for author in sorted(authors):
            copyright_notice += f"|{author_title}| {author} <{authors[author]}> |\n"
            author_title = " "

        if changelog:
            copyright_notice += "\n### Changelog\n\n"
            for version in versions:
                copyright_notice += f"#### {version['version']} ({version['modified']})\n\n{version['changes']}\n\n"

        return copyright_notice.strip()

    def generate(self):
        """Generate the document.
        Must be implemented by Sub Classes
        Which then call this for all common processing.
        """
        # Actual contents must be generated by subclass before calling this.

        # Remove any tabs from the file data.
        self.remove_tabs()

        # Add any links we find in the document.
        self.add_reference_links()
        self.add_metadata_links()
        self.add_metadata_format_links()
        self.add_doc_ref_links()

        # Remove any leading or trailing newlines and add a single newline at the end/
        # Helps make clean markdown files.
        self.strip_end_whitespace()

    def save_or_validate(
        self,
    ) -> bool:
        """Save a file or Validate it, depending on whats required."""
        self.generate()

        if self._generate:
            print(f"Generating {self._filename}")
            if self._filepath.exists():
                old_contents = self._filepath.read_text()
                if old_contents == self._filedata:
                    print(f"{self._filename} is already up-to-date.")
                    return True

            self._filepath.write_text(self._filedata)
            return True

        print(f"Validating {self._filename}")
        if not self._filepath.exists():
            print(f"Documentation file missing: {self._filename}.")
            return False

        current_file = self._filepath.read_text()
        if current_file != self._filedata:
            print(f"Documentation not generated correctly: {self._filename}.")
            diff = difflib.unified_diff(
                current_file.splitlines(),
                self._filedata.splitlines(),
                fromfile="Existing File",
                tofile="Expected File",
                fromfiledate="",
                tofiledate="",
                n=3,
                lineterm="\n",
            )
            for line in diff:
                print(line.rstrip())
            return False
        return True
