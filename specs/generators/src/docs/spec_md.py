"""Generate the spec.md file."""

import argparse

from spec.cddl.cose import CoseHeader, HeaderType
from spec.signed_doc import SignedDoc

from .cddl_file import CDDLFile
from .doc_generator import DocGenerator


class SpecMd(DocGenerator):
    """Generate the spec.md file."""

    def __init__(self, args: argparse.Namespace, spec: SignedDoc) -> None:
        """Initialise Spec.md generator."""
        super().__init__(args, spec, "spec.md")

    def header_parameter_doc(self, header: CoseHeader) -> str:
        """Create documentation for a single cose header."""
        custom_header = "***Custom Header***"
        if not isinstance(header.cose_label, str):
            custom_header = ""

        header_format_display = f"{header.format}"
        if isinstance(header.value, list) and len(header.value) > 0:
            header_format_display += "\n  * Supported Values:"
            for value in header.value:
                value_entry = f"\n    * {value}"
                description = None
                if header.format == "Media Type":
                    description = self._spec.content_types.description(value)
                if header.format == "HTTP Content Encoding":
                    description = self._spec.encoding_types.description(value)

                if description is not None:
                    value_entry += f" : {f'\n{description}'.replace('\n', '\n      ')}"

                header_format_display += value_entry

        return f"""
#### `{header.name()}`

{header.description}

* Required : {header.required.value}
* Cose Label : {header.cose_label} {custom_header}
* Format : {header_format_display}
    """

    def cose_header_parameters(self, header_type: HeaderType) -> str:
        """Insert details about Cose header Parameters that are defined for use."""
        if header_type == HeaderType.DOCUMENT:
            headers = self._spec.cose.headers.all
        elif header_type == HeaderType.SIGNATURE:
            headers = self._spec.cose.signature_headers.all
        else:
            return ""  # No Cose Headers in metadata.

        header_parameters_doc = ""
        for header in headers:
            header_parameters_doc += self.header_parameter_doc(header)

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

While every Catalyst Signed Document is a valid COSE Sign format document,
not every COSE Sign format document is a valid Catalyst Signed Document.
The following restrictions apply:

### Unprotected Headers are not permitted

It is a requirement that any document that contains exactly the same data, must produce the same
catalyst signed document.
This means that unprotected headers, which do not form part of the data protected by
the signature are not permitted.
Any document which contains any unprotected headers is not a valid Catalyst Signed Document,
even though it may be a valid COSE Sign formatted document.

### Only defined metadata and COSE Headers are allowed

Each document type, defines a set of metadata and the COSE Headers which are allowed in that document type.
Even if the Catalyst Signed document metadata exists in this specification, IF it is not defined as
a valid metadata or COSE Header field for that particular document it may not be present.
Unexpected but otherwise valid Metadata or COSE Header fields invalidate the Catalyst Signed Document.

### No undefined metadata or unused COSE Headers may be present

COSE Header Fields which are defined by the COSE Specification, but are NOT defined as part of a
Catalyst Signed Document may not be present.
Any such COSE Header Fields present in the document render it an invalid Catalyst Signed Document.

Any metadata field that is not defined in this specification may not be present in any protected header.
Unrecognized metadata fields in a document render it an invalid Catalyst Signed Document.

### CBOR Deterministic Encoding MUST be used

The Catalyst Signed Document **MUST** be encoded using CBOR Deterministic Encoding.
The "length-first core deterministic encoding requirements" variant of deterministic encoding *MUST* be used.

### Signed Document CDDL Definition

{signed_doc_cddl.markdown_reference(relative_doc=self)}

### COSE Header Parameters

COSE documents define a set of standard COSE header parameters.
All COSE Header Parameters are protected and
*MUST* appear in the protected headers section of the document.
The COSE header parameters defined and used by Catalyst Signed Documents are as follows:

{self.cose_header_parameters(header_type=HeaderType.DOCUMENT)}

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of [Metadata fields](./metadata.md).

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
