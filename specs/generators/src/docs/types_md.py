"""Generate the types.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .doc_relationship_diagrams import DocRelationshipFile


class TypesMd(DocGenerator):
    """Generate the `types.md` File."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, template="types.md.jinja")

    def doc_type_details(self) -> str:
        """Generate a Document Type Detailed Summary from the Document Specifications Data."""
        doc_type_details = """
<!-- markdownlint-disable MD033 -->
| Document Type | UUIDv4 | CBOR |
| :--- | :--- | :--- |
"""

        for k in self._spec.docs.names:
            doc_type = self._spec.docs.type(k)
            doc_type_details += (
                f"| {self.link_to_file(k, doc_name=k, template='document_page.md.jinja')} |"
                f" {doc_type.as_uuid_str} |"
                f" `{doc_type.as_cbor}` |\n"
            )

        doc_type_details += "<!-- markdownlint-enable MD033 -->"

        return doc_type_details.strip()

    def generate(self) -> bool:
        """Generate the `types.md` File."""
        graph = DocRelationshipFile(self._args, self._spec)
        if not graph.save_or_validate():
            return False

        self.generate_from_page_template(graph=graph)

        return super().generate()
