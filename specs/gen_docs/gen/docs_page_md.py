"""Generate the individual pages docs/<doc_name>.md file."""

import argparse
import json
import typing

from pydantic import HttpUrl

from gen.doc_generator import DocGenerator
from gen.doc_relationship_diagrams import DocRelationshipFile
from spec.signed_doc import SignedDoc


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
        super().__init__(args, spec, file_name, flags=self.HAS_MARKDOWN_LINKS)

        self._document_name = doc_name
        self._doc = self._spec.get_document(doc_name)
        self._depth = 1

    @classmethod
    def save_or_validate_all(cls, args: argparse.Namespace, spec: SignedDoc) -> bool:
        """Save or Validate all documentation pages."""
        good = True
        for doc_name in spec.document_names():
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
        for header, value in headers.items():
            value = value.value  # noqa: PLW2901
            if isinstance(value, list):
                value = f"[{','.join(value)}]"  # noqa: PLW2901
            link = f"../spec.md#{header.replace(' ', '-')}"
            header_docs += f"* [{header}]({link}) = `{value}`\n"
        return header_docs.strip()

    def document_payload(self) -> str:
        """Generate Payload Documentation."""
        if self._doc.payload is None:
            return self.TODO_MSG

        payload_docs = self._doc.payload.description + "\n"

        schema = self._doc.payload.doc_schema
        if schema is not None:
            if isinstance(schema, HttpUrl):
                if schema == "https://json-schema.org/draft-07/schema":
                    payload_docs += "\n**Must be a valid JSON Schema Draft 7 document.**"
                else:
                    payload_docs += f"\nMust be a valid according to <{schema}>."
            else:
                payload_docs += f"""\nSchema :
<!-- markdownlint-disable MD013 -->
```json
{json.dumps(schema, indent=2, sort_keys=True)}
```
<!-- markdownlint-enable MD013 -->
"""
        return payload_docs.strip()

    def document_signers(self) -> str:
        """Generate documentation about who may sign this documents."""
        signers = self._spec.data()["docs"][self._document_name]["signers"]
        signers_doc = ""

        for role_group in signers["roles"]:
            roles = signers["roles"][role_group]
            if roles:
                signers_doc += f"\nThe following {role_group} roles may sign documents of this type:\n\n"
                for role in roles:
                    signers_doc += f"* {role}\n"

        signers_doc = signers_doc.strip()

        signers_doc += "\n\nNew versions of this document may be published by:\n\n"
        for updater in signers["update"]:
            if signers["update"][updater]:
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

{graph.markdown_reference(relative_doc=self, extension="svg")}

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
