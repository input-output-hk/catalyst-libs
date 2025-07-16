"""Generate the spec.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator, LinkType


class DocIndex(DocGenerator):
    """Doc Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, template="docs/.pages.jinja", flags=self.NO_FLAGS)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        self.generate_from_page_template(LinkType=LinkType)

        return super().generate()
