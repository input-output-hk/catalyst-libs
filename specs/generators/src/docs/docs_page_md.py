"""Generate the individual pages docs/<doc_name>.md file."""

import argparse
import typing

from pydantic import HttpUrl

from spec.payload import DRAFT7_SCHEMA, DRAFT202012_SCHEMA
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
        super().__init__(
            args, spec, doc_name=doc_name, template="document_page.md.jinja", flags=self.HAS_MARKDOWN_LINKS
        )

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

        header_docs = ""
        for header in headers.names:
            value = headers.get(header).value
            if value is None:
                continue
            if isinstance(value, list):
                value = f"[{','.join(value)}]"
            link = f"../spec.md#{header.replace(' ', '-')}"
            header_docs += f"* [{header}]({link}) = `{value}`\n"
        if not header_docs:
            return "No Headers are defined for this document."

        return header_docs.strip()

    def document_payload_json(self) -> str:
        """Generate Payload Documentation - JSON."""
        docs = ""
        schema = self._doc.payload.doc_schema
        if schema is not None:
            if isinstance(schema, HttpUrl):
                if schema == DRAFT7_SCHEMA:
                    docs += "\n**Must be a valid JSON Schema Draft 7 document.**"
                if schema == DRAFT202012_SCHEMA:
                    docs += "\n**Must be a valid JSON Schema Draft 2020-12 document.**"
                else:
                    docs += f"\nMust be a valid according to <{schema}>."
                return docs

            docs += f"""\n### Schema

{self.json_example(schema, label="Schema", title="Payload JSON Schema", description=docs.strip(), icon_type="abstract")}
"""
        if len(self._doc.payload.examples) > 0:
            docs += "\n### Example\n" if len(self._doc.payload.examples) < 2 else "\n### Examples\n"  # noqa: PLR2004
            for example in self._doc.payload.examples:
                docs += f"{example}\n"

        return docs.strip()

    def document_payload_cbor(self) -> str:
        """Generate Payload Documentation - CBOR."""
        docs = "CBOR Payload Documentation\n"

        return docs.strip()

    def document_payload(self) -> str:
        """Generate Payload Documentation."""
        if self._doc.draft and self._doc.payload.description == "":
            return self.TODO_MSG

        docs = self._doc.payload.description + "\n"

        if self._doc.payload.nil:
            if self._doc.payload.doc_schema is None:
                docs += """
This document has no payload.
It must be encoded as a CBOR `null (0xf6)`.
"""
                return docs.strip()
            docs += """
This document *MAY* have no payload.
In this case, it *MUST* be encoded as a CBOR `null (0xf6)`.
"""

        schema = self._doc.payload.doc_schema
        if schema is not None and isinstance(schema, str):
            docs += self.document_payload_cbor()
        else:
            docs += self.document_payload_json()

        return docs.strip()

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

        signers_doc += f"\n\n{signers.update.description}\n\n"

        return signers_doc.strip()

    def generate(self) -> bool:
        """Generate the individual documents File."""
        # Generate the relationship diagram.

        graph = DocRelationshipFile(self._args, self._spec, self._document_name, depth=self._depth)
        if not graph.save_or_validate():
            return False

        self.generate_from_page_template(graph=graph)

        return super().generate()
