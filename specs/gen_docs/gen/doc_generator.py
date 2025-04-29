"""Base Document Generator Class."""

# Autogenerate Documentation Pages from the specification

import argparse
import difflib
import re
import typing
from pathlib import Path

import rich

from spec.metadata import Metadata
from spec.signed_doc import HeaderType, SignedDocSpec


class DocGenerator:
    """Base class for individual document generators."""

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
    ) -> None:
        """Must be called BEFORE subclasses add any customization."""
        self._args = args
        self._spec = spec
        self._filename = filename
        self._filepath = Path(args.output).joinpath(self._filename).resolve()
        self._generate = args.generate
        self._depth = depth
        self._filedata = ""
        self._has_markdown_links = flags & self.HAS_MARKDOWN_LINKS != 0
        self._is_metadata_primary_source = flags & self.IS_METADATA_PRIMARY_SOURCE != 0
        self._document_name = None

    @staticmethod
    def uuid_as_cbor(uuid: str) -> str:
        """UUID in CBOR Diagnostic Notation."""
        return f"37(h'{uuid.replace('-', '')}')"

    @staticmethod
    def name_to_spec_link(name: str, ref: str | None = None) -> str:
        """Create a link to a document type, and an optional ref inside the document."""
        link = "./docs/" + name.lower().replace(" ", "_") + ".md"
        if ref is not None:
            link += f"#{ref}"
        return link

    def add_generic_markdown_links(
        self,
        field_names: list[str],
        link_fmt_func: typing.Callable[[str, int], str],
        *,
        primary_source: bool = False,
    ) -> None:
        """Do NOT be used directly."""
        # Don't do this if the document does not have markdown style links
        if not self._has_markdown_links:
            return

        lines = self._filedata.splitlines()
        file_data = ""
        for line in lines:
            if not primary_source or not line.startswith("#"):
                for field_name in field_names:
                    field_name_regex = f"(^|\\s)`{field_name}`(\\.|\\s|$)"
                    replacement = f"\\1{link_fmt_func(field_name, self._depth)}\\2"
                    line = re.sub(  # noqa: PLW2901
                        field_name_regex,
                        replacement,
                        line,
                        flags=re.IGNORECASE | re.MULTILINE,
                    )
            file_data += f"{line}\n"

        self._filedata = file_data

    def add_doc_ref_links(self, *, primary_source: bool = False) -> None:
        """Add Individual Document Reference cross reference links to the document.

        All Document References in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.document_names(),
            Metadata.doc_ref_link,
            primary_source=primary_source,
        )

    def add_metadata_links(self) -> None:
        """Add metadata field documentation cross reference links to the document.

        All metadata fields in text must be as `<name>` or they will not be linked.
        """
        _, metadata_names, _ = self._spec.headers_and_order(
            header_type=HeaderType.METADATA
        )
        self.add_generic_markdown_links(
            metadata_names,
            Metadata.field_link,
            primary_source=self._is_metadata_primary_source,
        )

    def add_metadata_format_links(self) -> None:
        """Add metadata format documentation cross reference links to the document.

        All metadata formats in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.format_names(header_type=HeaderType.METADATA),
            Metadata.format_link,
            primary_source=self._is_metadata_primary_source,
        )

    def strip_end_whitespace(self) -> None:
        """Strip all whitespace from the end of any lines."""
        lines = self._filedata.splitlines()
        stripped_lines = [line.rstrip() for line in lines]
        self._filedata = "\n".join(stripped_lines).strip() + "\n"

    def code_block_aware_re_subn(
        self,
        link_name_regex: str | re.Pattern[str],
        replacement: str | typing.Callable[[re.Match[str]], str],
    ) -> bool:
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
                (line, this_cnt) = re.subn(  # noqa: PLW2901
                    link_name_regex,
                    replacement,
                    line,
                    flags=re.IGNORECASE,
                )
            cnt += this_cnt
            new_file_data += line + "\n"

        self._filedata = new_file_data

        return cnt != 0

    def add_reference_links(self) -> None:
        """Add Markdown reference links to the document.

        Only Reference links found to be used by the document will be added.
        """
        # Don't do this if the document does not have markdown style links
        if not self._has_markdown_links:
            return

        self.strip_end_whitespace()

        actual_link_names = self._spec.link_names()

        actual_links_used = {}
        for link_name in actual_link_names:
            esc_link_name = re.escape(link_name)
            link_name_regex = f"(^|\\s)({esc_link_name})(\\.|\\s|$)"
            aka = self._spec.link_aka(link_name)
            if aka is not None:
                replacement = f"\\1[\\2][{aka}]\\3"
                link_name = aka  # noqa: PLW2901
            else:
                replacement = r"\1[\2]\3"

            if self.code_block_aware_re_subn(
                link_name_regex,
                replacement,
            ):
                actual_links_used[link_name] = self._spec.link_for_link_name(link_name)

        for link, actual in actual_links_used.items():
            self._filedata += f"\n[{link}]: {actual}"

    def remove_tabs(self, tabstop: int = 4) -> str:
        """Replace tabs in the input text with spaces so that the text aligns on tab stops.

        Args:
            text (str): The input text containing tabs.
            tabstop (int): The number of characters per tab stop. Default is 8.

        Returns:
            str: Text with tabs replaced by spaces, aligned at each tab stop.

        """
        # Create a regex pattern to match any tab character
        pattern = "\t"

        def replace_tab(match: re.Match[str]) -> str:
            # Calculate the number of spaces needed to reach the next tab stop
            position = match.start()
            return " " * (tabstop - (position % tabstop))

        # Substitute tabs with spaces, using a custom replacement function
        no_tabs_text = re.sub(pattern, replace_tab, self._filedata)

        self._filedata = no_tabs_text

    def insert_copyright(self, *, changelog: bool = True) -> str:
        """Generate a copyright notice into the given document data.

        document_name: Name of the signed document we also get copyright info from.
        """
        (authors, copyright_data, versions, global_last_modified) = (
            self._spec.copyright(self._document_name)
        )

        copyright_year = copyright_data.created.year
        last_modified_year = global_last_modified.year
        if last_modified_year != copyright_year:
            copyright_year = f"{copyright_year:04}-{last_modified_year:04}"
        else:
            copyright_year = f"{copyright_year:04}"

        copyright_notice = (
            f"""
## Copyright

| Copyright | :copyright: {copyright_year} {copyright_data.copyright} |
| --- | --- |
| License | This document is licensed under {copyright_data.license} |
| Created | {copyright_data.created} |
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
                copyright_notice += f"""#### {version.version} ({version.modified})

{version.changes}

"""

        return copyright_notice.strip()

    def generate(self) -> bool:
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

        return True

    def save_or_validate(
        self,
    ) -> bool:
        """Save a file or Validate it, depending on whats required."""
        if not self.generate():
            return False

        if self._generate:
            rich.print(f"Generating {self._filename}")
            if self._filepath.exists():
                old_contents = self._filepath.read_text()
                if old_contents == self._filedata:
                    rich.print(f"{self._filename} is already up-to-date.")
                    return True

            self._filepath.write_text(self._filedata)
            return True

        rich.print(f"Validating {self._filename}")
        if not self._filepath.exists():
            rich.print(f"Documentation file missing: {self._filename}.")
            return False

        current_file = self._filepath.read_text()
        if current_file != self._filedata:
            rich.print(f"Documentation not generated correctly: {self._filename}.")
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
                rich.print(line.rstrip())
            return False
        return True

    def file_name(self) -> str:
        """Return the files name."""
        return self._filename

    def file_path(self, relative_doc: typing.Self | None = None) -> Path:
        """Return a path to the file."""
        if relative_doc is not None:
            relative_path = relative_doc.file_path().parent
            return self._filepath.relative_to(relative_path)
        return self._filepath
