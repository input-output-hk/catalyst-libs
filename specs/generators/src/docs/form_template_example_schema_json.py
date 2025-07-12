"""Generate the template_example.schema.json file."""

import argparse
import json

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplateExampleSchemaJson(DocGenerator):
    """Generate the template_example.schema.json file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise template_example.schema.json generator."""
        file_name = "schema/form_template_example.schema.json"

        super().__init__(args, spec, filename=file_name)

    def markdown_reference(
        self,
        *,
        indent: int = 0,
        relative_doc: DocGenerator | None = None,
        title: str = "Form Template Example Schema",
        filetype: str = "json",
    ) -> str:
        """Create a Markdown formatted reference for the file."""
        return super().markdown_reference(indent=indent, relative_doc=relative_doc, title=title, filetype=filetype)

    def generate(self) -> bool:
        """Generate a `template_example.schema.json` file from the definitions."""
        schema = self._spec.form_template.generic_schema.example(properties=self._spec.form_template.elements.example)
        template_schema = json.dumps(schema, indent=4)

        self._filedata = template_schema

        return super().generate()
