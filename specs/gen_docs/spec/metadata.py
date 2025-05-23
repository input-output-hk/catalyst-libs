"""Metadata Field Specification."""

from functools import cached_property

from pydantic import BaseModel, ConfigDict, Field, computed_field

from spec.optional import OptionalField


class Metadata(BaseModel):
    """Metadata Spec Data Definition."""

    description: str
    raw_exclusive: list[str] | None = Field(alias="exclusive")
    format: str
    required: OptionalField
    validation: str | None
    raw_type: str | list[str] | None = Field(alias="type", default=None)
    multiple: bool = Field(default=False)
    raw_linked_refs: list[str] | None = Field(alias="linked_refs", default=None)

    # Not deserialized, must be supplied.
    name: str | None = Field(default=None)  # Comes from `new`.
    doc_name: str | None = Field(default=None)

    model_config = ConfigDict(extra="forbid")

    def set_name(self, name: str, doc_name: str | None = None) -> None:
        """Set the name properties."""
        self.name = name
        self.doc_name = doc_name

    @staticmethod
    def fix_list(fix: str | list[str] | None) -> list[str]:
        """Fix up the named field, so it only has a list."""
        if fix is None:
            fix = []
        if isinstance(fix, str):
            fix = [fix]
        return fix

    @computed_field()
    @cached_property
    def exclusive(self) -> list[str]:
        """Exclusive."""
        return self.fix_list(self.raw_exclusive)

    @computed_field()
    @cached_property
    def type(self) -> list[str]:
        """Type."""
        return self.fix_list(self.raw_type)

    @computed_field()
    @cached_property
    def linked_refs(self) -> list[str]:
        """Linked Refs."""
        return self.fix_list(self.raw_linked_refs)

    @staticmethod
    def doc_ref_link(name: str, depth: int = 0, *, html: bool = False) -> str:
        """Metadata Document Reference link."""
        link = name.lower().replace(" ", "_")

        if html:
            link += "/"
        else:
            link += ".md"

        if depth == 0:
            link = f"./docs/{link}"
        else:
            maxdepth = 0 if html else 1
            while depth > maxdepth:
                link = f"../{link}"
                depth -= 1

        if html:
            return link
        return f"[{name}]({link})"

    @staticmethod
    def format_link(name: str, depth: int = 0) -> str:
        """Metadata Format link."""
        link = f"metadata.md#{name.lower().replace(' ', '-')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        return f"[{name}]({link})"

    @staticmethod
    def field_link(name: str, depth: int = 0) -> str:
        """Metadata Field link."""
        link = f"metadata.md#{name.lower().replace('`', '')}"

        while depth > 0:
            link = f"../{link}"
            depth -= 1

        return f"[`{name}`]({link})"

    def get_validation(self) -> str:
        """Get the Validation documentation (enhanced from the data itself)."""
        # Adds text to the validation description, so get it here.
        validation = self.validation

        # Add notes about exclusive field relationships.
        if len(self.exclusive) > 0:
            exclusive_def = f"\n`{self.exclusive[0]}`"
            if len(self.exclusive) > 1:
                if len(self.exclusive) > 2:  # noqa: PLR2004
                    for exclude in self.exclusive[1:-1]:
                        # We break the line so it doesn't get too long.
                        exclusive_def += f"\n, `{exclude}`"

                exclusive_def += f"\nand `{self.exclusive[-1]}`"
            validation += f"\n* MUST NOT be present in any document that contains {exclusive_def} metadata."

        for ref in self.linked_refs:
            validation += f"""
* The Document referenced by `{ref}`
  * MUST contain `{self.name}` metadata; AND
  * MUST match the referencing documents `{self.name}` value."""

        return validation.strip()

    def is_excluded(self) -> bool:
        """Is this metadata excluded from the specs definition. (must not be present)."""
        return self.required == "excluded"

    def metadata_as_markdown(self, *, doc_types: list[str] | None = None) -> str:
        """Generate Markdown of Metadata fields for the default set, or a specific document."""
        field_title_level = "###"

        field_display = f"""
{field_title_level} `{self.name}`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | {self.required.value} |
"""
        if not self.is_excluded():
            field_display += f"| Format | `{self.format}` |\n"

            if self.name == "type" and doc_types is not None:
                # Display the actual documents type values
                monospace_types = [f"`{doc_type}`" for doc_type in doc_types]
                field_display += f"| Type | {',<br/>'.join(monospace_types)} |\n"

            if self.multiple:
                field_display += f"| Multiple References | {self.multiple} |\n"

            ref_heading = "Valid References"
            for ref_doc in self.type:
                field_display += f"| {ref_heading} | `{ref_doc}` |\n"
                ref_heading = ""

            ref_heading = "Linked Reference Metadata"
            for ref_field in self.linked_refs:
                field_display += f"| {ref_heading} | [`{ref_field}`](#{ref_field}) |\n"
                ref_heading = ""

            exclusive_title = "Exclusive"
            for ref in self.exclusive:
                field_display += f"| {exclusive_title} | `{ref}` |\n"
                exclusive_title = ""

            field_display += f"""<!-- markdownlint-enable MD033 -->
{self.description}

{field_title_level}# `{self.name}` Validation

{self.get_validation()}
"""
        return field_display
