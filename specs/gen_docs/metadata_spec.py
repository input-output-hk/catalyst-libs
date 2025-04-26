"""Metadata Field Specification."""

from dataclasses import dataclass


@dataclass(kw_only=True)
class Metadata:
    """Metadata Data Definition."""

    name: str  # Comes from init.

    # Comes from the json data in the spec.
    description: str
    exclusive: list[str] | None
    format: str
    required: str
    validation: str
    # Only if its a Document Reference metadata
    multiple: bool | None = None
    type: str | list[str] | None = None
    linked_refs: list[str] | None = None

    def get_exclusive(self) -> list[str]:
        """Get the exclusive field, cleaned up to detect empty lists in the spec."""
        if self.exclusive is None:
            return []
        return self.exclusive

    def get_linked_refs(self) -> list[str]:
        """Get the linked_ref, cleaned up to detect empty lists in the spec."""
        if self.linked_refs is None:
            return []
        return self.linked_refs

    def get_ref_types(self) -> list[str]:
        """Get the types this references, if any."""
        clean_type = self.type
        if clean_type is None:
            return []
        if isinstance(clean_type, str):
            clean_type = [clean_type]
        return clean_type

    def get_validation(self) -> str:
        """Get the Validation documentation (enhanced from the data itself)."""
        # Adds text to the validation description, so get it here.
        validation = self.validation

        # Add notes about exclusive field relationships.
        exclusive = self.get_exclusive()
        if exclusive != []:
            exclusive_def = f"\n`{exclusive[0]}`"
            if len(exclusive) > 1:
                if len(exclusive) > 2:  # noqa: PLR2004
                    for exclude in exclusive[:-1]:
                        # We break the line so it doesn't get too long.
                        exclusive_def += f"\n, `{exclude}"

                exclusive_def += f"\nand `{exclusive[-1]}`"
            validation += f"\n* MUST NOT be present in any document that contains {exclusive} metadata."

        for ref in self.get_linked_refs():
            validation += f"""
* The Document referenced by `{ref}`
  * MUST contain `{self.name}` metadata; AND
  * MUST match the referencing documents `{self.name}` value."""

        return validation.strip()

    def is_excluded(self) -> bool:
        """Is this metadata excluded from the specs definition. (must not be present)."""
        return self.required == "excluded"

    def metadata_as_markdown(self, include_types: bool = True) -> str:
        """Generate Markdown of Metadata fields for the default set, or a specific document."""
        field_title_level = "###"

        field_display = f"""
{field_title_level} `{self.name}`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | {self.required} |
"""
        if not self.is_excluded():
            field_display += f"| Format | `{self.format}` |\n"

            if include_types and len(self.get_ref_types()) > 0:
                # Display the actual documents type values
                monospace_types = [f"`{doc_type}`" for doc_type in self.get_ref_types()]
                field_display += f"| Type | {',<br/>'.join(monospace_types)} |\n"

            if self.multiple:
                field_display += f"| Multiple References | {self.multiple} |\n"

            ref_heading = "Valid References"
            for ref_doc in self.get_ref_types():
                field_display += f"| {ref_heading} | `{ref_doc}` |\n"
                ref_heading = ""

            ref_heading = "Linked Reference Metadata"
            for ref_field in self.get_linked_refs():
                field_display += f"| {ref_heading} | [`{ref_field}`](#{ref_field}) |\n"
                ref_heading = ""

            exclusive_title = "Exclusive"
            for ref in self.get_exclusive():
                field_display += f"| {exclusive_title} | `{ref}` |\n"
                exclusive_title = ""

            field_display += f"""<!-- markdownlint-enable MD033 -->
{self.description}

{field_title_level}# `{self.name}` Validation

{self.get_validation()}
"""
        return field_display
