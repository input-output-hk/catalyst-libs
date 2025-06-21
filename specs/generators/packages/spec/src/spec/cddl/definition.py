"""CDDL Definition Specification."""

import re
import typing

from pydantic import BaseModel, ConfigDict, Field, PrivateAttr, RootModel


class CDDLDefinition(BaseModel):
    """CDDL Definition Deserialized Specification."""

    definition: str = Field(alias="def")
    requires: list[str]
    description: str | None = Field(default=None)
    comment: str = Field(default_factory=str)

    _name: str = PrivateAttr(default="Unknown")

    model_config = ConfigDict(extra="forbid")

    def name(self) -> str:
        """Name Of the Parameter."""
        return self._name

    def set_name(self, val: str) -> None:
        """Set Name."""
        self._name = val


class CDDLDefinitions(RootModel[dict[str, CDDLDefinition]]):
    """Template Json Schema Definitions."""

    root: dict[str, CDDLDefinition]

    def model_post_init(self, context: typing.Any) -> None:  # noqa: ANN401
        """Extra setup after we deserialize."""
        super().model_post_init(context)

        for def_name, value in self.root.items():
            value.set_name(def_name)

    def get(self, name: str) -> CDDLDefinition:
        """Get a CDDL Definition."""
        return self.root[name]

    def add(self, definition: CDDLDefinition | list[CDDLDefinition]) -> None:
        """Add (or change) a definitions to the CDDL Definitions."""
        if isinstance(definition, CDDLDefinition):
            definition = [definition]
        for this_def in definition:
            self.root[this_def.name()] = this_def

    @staticmethod
    def _add_cddl_comments(comment: str) -> tuple[str, bool]:
        """Add cddl comment markers to lines.

        Returns True if more than 1 line.
        """
        comment = comment.strip()
        comment_lines = comment.splitlines()
        comment = ""
        for line in comment_lines:
            comment += f"; {line}\n"
        comment = comment.strip()

        return comment, len(comment_lines) > 0

    def _nested_cddl(self, name: str, found: list[str]) -> tuple[str, list[str]]:
        """Get the CDDL for a names definition, recursively."""
        this_cddl = ""
        this_def = self.get(name)
        cddl_def = this_def.definition.strip()
        cddl_def_multiline = len(cddl_def.splitlines()) > 1

        # Add required definitions to this one (recursive)
        for requires in this_def.requires:
            if requires not in found:
                next_cddl, found = self._nested_cddl(requires, found)
                found.append(requires)
                this_cddl += next_cddl

        comment: str = this_def.comment
        leading_comment = ""
        if len(comment) > 0:
            comment, multiline = self._add_cddl_comments(comment)
            if multiline or cddl_def_multiline:
                leading_comment = comment
                comment = "\n"  # Adds a blank line after defs with multiline comments

        this_cddl = f"""
{leading_comment}
{name} = {cddl_def} {comment}

{this_cddl}
"""

        return this_cddl, found

    def cddl_file(self, root: str) -> str:
        """Get the CDDL File for a root definition with a given name."""
        cddl_data = self._nested_cddl(root, [])[0]
        description = self.get(root).description
        if description is None:
            description = root
        description = self._add_cddl_comments(description)[0]

        # Remove double line breaks,
        # so we only ever have 1 between definitions
        cddl_data = re.sub(r"\n\n[\n]+", "\n\n", cddl_data)

        return f"""
{description}


{cddl_data.strip()}
"""
