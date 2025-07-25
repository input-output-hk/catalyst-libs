"""Generate the template_example.schema.json file."""

import argparse

import jsonschema
import rich
import rich.pretty

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
        schema = self._spec.form_template.elements.example()

        # Just ensure the generated example is valid.
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
        except Exception:
            try:
                rich.print_json(data=schema, indent=4)
            except Exception:
                rich.pretty.pprint(schema)
                raise
            raise

        template_schema = self.json_schema_sort(schema)

        self._filedata = template_schema

        return super().generate()
