"""Generate the types.md file."""

import argparse

from gen.doc_generator import DocGenerator
from gen.doc_relationship_diagrams import DocRelationshipFile
from spec.signed_doc import SignedDoc


class TypesMd(DocGenerator):
    """Generate the `types.md` File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, "types.md")

    def formatted_doc_types(self, name: str) -> str:
        """Return a formatted doc types entry."""
        types = self._spec.document_type(name)
        type_names = ""
        for sub_type in types:
            type_names += f"{self._spec.doc_name_for_type(sub_type)}/"
        return type_names[:-1]

    def formatted_cbor_doc_types(self, name: str) -> str:
        """Return doc types formatted as cbor."""
        types = self._spec.document_type(name)
        type_names = "["
        for sub_type in types:
            type_names += self.uuid_as_cbor(sub_type) + ",<br/>"
        return type_names[:-6] + "]"

    def doc_type_summary(self) -> None:
        """Generate a Document Base Type Summary from the Document Specifications Data."""
        doc_types = self._spec.base_document_types()

        doc_type_summary = """
| Base Type | UUID | CBOR |
| :--- | :--- | :--- |
"""

        for k, v in doc_types.items():
            doc_type_summary += f"| {k} | `{v}` | `{self.uuid_as_cbor(v)}` |\n"

        return doc_type_summary.strip()

    def doc_type_details(self) -> str:
        """Generate a Document Type Detailed Summary from the Document Specifications Data."""
        docs = self._spec.document_names()

        doc_type_details = """
<!-- markdownlint-disable MD033 -->
| Document Type | Base Types | CBOR |
| :--- | :--- | :--- |
"""

        for k in docs:
            doc_type_details += (
                f"| [{k}]({self.name_to_spec_link(k)}) |"
                f" {self.formatted_doc_types(k)} |"
                f" {self.formatted_cbor_doc_types(k)} |\n"
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
