"""Generate the metadata.md file."""

import argparse
from typing import NamedTuple

import rich
from rich.console import Console

from spec.metadata_formats import MetadataFormat
from spec.signed_doc import SignedDoc

from .cddl_file import CDDLFile
from .doc_generator import DocGenerator

console = Console()


class MetadataType(NamedTuple):
    """Metadata Data Extract."""

    name: str
    format_def: MetadataFormat
    cddl_def: CDDLFile


class MetadataMd(DocGenerator):
    """Generate the metadata.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise metadata.md generator."""
        super().__init__(
            args,
            spec,
            template="metadata.md.jinja",
            flags=self.HAS_MARKDOWN_LINKS + self.IS_METADATA_PRIMARY_SOURCE,
        )

    def metadata_types(self) -> list[MetadataType]:
        """Generate the metadata types documentation."""
        metadata_types: list[MetadataType] = []

        for format_name in self._spec.metadata.formats.all:
            format_def = self._spec.metadata.formats.get(format_name)

            cddl_def = CDDLFile(self._args, self._spec, format_def.cddl)
            if not cddl_def.save_or_validate():
                raise ValueError
            metadata_types.append(MetadataType(format_name, format_def, cddl_def))

        return metadata_types

    def generate(self) -> bool:
        """Generate the `metadata.md` File."""
        try:
            self.generate_from_page_template()
        except Exception as e:  # noqa: BLE001
            rich.print(f"Failed to generate metadata: {e}")
            console.print_exception()
            return False
        return super().generate()
