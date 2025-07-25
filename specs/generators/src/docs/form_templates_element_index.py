"""Generate the Form Template Element Index file."""

import argparse

from docs.form_templates_element_md import FormTemplatesElementMd
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator, LinkType


class FormTemplatesElementIndex(DocGenerator):
    """Form Template Element Index Generator."""

    TEMPLATE: str = "form_template_elements/.pages.jinja"
    ELEMENT_TEMPLATE: str = FormTemplatesElementMd.TEMPLATE

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialize."""
        super().__init__(args, spec, template=self.TEMPLATE, flags=self.NO_FLAGS)

    def generate(self) -> bool:
        """Generate the Spec Index."""
        pages_ok = FormTemplatesElementMd.save_or_validate_all(self._args, self._spec)

        self.generate_from_page_template(LinkType=LinkType)

        return pages_ok and super().generate()
