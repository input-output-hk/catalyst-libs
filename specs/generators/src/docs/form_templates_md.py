"""Generate the templates.md file."""

import argparse

from docs.form_template_basic_schema_json import FormTemplateBasicSchemaJson
from docs.form_template_example_schema_json import FormTemplateExampleSchemaJson
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplatesMd(DocGenerator):
    """Generate the form_templates.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise form_templates.md generator."""
        super().__init__(args, spec, "form_templates.md")

    def all_form_elements(self) -> str:
        """List and cross reference all defined form elements."""
        # self._spec.form_template.elements.root.i
        return """
### TODO
"""

    def generate(self) -> bool:
        """Generate a `form_templates.md` file from the definitions."""
        # Generate the example_template_schema.json file.
        example_schema = FormTemplateExampleSchemaJson(self._args, self._spec)
        if not example_schema.save_or_validate():
            return False

        base_schema = FormTemplateBasicSchemaJson(self._args, self._spec)
        if not base_schema.save_or_validate():
            return False

        self.generate_from_page_template(
            "form_templates.md.jinja", example_schema=example_schema, base_schema=base_schema
        )

        return super().generate()
