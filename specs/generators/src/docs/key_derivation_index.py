"""Generate the Key Derivation Index file."""

import argparse
import typing

import rich

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .key_derivation_md import KeyDerivationPageMd


class KeyDerivationIndex(DocGenerator):
    """Key Derivation Index Generator."""

    TEMPLATE: str = "key_derivation/.pages.jinja"

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, template=self.TEMPLATE, flags=self.NO_FLAGS)

    def pages_data(self) -> dict[str, dict[str, typing.Any]]:
        """Get all page data."""
        return DocGenerator.read_md_jinja_with_frontmatter(self.TEMPLATE)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        all_page_data = self.pages_data()
        rich.print(all_page_data)
        good = True
        for page in all_page_data:
            rich.print(page)
            good &= KeyDerivationPageMd.save_or_validate_all(self._args, self._spec, all_page_data)
        self.generate_from_page_template(extra={"pages_data": all_page_data})

        return good & super().generate()
