"""Generate the metadata.md file."""

import argparse

from doc_generator import DocGenerator
from gen_cddl_file import CDDLFile
from signed_doc_spec import SignedDocSpec


class MetadataMd(DocGenerator):
    """Generate the metadata.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDocSpec) -> None:
        """Initialise Spec.md generator."""
        super().__init__(args, spec, "metadata.md", flags=self.HAS_MARKDOWN_LINKS + self.IS_METADATA_PRIMARY_SOURCE)

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
            print(f"Failed to generate metadata: {e}")
            return False
        return super().generate()
