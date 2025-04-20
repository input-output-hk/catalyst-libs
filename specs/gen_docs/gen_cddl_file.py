# Generate the spec.md file

import re
from pathlib import Path

from doc_generator import DocGenerator


def add_cddl_comments(comment: str) -> tuple[str, bool]:
    """Add cddl comment markers to lines.

    Returns True if more than 1 line."""
    comment = comment.strip()
    comment_lines = comment.splitlines()
    comment = ""
    for line in comment_lines:
        comment += f"; {line}\n"
    comment = comment.strip()

    return comment, len(comment_lines) > 0


class CDDLFile(DocGenerator):
    def __init__(self, args, spec, cddl_root: str):
        file_name = (
            "cddl/" + cddl_root.lower().replace(" ", "_").replace("-", "_") + ".cddl"
        )

        super().__init__(args, spec, file_name, flags=self.NO_FLAGS)
        self._cddl_root = cddl_root

    def get_cddl(self, name, found=[]) -> tuple[str, list[str]]:
        """Get the CDDL for a metadatum."""
        this_cddl = ""
        this_def = self._spec.cddl_def(name)
        cddl_def: str = this_def["def"].strip()
        cddl_def_multiline = len(cddl_def.splitlines()) > 1

        # Add required definitions to this one (recursive)
        for requires in this_def["requires"]:
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

    def generate(self):
        cddl_data, _ = self.get_cddl(self._cddl_root)
        defs = self._spec.cddl_def(self._cddl_root)

        description, _ = add_cddl_comments(
            defs.get("description", f"{self._cddl_root}")
        )

        # Remove double line breaks,
        # so we only ever have 1 between definitions
        cddl_data = re.sub(r"\n\n[\n]+", "\n\n", cddl_data)

        self._filedata = f"""
{description}


{cddl_data.strip()}
"""
        super().generate()

    def file_path(self, depth: int = 0) -> Path:
        """Return a path to the file"""
        path = self._filename
        while depth > 0:
            depth -= 1
            path = "../" + path
        return path
