"""Generate the individual pages docs/<doc_name>.md file."""

import argparse
import typing

from spec.signed_doc import SignedDoc

from .doc_generator import DocGenerator
from .doc_relationship_diagrams import DocRelationshipFile


class IndividualDocMd(DocGenerator):
    """Generate the individual pages docs/<doc_name>.md file."""

    TODO_MSG: typing.ClassVar[str] = """
This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.
""".strip()

    def __init__(self, args: argparse.Namespace, spec: SignedDoc, doc_name: str) -> None:
        """Generate the individual pages docs/<doc_name>.md file."""
        file_name = "docs/" + doc_name.lower().replace(" ", "_") + ".md"
        super().__init__(args, spec, filename=file_name, flags=self.HAS_MARKDOWN_LINKS)

        self._document_name = doc_name
        self._doc = self._spec.docs.get(doc_name)
        self._depth = 1

    @classmethod
    def save_or_validate_all(cls, args: argparse.Namespace, spec: SignedDoc) -> bool:
        """Save or Validate all documentation pages."""
        good = True
        for doc_name in spec.docs.names:
            good &= cls(args, spec, doc_name).save_or_validate()

        return good

    def description_or_todo(self, desc: str | None) -> str:
        """Return a description of TODO text."""
        if desc is None:
            desc = self.TODO_MSG
        return desc

    def header_parameter_summary(self) -> str:
        """Generate concrete Cose header parameter settings for a specific document."""
        headers = self._doc.headers

        if headers is None:
            return "No Headers are defined for this document."

        header_docs = ""
        for header in headers.names:
            value = headers.get(header).value
            if isinstance(value, list):
                value = f"[{','.join(value)}]"
            link = f"../spec.md#{header.replace(' ', '-')}"
            header_docs += f"* [{header}]({link}) = `{value}`\n"
        return header_docs.strip()

    def document_payload(self) -> str:
        """Generate Payload Documentation."""
        if self._doc.payload is None:
            return self.TODO_MSG

        return f"{self._doc.payload}"

    def document_signers(self) -> str:
        """Generate documentation about who may sign this documents."""
        signers = self._spec.docs.get(self._document_name).signers
        signers_doc: str = ""

        def add_role_group(name: str, roles: list[str]) -> None:
            nonlocal signers_doc
            if len(roles) > 0:
                signers_doc += f"\nThe following {name} roles may sign documents of this type:\n\n"
                for role in roles:
                    signers_doc += f"* {role}\n"

        add_role_group("User", signers.roles.user)
        add_role_group("Admin", signers.roles.admin)

        signers_doc = signers_doc.strip()

        signers_doc += "\n\nNew versions of this document may be published by:\n\n"
        for updater in signers.update:
            if signers.update[updater]:
                signers_doc += f"* {updater}\n"

        return signers_doc.strip()

    def generate(self) -> bool:
        """Generate the individual documents File."""
        # Generate the relationship diagram.

        graph = DocRelationshipFile(self._args, self._spec, self._document_name, depth=self._depth)
        if not graph.save_or_validate():
            return False

        self._filedata = f"""
# {self._document_name}

## Description

{self.description_or_todo(self._doc.description)}

{graph.markdown_reference(relative_doc=self, filetype="svg")}

### Validation

{self.description_or_todo(self._doc.validation)}

### Business Logic

#### Front End

{self.description_or_todo(self._doc.business_logic.front_end)}

#### Back End

{self.description_or_todo(self._doc.business_logic.back_end)}

## COSE Header Parameters

{self.header_parameter_summary()}

## Metadata

{self._spec.get_metadata_as_markdown(self._document_name)}

## Payload

{self.document_payload()}

## Signers

{self.document_signers()}

{self.insert_copyright()}
"""
        return super().generate()
