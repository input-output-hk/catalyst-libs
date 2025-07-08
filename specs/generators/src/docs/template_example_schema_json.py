"""Generate the template_example.schema.json file."""

import argparse
import json

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class TemplateExampleSchemaJson(DocGenerator):
    """Generate the template_example.schema.json file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise template_example.schema.json generator."""
        file_name = "schema/template_example.schema.json"

        super().__init__(args, spec, file_name)

    def generate(self) -> bool:
        """Generate a `template_example.schema.json` file from the definitions."""
        schema = self._spec.form_template.generic_schema.example(properties=self._spec.form_template.elements.example)
        template_schema = json.dumps(schema, indent=4)

        self._filedata = template_schema

        return super().generate()
