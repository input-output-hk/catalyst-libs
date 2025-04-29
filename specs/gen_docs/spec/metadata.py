"""Metadata Field Specification."""

from dataclasses import dataclass
from functools import cached_property
from typing import Any, Self

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
    def doc_ref_link(name: str, depth: int = 0) -> str:
        """Metadata Document Reference link."""
        link = name.lower().replace(" ", "_") + ".md"

        if depth == 0:
            link = f"./docs/{link}"
        else:
            while depth > 1:
                link = f"../{link}"
                depth -= 1

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


@dataclass(kw_only=True, frozen=True)
class MetadataOld:
    """Metadata Spec Data Definition."""

    name: str  # Comes from `new`.
    doc_name: str | None  # Comes from `new`

    # Comes from the json data in the spec.
    description: str
    exclusive: list[str] | None
    format: str
    required: str
    validation: str
    # Only if its a Document Reference metadata
    multiple: bool
    type: list[str]
    linked_refs: list[str]

    @staticmethod
    def fix_list(src: dict[str, Any], field: str) -> None:
        """Fix up the named field, so it only has a list."""
        fix = src.get(field, [])
        if fix is None:
            fix = []
        if isinstance(fix, str):
            fix = [fix]
        src[field] = fix

    @classmethod
    def new(
        cls, raw_metadata: dict[str, Any], metadata_name: str, doc_name: str | None
    ) -> Self:  # noqa: ANN401
        """Call this instead of __init__ directly."""
        # Clean up raw data.
        multiple = raw_metadata.get("multiple", False)
        raw_metadata["multiple"] = False if multiple is None else multiple
        cls.fix_list(raw_metadata, "exclusive")
        cls.fix_list(raw_metadata, "type")
        cls.fix_list(raw_metadata, "linked_refs")

        return cls(name=metadata_name, doc_name=doc_name, **raw_metadata)

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

    def metadata_as_markdown(self, *, include_types: bool = True) -> str:
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

            if include_types and len(self.type) > 0:
                # Display the actual documents type values
                monospace_types = [f"`{doc_type}`" for doc_type in self.type]
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

    @staticmethod
    def doc_ref_link(name: str, depth: int = 0) -> str:
        """Metadata Document Reference link."""
        link = name.lower().replace(" ", "_") + ".md"

        if depth == 0:
            link = f"./docs/{link}"
        else:
            while depth > 1:
                link = f"../{link}"
                depth -= 1

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
