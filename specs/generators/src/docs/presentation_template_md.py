"""Generate the form_templates.md file."""

import argparse
from functools import cached_property

import polars as pl
from great_tables import GT
from pydantic import computed_field

from docs.markdown import MarkdownHelpers
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
        table_data: dict[str, list[str]] = {"Name": [], "Field": [], "Value": []}

        def add_table_data(name: str, field: str, value: str) -> None:
            """Add another entry to the table."""
            table_data["Name"].append(MarkdownHelpers.to_html(name))
            table_data["Field"].append(field)
            table_data["Value"].append(value)

        for card_id, card in self._spec.presentation_template.cards.all():
            name = f"""
### {card.name}

{card.description}
""".strip()
            add_table_data(name, "Card ID", f"`{card_id}`")
            doc_links = [
                f'<a href=".{MarkdownHelpers.doc_ref_link(doc, html=True)}">{doc}</a>' for doc in card.available_docs
            ]
            add_table_data(name, "Available Documents", ("* " + "\n* ".join(doc_links)).strip())

        params = pl.DataFrame(table_data)

        table = (
            GT(params)
            .with_id(id=f"cards {self.name()}".replace(" ", "_"))
            .tab_header(
                title="Defined Presentation Cards",
                subtitle="\n\nAll Presentation Card Names that may be defined by Presentation Templates.\n\n",
            )
            .fmt_markdown(["Name", "Value"])
            .tab_stub(rowname_col="Field", groupname_col="Name")
            .cols_label({"Value": ""})
            .opt_stylize(style=6)
        )

        return f"{self.wrap_html(table.as_raw_html())}".strip() + "\n"

    def generate(self) -> bool:
        """Generate a `form_templates.md` file from the definitions."""
        # Generate the example_template_schema.json file.
        schema = PresentationTemplateSchemaJson(self._args, self._spec)
        schema_ok = schema.save_or_validate()

        self.generate_from_page_template(schema=schema)

        return super().generate() and schema_ok
