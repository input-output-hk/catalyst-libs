"""Generate the form_templates_element.md.jinja templated files."""

import argparse
import json
from functools import cached_property
from typing import Any

import polars as pl
from great_tables import GT
from pydantic import computed_field

from docs.form_template_basic_schema_json import FormTemplateBasicSchemaJson
from docs.form_template_example_schema_json import FormTemplateExampleSchemaJson
from docs.markdown import MarkdownHelpers
from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator, LinkType


class FormTemplatesElementMd(DocGenerator):
    """Generate the Element documentation for a form template."""

    TEMPLATE: str = "form_templates_element.md.jinja"
    FORM_TEMPLATE: str = "form_templates.md.jinja"

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, name: str) -> None:
        """Initialise form templates Element documentation generator."""
        self._element = spec.form_template.elements.get(name)
        super().__init__(args, spec, doc_name=self._element.title_name, template=self.TEMPLATE)

    @computed_field
    @cached_property
    def example_definition(self) -> dict[str, Any]:
        """Example Json Definition."""
        return {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$defs": {self._element.name: self._element.definition},
        }

    def example(self) -> dict[str, Any]:
        """Generate an example of the element in a template."""
        return self._spec.form_template.elements.example(self._element.name)

    @computed_field
    @cached_property
    def parameters_table(self) -> str:  # noqa: C901, PLR0912
        """Definitions Parameters as an HTML Table."""
        table_data: dict[str, list[Any]] = {"Group": [], "Headings": [], "Values": []}  # , "Docs": []}

        def add_param_field(prop_name: str, heading: str, value: str = "") -> None:
            """Add a parameter field."""
            table_data["Group"].append(MarkdownHelpers.to_html(prop_name))
            table_data["Headings"].append(heading)
            table_data["Values"].append(value)

        for parameter in self._element.parameters.all:
            group = f"**`{parameter.element_name}`**<br>{parameter.description}"
            add_param_field(group, "Required", f"{parameter.required.value}")
            add_param_field(group, "Type", f"`{parameter.type}`")
            if parameter.items is not None:
                if parameter.items.type != "object":
                    add_param_field(group, "Items", f"`{parameter.items.type}`")
                else:
                    add_param_field(group, "Items", f"{parameter.items}")
            if parameter.choices is not None:
                if self._spec.form_template.assets.icons.check(parameter.choices):
                    choices = self.link_to_file(
                        "Icons", link_file="form_templates", heading="icons", link_type=LinkType.HTML
                    )
                else:
                    choices = "[" + ",<br>".join(f"`{choice}`" for choice in parameter.choices) + "]"
                add_param_field(group, "Choices", choices)
            if parameter.format is not None:
                add_param_field(group, "Format", parameter.format)
            if parameter.content_media_type is not None:
                add_param_field(group, "Content Media Type", parameter.content_media_type)
            if parameter.pattern is not None:
                add_param_field(group, "Pattern", parameter.pattern)
            if parameter.min_length is not None:
                add_param_field(group, "Minimum Length", f"{parameter.min_length}")
            if parameter.minimum is not None:
                add_param_field(group, "Minimum", f"{parameter.minimum}")
            if parameter.maximum is not None:
                add_param_field(group, "Minimum", f"{parameter.min_length}")
            if parameter.example is not None:
                example = json.dumps(parameter.example)
                add_param_field(group, "Example", f"`{parameter.element_name}: {example}`")

        params = pl.DataFrame(table_data)

        table = (
            GT(params)
            .with_id(id=f"element {self.name()} parameters".replace(" ", "_"))
            .tab_header(title=f"{self.name()}", subtitle="\n\nParameters\n\n")
            .fmt_markdown(["Group", "Values"])  # , "Docs"])
            .cols_width(cases={"Headings": "10%", "Values": "50%"})
            .tab_stub(rowname_col="Headings", groupname_col="Group")
            .tab_options(column_labels_hidden=True, container_width="100%", table_width="100%")
            .opt_stylize(style=6)
        )

        return f"{self.wrap_html(table.as_raw_html())}".strip()

    @classmethod
    def save_or_validate_all(cls, args: argparse.Namespace, spec: SignedDoc) -> bool:
        """Save or Validate all documentation pages."""
        good = True
        for doc_name in spec.form_template.elements.names():
            good &= cls(args, spec, doc_name).save_or_validate()

        return good

    def generate(self) -> bool:
        """Generate a `form_templates_element.md` type file from the definitions."""
        # Generate the example_template_schema.json file.
        example_schema = FormTemplateExampleSchemaJson(self._args, self._spec)
        if not example_schema.save_or_validate():
            return False

        base_schema = FormTemplateBasicSchemaJson(self._args, self._spec)
        if not base_schema.save_or_validate():
            return False

        self.generate_from_page_template(element=self._element)

        return super().generate()
