# Generate the spec.md file

from doc_generator import DocGenerator


class SpecMd(DocGenerator):
    def __init__(self, args, spec):
        super().__init__(args, spec, "spec.md")
        print(self)

    def header_parameter_doc(self, header: str) -> str:
        """Create documentation for a single cose header."""
        options = self._spec.cose_header(header)
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
                if header_format == "IANA Media Type":
                    description = self._spec.content_type_description(value)
                if header_format == "HTTP Content Encoding":
                    description = self._spec.encoding_type_description(value)

                if description is not None:
                    value_entry += f" : {description.replace('\n', '\n      ')}"

                header_format_display += value_entry

        return f"""
#### {header}

{options.get("description")}

* Required : {options["required"]}
* Cose Label : {label} {custom_header}
* Format : {header_format_display}
    """

    def cose_header_parameters(self) -> str:
        """Insert details about Cose header Parameters that are defined for use."""
        headers = self._spec.all_cose_headers()
        header_parameters_doc = ""
        for header in headers:
            header_parameters_doc += self.header_parameter_doc(header)

        return header_parameters_doc.strip()

    def generate(self):
        """Generate a `spec.md` file from the definitions."""
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

### COSE Header Parameters

COSE documents define a set of standard COSE header parameters.
All COSE Header Parameters are protected and 
*MUST* appear in the protected headers section of the document.
The COSE header parameters defined and used by Catalyst Signed Documents are as follows:

{self.cose_header_parameters()}

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

#### `kid`

The kid is a UTF-8 encoded Catalyst ID.
Any `kid` format which conforms to the Catalyst ID specification may be used.
The Catalyst ID unambiguously defines both the signing keys and signing algorithm 
used to sign the protected portion of the document.

* Required: yes
* Cose Label: 4
* Format: UTF-8 encoded Catalyst ID

{self.insert_copyright()}
"""
        super().generate()
