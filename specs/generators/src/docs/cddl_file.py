"""Generate CDDL Files."""

import argparse
import textwrap

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


def x_add_cddl_comments(comment: str) -> tuple[str, bool]:
    """Add cddl comment markers to lines.

    Returns True if more than 1 line.
    """
    comment = comment.strip()
    comment_lines = comment.splitlines()
    comment = ""
    for line in comment_lines:
        comment += f"; {line}\n"
    comment = comment.strip()

    return comment, len(comment_lines) > 0


class CDDLFile(DocGenerator):
    """Generate a CDDL File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, cddl_root: str) -> None:
        """CDDL File Generator."""
        file_name = "cddl/" + cddl_root.lower().replace(" ", "_").replace("-", "_") + ".cddl"

        super().__init__(args, spec, file_name, flags=self.NO_FLAGS)
        self._cddl_root = cddl_root

    def x_get_cddl(self, name: str, found: list[str] | None = None) -> tuple[str, list[str]]:
        """Get the CDDL for a metadatum."""
        if found is None:
            found = []

        this_cddl = ""
        this_def = self._spec.cddl_definitions.get(name)
        cddl_def = this_def.definition.strip()
        cddl_def_multiline = len(cddl_def.splitlines()) > 1

        # Add required definitions to this one (recursive)
        for requires in this_def.requires:
            if requires not in found:
                next_cddl, found = self.get_cddl(requires, found)
                found.append(requires)
                this_cddl += next_cddl

        comment: str = this_def.get("comment", "")
        leading_comment = ""
        if len(comment) > 0:
            comment, multiline = add_cddl_comments(comment)
            if multiline or cddl_def_multiline:
                leading_comment = comment
                comment = "\n"  # Adds a blank line after defs with multiline comments

        this_cddl = f"""
{leading_comment}
{name} = {cddl_def} {comment}

{this_cddl}
"""

        return this_cddl, found

    def markdown_reference(self, *, indent: int = 0, relative_doc: DocGenerator | None = None) -> str:
        """Create a Markdown formatted reference for the CDDL file."""
        file_path = self.file_path(relative_doc)
        file_name = self.file_name()

        return textwrap.indent(
            f"""
<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [{file_name}]({file_path})

    ```cddl
    {{{{ include_file('./{file_path}', indent={indent + 4}) }}}}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->
""".strip(),
            " " * indent,
        )

    def generate(self) -> bool:
        """Generate a CDDL File."""
        self._filedata = self._spec.cddl_definitions.cddl_file(self._cddl_root)

        return super().generate()
