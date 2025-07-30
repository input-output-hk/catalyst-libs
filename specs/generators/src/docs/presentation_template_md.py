"""Generate the form_templates.md file."""

import argparse
from functools import cached_property

import polars as pl
from great_tables import GT
from pydantic import computed_field

from docs.presentation_template_schema_json import PresentationTemplateSchemaJson
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator


class PresentationTemplatesMd(DocGenerator):
    """Generate the form_templates.md file."""

    TEMPLATE: str = "presentation_template.md.jinja"

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise presentation_template.md generator."""
        super().__init__(args, spec, template=self.TEMPLATE)

    @computed_field
    @cached_property
    def all_cards(self) -> str:
        """Generate a Reference table for all defined Presentation Cards."""
        table_data: dict[str, list[str]] = {"Id": [], "Name": [], "Description": [], "Available Documents": []}

        for card_id, card in self._spec.presentation_template.cards.all():
            table_data["Id"].append(card_id)
            table_data["Name"].append(card.name)
            table_data["Description"].append(card.description)
            table_data["Available Documents"].append(f"{card.available_docs}")

        params = pl.DataFrame(table_data)

        table = (
            GT(params)
            .with_id(id=f"icon {self.name()}".replace(" ", "_"))
            .tab_header(
                title="Defined Presentation Cards",
                subtitle="\n\nAll Presentation Card Names that may be defined by Presentation Templates.\n\n",
            )
            .tab_stub(groupname_col="Name")
            .opt_stylize(style=5)
        )

        return f"{self.wrap_html(table.as_raw_html())}".strip() + "\n"

    def generate(self) -> bool:
        """Generate a `form_templates.md` file from the definitions."""
        # Generate the example_template_schema.json file.
        schema = PresentationTemplateSchemaJson(self._args, self._spec)
        schema_ok = schema.save_or_validate()

        self.generate_from_page_template(schema=schema)

        return super().generate() and schema_ok
