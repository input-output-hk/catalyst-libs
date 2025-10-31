"""Generate the form_templates_element.md.jinja templated files."""

import argparse
from pathlib import Path
from typing import Any

import rich

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class KeyDerivationPageMd(DocGenerator):
    """Generate a single Key Derivation Page from its template."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, page: dict[str, Any]) -> None:
        """Initialise form templates Element documentation generator."""
        self._spec = spec
        self._extra_ = page
        rich.print(page)
        doc_name = page["front_matter"]["Title"]
        template = page["path"].name
        filename = Path(page["path"].parent.name) / page["path"].stem
        rich.print(f"Generating Key Derivation Page: {doc_name} -> {filename}")

        super().__init__(args, spec, doc_name=doc_name, filename=filename, template=template)

    @classmethod
    def save_or_validate_all(cls, args: argparse.Namespace, spec: SignedDoc, pages: list[dict[str, Any]]) -> bool:
        """Save or Validate all documentation pages."""
        good = True
        for page in pages:
            good &= cls(args, spec, page).save_or_validate()

        return good

    def generate(self) -> bool:
        """Generate a `key_derivation.md` type file from the definitions."""
        self.generate_from_page_template(extra=self._extra_)

        return super().generate()
