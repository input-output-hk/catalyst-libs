"""Generate the template_example.schema.json file."""

import argparse
import json
from typing import Any

import jsonschema

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
        schema: dict[str, Any] = {
            "$schema": "https://json-schema.org/draft/2020-12/schema#",
            "title": "Example Template Schema",
            "description": "Example Template Schema showing all defined field types.",
            "maintainers": [{"name": "Catalyst Team", "url": "https://projectcatalyst.io/"}],
            "$defs": self._spec.form_template.json_definition,
            "type": "object",
            "additionalProperties": False,
            "properties": self._spec.form_template.example,
        }

        template_schema = json.dumps(schema, indent=4)
        jsonschema.Draft202012Validator.check_schema(schema)

        self._filedata = template_schema

        return super().generate()
