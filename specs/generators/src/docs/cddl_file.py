"""Generate CDDL Files."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class CDDLFile(DocGenerator):
    """Generate a CDDL File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, cddl_root: str) -> None:
        """CDDL File Generator."""
        file_name = "cddl/" + cddl_root.lower().replace(" ", "_").replace("-", "_") + ".cddl"

        super().__init__(args, spec, filename=file_name, flags=self.NO_FLAGS)
        self._cddl_root = cddl_root

    def markdown_reference(
        self,
        *,
        indent: int = 0,
        relative_doc: DocGenerator | None = None,
        title: str = "CDDL Specification",
        filetype: str = "cddl",
    ) -> str:
        """Create a Markdown formatted reference for the CDDL file."""
        return super().markdown_reference(indent=indent, relative_doc=relative_doc, title=title, filetype=filetype)

    def generate(self) -> bool:
        """Generate a CDDL File."""
        self._filedata = self._spec.cddl_definitions.cddl_file(self._cddl_root)

        return super().generate()

    def contents(self) -> str:
        """Get the contents of the CDDL file."""
        return self._filedata
