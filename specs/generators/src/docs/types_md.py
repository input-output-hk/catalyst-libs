"""Generate the types.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .doc_relationship_diagrams import DocRelationshipFile


class TypesMd(DocGenerator):
    """Generate the `types.md` File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, "types.md")

    def doc_type_summary(self) -> str:
        """Generate a Document Base Type Summary from the Document Specifications Data."""
        doc_type_summary = """
| Base Type | UUID | CBOR |
| :--- | :--- | :--- |
"""

        for type_name in self._spec.base_types.all:
            uuid = self._spec.base_types.uuid(type_name)
            doc_type_summary += f"| {type_name} | `{uuid.as_uuid_str}` | `{uuid.as_cbor}` |\n"

        return doc_type_summary.strip()

    def doc_type_details(self) -> str:
        """Generate a Document Type Detailed Summary from the Document Specifications Data."""
        doc_type_details = """
<!-- markdownlint-disable MD033 -->
| Document Type | Base Types | CBOR |
| :--- | :--- | :--- |
"""

        for k in self._spec.docs.names:
            doc_type = self._spec.docs.type(k)
            doc_type_details += (
                f"| [{k}]({self.name_to_spec_link(k)}) |"
                f" {doc_type.formatted_names()} |"
                f" {doc_type.formatted_ids(separator=',<br/>')} |\n"
            )

        doc_type_details += "<!-- markdownlint-enable MD033 -->"

        return doc_type_details.strip()

    def generate(self) -> bool:
        """Generate the `types.md` File."""
        graph = DocRelationshipFile(self._args, self._spec)
        if not graph.save_or_validate():
            return False

        self._filedata = f"""
# Document Types Table

## Document Base Types

All Document Types are defined by composing these base document types:

{self.doc_type_summary()}

## Document Types

All Defined Document Types

{self.doc_type_details()}

## Document Relationship Hierarchy

{graph.markdown_reference(relative_doc=self)}

{self.insert_copyright(changelog=False)}
"""
        return super().generate()
