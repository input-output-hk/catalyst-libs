"""Base Document Generator Class."""

# Autogenerate Documentation Pages from the specification

import argparse
import difflib
import re
import textwrap
import typing
from pathlib import Path

import rich
import rich.markdown
from jinja2 import Environment, FileSystemLoader, select_autoescape

from docs.markdown import MarkdownHelpers
from spec.signed_doc import SignedDoc

__jinja_env: Environment | None = None
TEMPLATES: str = "./pages"


def get_jinja_environment(spec: SignedDoc) -> Environment:
    """Get the current jinja environment for rendering templates."""
    global __jinja_env  # noqa: PLW0603
    if __jinja_env is None:
        __jinja_env = Environment(
            loader=FileSystemLoader(TEMPLATES), autoescape=select_autoescape(), trim_blocks=True, lstrip_blocks=True
        )
        print(Path.cwd())
        __jinja_env.globals["spec"] = spec  # type: ignore reportUnknownMemberType

    return __jinja_env


def get_template_with_path(template: str) -> str:
    """Get a template and its path, just from template name."""
    return next(iter(Path(TEMPLATES).rglob(template))).relative_to(TEMPLATES).as_posix()


class DocGenerator:
    """Base class for individual document generators."""

    NO_FLAGS = 0
    HAS_MARKDOWN_LINKS = 1
    IS_METADATA_PRIMARY_SOURCE = 2

    def __init__(  # noqa: PLR0913
        self,
        args: argparse.Namespace,
        spec: SignedDoc,
        depth: int = 0,
        flags: int = HAS_MARKDOWN_LINKS,
        *,
        filename: str | None = None,
        template: str | None = None,
    ) -> None:
        """Must be called BEFORE subclasses add any customization."""
        self._args = args
        self._spec = spec
        self._generate = args.generate
        self._depth = depth
        self._filedata = ""
        self._has_markdown_links = flags & self.HAS_MARKDOWN_LINKS != 0
        self._is_metadata_primary_source = flags & self.IS_METADATA_PRIMARY_SOURCE != 0
        self._document_name = None

        if template is not None:
            template = get_template_with_path(template)
        if filename is None and template is not None:
            filename = Path(template).relative_to("signed_doc").with_suffix("").as_posix()
        if filename is None:
            msg = "`filename` or `template` (or both) parameters must be defined."
            raise NotImplementedError(msg)

        self._filename = filename
        self._template = template
        self._filepath = Path(args.output).joinpath(filename).resolve()

        # Make sure any destination directory exists.
        self._filepath.parent.mkdir(parents=True, exist_ok=True)

    @staticmethod
    def name_to_doc_page_link(name: str, ref: str | None = None) -> str:
        """Create a link to a document type, and an optional ref inside the document."""
        link = "docs/" + name.lower().replace(" ", "_") + ".md"
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
            self._spec.docs.names,
            MarkdownHelpers.doc_ref_link,
            primary_source=primary_source,
        )

    def add_metadata_links(self) -> None:
        """Add metadata field documentation cross reference links to the document.

        All metadata fields in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.metadata.headers.names,
            MarkdownHelpers.field_link,
            primary_source=self._is_metadata_primary_source,
        )

    def add_metadata_format_links(self) -> None:
        """Add metadata format documentation cross reference links to the document.

        All metadata formats in text must be as `<name>` or they will not be linked.
        """
        self.add_generic_markdown_links(
            self._spec.metadata.formats.all,
            MarkdownHelpers.format_link,
            primary_source=self._is_metadata_primary_source,
        )

    def strip_end_whitespace(self) -> None:
        """Strip all whitespace from the end of any lines."""
        lines = self._filedata.splitlines()
        stripped_lines = [line.rstrip() for line in lines]
        self._filedata = "\n".join(stripped_lines).strip() + "\n"

    def add_reference_links(self, *, html: str | None = None) -> str | None:
        """Add Markdown reference links to the document.

        Only Reference links found to be used by the document will be added.
        """
        # Don't do this if the document does not have markdown style links
        if html is None and not self._has_markdown_links:
            return None

        doc_data = html if html is not None else self._filedata

        self.strip_end_whitespace()

        actual_links_used: dict[str, str] = {}
        for link_name in self._spec.documentation.links.all:
            esc_link_name = re.escape(link_name)
            html_start = "" if html is None else "|>"
            html_end = "" if html is None else "|<"
            link_name_regex = f"(^|\\s{html_start})({esc_link_name})(;|:|,|\\.|\\s{html_end}|$)"
            aka = self._spec.documentation.links.aka(link_name)
            if html:
                link_ref = link_name if aka is None else aka
                replacement = f'\\1<a href="{self._spec.documentation.links.link(link_ref)}">\\2</a>\\3'
            else:
                aka = self._spec.documentation.links.aka(link_name)
                if aka is not None:
                    replacement = f"\\1[\\2][{aka}]\\3"
                    link_name = aka  # noqa: PLW2901
                else:
                    replacement = r"\1[\2]\3"

            (doc_data, cnt) = MarkdownHelpers.block_aware_re_subn(
                doc_data,
                link_name_regex,
                replacement,
            )

            if html is None and cnt != 0:
                actual_links_used[link_name] = self._spec.documentation.links.link(link_name)

        if html is None:
            for link, actual in actual_links_used.items():
                doc_data += f"\n[{link}]: {actual}"
            self._filedata = doc_data
            return None

        return doc_data

    def remove_tabs(self, tabstop: int = 4) -> None:
        """Replace tabs in the input text with spaces so that the text aligns on tab stops.

        Args:
            text (str): The input text containing tabs.
            tabstop (int): The number of characters per tab stop. Default is 4.

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
        lines = list[str](
            self._filedata.splitlines()  # type: ignore  # noqa: PGH003
        )
        no_tabs: list[str] = []
        for line in lines:
            new_line = re.sub(pattern, replace_tab, line)
            no_tabs.append(new_line)

        self._filedata = "\n".join(no_tabs)

    def insert_copyright(self, *, changelog: bool = True) -> str:
        """Generate a copyright notice into the given document data.

        document_name: Name of the signed document we also get copyright info from.
        """
        (authors, copyright_data, versions, global_last_modified) = self._spec.get_copyright(self._document_name)

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
        for author in authors.all():
            copyright_notice += f"|{author_title}| {author} <{authors.email(author)}> |\n"
            author_title = " "

        if changelog:
            copyright_notice += "\n### Changelog\n\n"
            for version in versions:
                copyright_notice += f"""#### {version.version} ({version.modified})

{version.changes}

"""

        return copyright_notice.strip()

    def generate_from_page_template(self, **kwargs: typing.Any) -> None:  # noqa: ANN401
        """Generate a Page from a Page Template inside the specifications."""
        if self._template is not None:
            env = get_jinja_environment(self._spec)
            template = env.get_template(self._template)
            self._filedata = template.render(doc=self, **kwargs)
        else:
            msg = f"No Template for {self._filename} document is defined."
            raise NotImplementedError(msg)

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

    def validate_generation(self) -> bool:
        """Check and Output the status when a file does not validate."""
        if not self._filepath.exists():
            rich.print(f"Documentation file missing: {self._filename}.")
            return False

        current_file = self._filepath.read_text()
        if current_file == self._filedata:
            return True

        rich.print(f"Documentation not generated correctly: {self._filename}.")
        diff = difflib.unified_diff(
            current_file.splitlines(),
            self._filedata.splitlines(),
            fromfile=self._filename,
            tofile="Expected File",
            fromfiledate="",
            tofiledate="",
            n=3,
            lineterm="\n",
        )
        rich.print(
            rich.markdown.Markdown(
                f"""
```diff
{"\n".join(diff)}
```
""",
                code_theme="vim",
            ),
        )
        return False

    def save_or_validate(
        self,
    ) -> bool:
        """Save a file or Validate it, depending on whats required."""
        rich.print(f"{'Generating' if self._generate else 'Validating'} {self._filename}")

        try:
            if not self.generate():
                return False
        except Exception as e:  # noqa: BLE001
            rich.print(f"Failed to generate documentation for {self._filename}: {e}")
            rich.console.Console().print_exception()
            return False

        if self._generate:
            if self._filepath.exists():
                old_contents = self._filepath.read_text()
                if old_contents == self._filedata:
                    rich.print(f"{self._filename} is already up-to-date.")
                    return True

            self._filepath.write_text(self._filedata)
            return True

        return self.validate_generation()

    def file_name(self) -> str:
        """Return the files name."""
        return self._filename

    def file_path(self, relative_doc: "DocGenerator | None" = None) -> Path:
        """Return a path to the file."""
        if relative_doc is not None:
            relative_path = relative_doc.file_path().parent
            return self._filepath.relative_to(relative_path, walk_up=True)
        return self._filepath

    def markdown_reference(
        self,
        *,
        indent: int = 0,
        relative_doc: "DocGenerator | None" = None,
        title: str = "Markdown Document",
        filetype: str = "md",
    ) -> str:
        """Create a Markdown formatted reference for the file."""
        file_path = self.file_path(relative_doc)
        file_name = self.file_name().rsplit("/", 1)[-1]

        return textwrap.indent(
            f"""
<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "{title}"

    * [{file_name}]({file_path})

    ``` {filetype}
    {{{{ include_file('./{file_path}', indent={indent + 4}) }}}}
    ```
<!-- markdownlint-enable max-one-sentence-per-line -->
""".strip(),
            " " * indent,
        )

    def wrap_html(self, html: str) -> str:
        """Wrap HTML so it is OK inside the markdown.

        Also, automatically link any words with known links.
        """
        # This is always returning a string when html is a string.
        # but the type checker can't check that deeply.
        html = self.add_reference_links(html=html)  # type: ignore reportAssignmentType

        return f"""
{MarkdownHelpers.HTML_START}
{MarkdownHelpers.ALLOW_HTML_IN_MD}
{html}
{MarkdownHelpers.DISALLOW_HTML_IN_MD}
{MarkdownHelpers.HTML_END}
""".strip()

    def link_to_file(
        self, name: str, link_file: str, *, template: str | None = None, heading: str | None = None
    ) -> str:
        """Create a link to a file, relative to self."""
        if template is None:
            template = link_file + ".jinja"
        if self._template is None:
            msg = "Not a templated file."
            raise NotImplementedError(msg)
        link_template: Path = Path(TEMPLATES) / get_template_with_path(template)
        this_template: Path = Path(TEMPLATES) / self._template
        relative_template = link_template.resolve().relative_to(this_template.parent.resolve(), walk_up=True)
        relative_file = relative_template.with_name(link_file)

        heading = "#" + heading.lower().replace(" ", "-") if heading is not None else ""

        link = f"[{name}]({relative_file}{heading})"
        print(link)

        return link
