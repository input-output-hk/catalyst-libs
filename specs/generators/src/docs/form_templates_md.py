"""Generate the form_templates.md file."""

import argparse

from docs.form_template_basic_schema_json import FormTemplateBasicSchemaJson
from docs.form_template_example_schema_json import FormTemplateExampleSchemaJson
from docs.form_templates_element_index import FormTemplatesElementIndex
from docs.form_templates_element_md import FormTemplatesElementMd
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplatesMd(DocGenerator):
    """Generate the form_templates.md file."""

    TEMPLATE: str = "form_templates.md.jinja"
    ELEMENT_TEMPLATE: str = FormTemplatesElementMd.TEMPLATE

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise form_templates.md generator."""
        super().__init__(args, spec, template=self.TEMPLATE)

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

        pages_ok = FormTemplatesElementIndex(self._args, self._spec).save_or_validate()

        self.generate_from_page_template(example_schema=example_schema, base_schema=base_schema)

        return pages_ok and super().generate()
