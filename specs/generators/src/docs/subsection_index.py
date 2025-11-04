"""Generate the Key Derivation Index file."""

import argparse
import typing

import rich

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .subsection_page_md import SubSectionPageMd


class SubSectionIndex(DocGenerator):
    """Key Derivation Index Generator."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, template: str) -> None:
        """Initialize."""
        self._template = template
        super().__init__(args, spec, template=self.TEMPLATE, flags=self.NO_FLAGS)

    def pages_data(self) -> dict[str, dict[str, typing.Any]]:
        """Get all page data."""
        return DocGenerator.read_md_jinja_with_frontmatter(self._template)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        all_page_data = self.pages_data()
        rich.print(all_page_data)
        good = True
        for page in all_page_data:
            rich.print(page)
            good &= SubSectionPageMd.save_or_validate_all(self._args, self._spec, all_page_data)
        self.generate_from_page_template(extra={"pages_data": all_page_data})

        return good & super().generate()
