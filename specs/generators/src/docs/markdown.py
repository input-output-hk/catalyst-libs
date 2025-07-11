"""Markdown Helper and Formatting Functions."""

import re
import typing


class MarkdownHelpers:
    """Markdown Helper and Formatting Functions."""

    HTML_START = "<!---HTML START-->"
    HTML_END = "<!---HTML END-->"

    # Multiple Line Blanks, Line Length, Inline HTML and max-sentence-per-line
    MD_LINTS = "MD012 MD013 MD033 max-one-sentence-per-line"
    ALLOW_HTML_IN_MD = f"<!-- markdownlint-disable {MD_LINTS} -->"
    DISALLOW_HTML_IN_MD = f"<!-- markdownlint-enable {MD_LINTS} -->"

    @staticmethod
    def format_link(name: str, depth: int = 0, *, file: str = "metadata.md", monospace: bool = False) -> str:
        """Format link."""
        link = f"{file}#{name.lower().replace(' ', '-')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        if monospace:
            name = f"`{name}`"

        return f"[{name}]({link})"

    @staticmethod
    def doc_ref_link(name: str, depth: int = 0, *, html: bool = False) -> str:
        """Metadata Document Reference link."""
        link = name.lower().replace(" ", "_")

        if html:
            link += "/"
        else:
            link += ".md"

        if depth == 0:
            link = f"./docs/{link}"
        else:
            maxdepth = 0 if html else 1
            while depth > maxdepth:
                link = f"../{link}"
                depth -= 1

        if html:
            return link
        return f"[{name}]({link})"

    @staticmethod
    def field_link(name: str, depth: int = 0) -> str:
        """Metadata Field link."""
        return MarkdownHelpers.format_link(name, depth, monospace=True)

    @classmethod
    def block_aware_re_subn(
        cls,
        doc: str,
        link_name_regex: str | re.Pattern[str],
        replacement: str | typing.Callable[[re.Match[str]], str],
        *,
        ignore_html: bool = True,
        ignore_code: bool = True,
    ) -> tuple[str, bool]:
        """Do a multiline regex replacement.

        But ignore anything inside a code block.
        And anything thats pure HTML.
        """
        lines = doc.splitlines()
        new_file_data = ""
        cnt = 0
        in_code_block = False
        in_html_block = False
        for line in lines:
            # We ignore HTML inside code blocks.
            if ignore_html:
                if not in_code_block and line.strip().startswith(cls.HTML_START):
                    in_html_block = True
                if in_html_block and line.strip().startswith(cls.HTML_END):
                    in_html_block = False

            # We ignore code blocks that appear inside HTML.
            if ignore_code and not in_html_block and line.strip().startswith("```"):
                in_code_block = not in_code_block

            # We don't do any replacements in lines that are inside code or html blocks.
            if in_html_block or in_code_block:
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

        return (new_file_data, cnt != 0)
