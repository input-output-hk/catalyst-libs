"""Generate the form_templates.md file."""

import argparse
from functools import cached_property

import polars as pl
from great_tables import GT, md
from pydantic import computed_field

from docs.form_template_basic_schema_json import FormTemplateBasicSchemaJson
from docs.form_template_example_schema_json import FormTemplateExampleSchemaJson
from docs.form_templates_element_index import FormTemplatesElementIndex
from docs.form_templates_element_md import FormTemplatesElementMd
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class FormTemplatesMd(DocGenerator):
    """Generate the form_templates.md file."""

    TEMPLATE: str = FormTemplatesElementMd.FORM_TEMPLATE
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

    @computed_field
    @cached_property
    def all_icons(self) -> str:
        """Generate a Reference table for all defined Icon Assets."""
        table_data: dict[str, list[str]] = {"Name": [], "Icon Image": []}

        for icon in self._spec.form_template.assets.icons.all:
            svg = self._spec.form_template.assets.icons.svg(icon)

            table_data["Name"].append(icon)
            table_data["Icon Image"].append(svg)

        params = pl.DataFrame(table_data)

        table = (
            GT(params)
            .with_id(id=f"icon {self.name()}".replace(" ", "_"))
            .tab_header(
                title="Defined Icons", subtitle="\n\nAll icon Names that may be referenced by Form Elements.\n\n"
            )
            .tab_source_note(md("*Icon images are representative and may be customized by implementations.*"))
            .cols_align("center", "Icon Image")
            .opt_stylize(style=5)
        )

        return f"{self.wrap_html(table.as_raw_html())}".strip() + "\n"

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
