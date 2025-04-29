"""Generate the metadata.md file."""

import argparse

import rich
from rich.console import Console

from gen.cddl_file import CDDLFile
from gen.doc_generator import DocGenerator
from spec.signed_doc import SignedDocSpec

console = Console()


class MetadataMd(DocGenerator):
    """Generate the metadata.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDocSpec) -> None:
        """Initialise metadata.md generator."""
        super().__init__(
            args,
            spec,
            "metadata.md",
            flags=self.HAS_MARKDOWN_LINKS + self.IS_METADATA_PRIMARY_SOURCE,
        )

    def metadata_types(self) -> str:
        """Generate the metadata types documentation."""
        metadata_types = ""

        for format_name in self._spec.get_all_metadata_formats():
            format_def = self._spec.get_metadata_format(format_name)

            cddl_def = CDDLFile(self._args, self._spec, format_def.cddl)
            if not cddl_def.save_or_validate():
                raise ValueError
            cddl_markdown_ref = cddl_def.markdown_reference(relative_doc=self)
            metadata_types += f"""
### {format_name}

{format_def.description}

{cddl_markdown_ref}
"""

        return metadata_types.strip()

    def generate(self) -> bool:
        """Generate the `types.md` File."""
        try:
            self._filedata = f"""
# Metadata Fields

## Metadata Types

The following types of metadata have been defined.
All Metadata fields use one of these types.

{self.metadata_types()}

## Individual Metadata field definitions

{self._spec.get_metadata_as_markdown()}

{self.insert_copyright(changelog=False)}
"""
        except Exception as e:  # noqa: BLE001
            rich.print(f"Failed to generate metadata: {e}")
            console.print_exception()
            return False
        return super().generate()

    @staticmethod
    def format_link(name: str, depth: int = 0) -> str:
        """Metadata Format link."""
        link = f"metadata.md#{name.lower().replace(' ', '-')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        return f"[{name}]({link})"

    @staticmethod
    def field_link(name: str, depth: int = 0) -> str:
        """Metadata Field link."""
        link = f"metadata.md#{name.lower().replace('`', '')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        return f"[`{name}`]({link})"
