"""Generate the spec.md file."""

import argparse

from gen.cddl_file import CDDLFile
from gen.doc_generator import DocGenerator
from spec.signed_doc import HeaderType, SignedDoc


class SpecMd(DocGenerator):
    """Generate the spec.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise Spec.md generator."""
        super().__init__(args, spec, "spec.md")

    def header_parameter_doc(self, header: str, header_type: HeaderType) -> str:
        """Create documentation for a single cose header."""
        options = self._spec.header(header, header_type=header_type)
        label = options.get("coseLabel")

        custom_header = "***Custom Header***"
        if not isinstance(label, str):
            custom_header = ""
        header_format = options["format"]
        header_value = options.get("value", None)
        header_format_display = f"{header_format}"
        if isinstance(header_value, list) and len(header_value) > 0:
            header_format_display += "\n  * Supported Values:"
            for value in header_value:
                value_entry = f"\n    * {value}"
                description = None
                if header_format == "Media Type":
                    description = self._spec.content_type_description(value)
                if header_format == "HTTP Content Encoding":
                    description = self._spec.encoding_type_description(value)

                if description is not None:
                    value_entry += f" : {description.replace('\n', '\n      ')}"

                header_format_display += value_entry

        return f"""
#### `{header}`

{options.get("description")}

* Required : {options["required"]}
* Cose Label : {label} {custom_header}
* Format : {header_format_display}
    """

    def cose_header_parameters(self, header_type: HeaderType) -> str:
        """Insert details about Cose header Parameters that are defined for use."""
        headers = self._spec.all_headers(header_type)
        header_parameters_doc = ""
        for header in headers:
            header_parameters_doc += self.header_parameter_doc(header, header_type=header_type)

        return header_parameters_doc.strip()

    def generate(self) -> bool:
        """Generate a `spec.md` file from the definitions."""
        signed_doc_cddl = CDDLFile(self._args, self._spec, "signed_document")
        if not signed_doc_cddl.save_or_validate():
            return False

        self._filedata = f"""
# Catalyst Signed Document Specification

## Abstract

Project Catalyst requires a verifiable data format for the publication and validation of
large volumes of off chain information.

The Catalyst Signed Documents Specification is based on COSE
and provides the basis of this document specification.

## Motivation

As Project Catalyst decentralizes via both on-chain and off-chain mechanisms, a reliable,
standardized process for authenticating documents and their relationships is required.

## Specification

Project Catalyst generates a large volume of off chain information.
This information requires similar guarantees as on-chain data.
It needs to be verifiably published and also immutable.
However, we also require the ability to publish new versions of documents,
and for documents to be able to securely reference one another.

Catalyst Signed Documents are based on COSE.
Specifically, the COSE Sign format is used.
This allows one or more signatures to be attached to the same document.

### Signed Document CDDL Definition

{signed_doc_cddl.markdown_reference(relative_doc=self)}

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL"

    * [{signed_doc_cddl.file_name()}]({signed_doc_cddl.file_path(self)})

    ```cddl
    {{{{ include_file('./{signed_doc_cddl.file_path(self)}', indent=4) }}}}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### COSE Header Parameters

COSE documents define a set of standard COSE header parameters.
All COSE Header Parameters are protected and
*MUST* appear in the protected headers section of the document.
The COSE header parameters defined and used by Catalyst Signed Documents are as follows:

{self.cose_header_parameters(header_type=HeaderType.DOCUMENT)}

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of Metadata fields.
These fields are defined [here](./metadata.md).

### Signing Catalyst Signed Documents

Catalyst Signed Documents are based on the COSE Sign format.
This allows one or more signatures to be attached to the same document.
A catalyst signed document *MUST* have at least one valid signature attached.
Multiple signatures may also be attached to the same document, where that is required.

Each signature is contained in an array of signatures attached to the document.
The signatures contain protected headers, and the signature itself.
The headers currently defined for the signatures are:

{self.cose_header_parameters(header_type=HeaderType.SIGNATURE)}

{self.insert_copyright()}
"""
        return super().generate()
