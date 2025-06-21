"""Generate the templates.md file."""

import argparse

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .template_example_schema_json import TemplateExampleSchemaJson


class TemplatesMd(DocGenerator):
    """Generate the templates.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise Spec.md generator."""
        super().__init__(args, spec, "templates.md")

    def generate(self) -> bool:
        """Generate a `templates.md` file from the definitions."""
        # Generate the example_template_schema.json file.
        example_schema = TemplateExampleSchemaJson(self._args, self._spec)
        if not example_schema.save_or_validate():
            return False

        self._filedata = """
# Templates"
"""

        return super().generate()
