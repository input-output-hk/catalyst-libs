"""Metadata Field Specification."""

import textwrap
import typing
from functools import cached_property

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, RootModel, computed_field

from spec.base_types import DocTypeId
from spec.cddl.cose import GenericHeader
from spec.cddl.definition import CDDLDefinition
from spec.metadata_formats import MetadataFormats
from spec.optional import OptionalField


class MetadataHeader(GenericHeader):
    """Metadata Spec Data Definition."""

    validation: str | None
    raw_type: str | list[str] | None = Field(alias="type", default=None)
    multiple: bool = Field(default=False)
    raw_linked_refs: list[str] | None = Field(alias="linked_refs", default=None)

    # Not deserialized, must be supplied.
    _name: str = PrivateAttr(default="Unknown")
    _doc_name: str | None = PrivateAttr(default=None)

    model_config = ConfigDict(extra="forbid")

    def set_name(self, name: str, doc_name: str | None = None) -> None:
        """Set the name properties."""
        self._name = name
        self._doc_name = doc_name

    @staticmethod
    def fix_list(fix: str | list[str] | None) -> list[str]:
        """Fix up the named field, so it only has a list."""
        if fix is None:
            fix = []
        if isinstance(fix, str):
            fix = [fix]
        return fix

    @computed_field
    @cached_property
    def type(self) -> list[str]:
        """Type."""
        return self.fix_list(self.raw_type)

    @computed_field
    @cached_property
    def linked_refs(self) -> list[str]:
        """Linked Refs."""
        return self.fix_list(self.raw_linked_refs)

    def get_validation(self) -> str:
        """Get the Validation documentation (enhanced from the data itself)."""
        # Adds text to the validation description, so get it here.
        validation = self.validation if self.validation is not None else ""

        for ref in self.linked_refs:
            validation += f"""

* The Document referenced by `{ref}`
  * MUST contain `{self._name}` metadata; AND
  * MUST match the referencing documents `{self._name}` value."""

        return validation.strip()

    def is_excluded(self) -> bool:
        """Is this metadata excluded from the specs definition. (must not be present)."""
        return self.required == "excluded"

    def metadata_as_markdown(self, *, doc_type: DocTypeId | None = None) -> str:
        """Generate Markdown of Metadata fields for the default set, or a specific document."""
        field_title_level = "###"

        field_display = f"""
{field_title_level} `{self._name}`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | {self.required.value} |
"""
        if not self.is_excluded():
            field_display += f"| Format | `{self.format}` |\n"

            if self._name == "type" and doc_type is not None:
                # Display the actual documents type values
                field_display += f"| Type | {doc_type.as_uuid_str} |\n"

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

            field_display += f"""<!-- markdownlint-enable MD033 -->
{self.description}

{field_title_level}# `{self._name}` Validation

{self.get_validation()}
"""
        return field_display


class MetadataHeaders(RootModel[dict[str, MetadataHeader]]):
    """All Metadata Headers."""

    root: dict[str, MetadataHeader]

    _order: list[str] | None = PrivateAttr(default=None)
    _doc_name: str | None = PrivateAttr(default=None)

    def get(self, name: str) -> MetadataHeader:
        """Get a Metadata Header by its name."""
        return self.root[name]

    @computed_field
    @property
    def names(self) -> list[str]:
        """Get ordered list of all defined Metadata Header Names."""
        if self._order is not None:
            return self._order
        return list(self.root.keys())

    @computed_field
    @property
    def all(self) -> typing.Sequence[MetadataHeader]:
        """Get all metadata headers, in order."""
        return [self.root[header] for header in self.names]

    def order(self, order: list[str]) -> None:
        """Set the order of fields."""
        tmp_order = order
        # Add in any unlisted headers
        for name in self.root:
            if name not in tmp_order:
                tmp_order.append(name)
        self._order = []
        # remove any listed headers that are not used.
        for name in tmp_order:
            if name in self.root:
                self._order.append(name)

    def set_name(self, doc_name: str | None = None) -> None:
        """Set the name properties."""
        self._doc_name = doc_name
        for name, header in self.root.items():
            header.set_name(name, doc_name)


class Metadata(BaseModel):
    """Metadata Fields in the COSE Document."""

    headers: MetadataHeaders
    formats: MetadataFormats
    order: list[str]

    _doc_name: str | None = PrivateAttr(default=None)

    model_config = ConfigDict(extra="forbid")

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        # Set Header Order
        self.headers.order(self.order)

    def set_name(self, doc_name: str | None = None) -> None:
        """Set the name properties."""
        self._doc_name = doc_name
        self.headers.set_name(doc_name)

    @staticmethod
    def custom_metadata_header(
        cddl_def: CDDLDefinition, headers: typing.Sequence[GenericHeader], formats: MetadataFormats
    ) -> CDDLDefinition:
        """Generate a synthetic cddl def for this type.

        Needs to be generated from Metadata definitions.
        """
        new_def = cddl_def.model_copy()
        new_def.requires = []
        new_cddl: str = ""

        for header in headers:
            optional = "" if header.required == OptionalField.required else "?"
            cddl_type = formats.get(header.format).cddl
            new_cddl += f"{optional}{header.label} => {cddl_type}\n"
            if cddl_type not in new_def.requires:
                new_def.requires.append(cddl_type)

        new_def.definition = f"(\n{textwrap.indent(new_cddl, '  ')})"
        return new_def
