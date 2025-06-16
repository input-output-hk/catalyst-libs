"""Generate CDDL Files."""

import argparse
import textwrap

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class CDDLFile(DocGenerator):
    """Generate a CDDL File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, cddl_root: str) -> None:
        """CDDL File Generator."""
        file_name = "cddl/" + cddl_root.lower().replace(" ", "_").replace("-", "_") + ".cddl"

        super().__init__(args, spec, file_name, flags=self.NO_FLAGS)
        self._cddl_root = cddl_root

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
