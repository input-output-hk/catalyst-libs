"""Generate the spec.md file."""

import argparse

from docs.docs_page_md import IndividualDocMd
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator, LinkType


class DocIndex(DocGenerator):
    """Doc Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, template="docs/.pages.jinja", flags=self.NO_FLAGS)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        # Generate all the pages for each document type.
        good = IndividualDocMd.save_or_validate_all(self._args, self._spec)

        self.generate_from_page_template(LinkType=LinkType)

        return good and super().generate()
