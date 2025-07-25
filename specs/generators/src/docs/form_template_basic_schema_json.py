"""Generate the template_example.schema.json file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplateBasicSchemaJson(DocGenerator):
    """Generate the form_template.schema.json file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise template_example.schema.json generator."""
        file_name = "schema/form_template.schema.json"

        super().__init__(args, spec, filename=file_name)

    def markdown_reference(
        self,
        *,
        indent: int = 0,
        relative_doc: DocGenerator | None = None,
        title: str = "Form Template Base Schema",
        filetype: str = "json",
    ) -> str:
        """Create a Markdown formatted reference for the file."""
        return super().markdown_reference(indent=indent, relative_doc=relative_doc, title=title, filetype=filetype)

    def generate(self) -> bool:
        """Generate a `form_template.schema.json` file from the definitions."""
        schema = self._spec.form_template.generic_schema.basic()
        template_schema = self.json_schema_sort(schema)

        self._filedata = template_schema

        return super().generate()
