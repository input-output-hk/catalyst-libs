"""Generate the spec.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class SpecIndex(DocGenerator):
    """Spec Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, flags=self.NO_FLAGS, template=".pages.jinja")

    def generate(self) -> bool:
        """Generate the Spec Index."""
        self.generate_from_page_template()

        return super().generate()
