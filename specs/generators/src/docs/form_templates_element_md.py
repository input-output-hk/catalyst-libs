"""Generate the templates.md file."""

import argparse

from docs.form_template_basic_schema_json import FormTemplateBasicSchemaJson
from docs.form_template_example_schema_json import FormTemplateExampleSchemaJson
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplatesMd(DocGenerator):
    """Generate the form_templates.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, name: str) -> None:
        """Initialise form_templates.md generator."""
        super().__init__(args, spec, doc_name=name, template="form_templates_element.md.jinja")

    @classmethod
    def save_or_validate_all(cls, args: argparse.Namespace, spec: SignedDoc) -> bool:
        """Save or Validate all documentation pages."""
        good = True
        for doc_name in spec.form_template.elements.names():
            good &= cls(args, spec, doc_name).save_or_validate()

        return good

    def generate(self) -> bool:
        """Generate a `form_templates_element.md` type file from the definitions."""
        # Generate the example_template_schema.json file.
        example_schema = FormTemplateExampleSchemaJson(self._args, self._spec)
        if not example_schema.save_or_validate():
            return False

        base_schema = FormTemplateBasicSchemaJson(self._args, self._spec)
        if not base_schema.save_or_validate():
            return False

        self.generate_from_page_template(example_schema=example_schema, base_schema=base_schema)

        return super().generate()
