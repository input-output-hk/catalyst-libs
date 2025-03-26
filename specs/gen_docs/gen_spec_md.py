# Generate the spec.md file
from common import insert_copyright


def header_parameter_doc(header, doc_data):
    """
    Create documentation for a single cose header.
    """
    options = doc_data["cose_headers"][header]
    content_types = doc_data["contentTypes"]
    encoding_types = doc_data["encodingTypes"]
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
            value_data = None
            if header_format == "IANA Media Type" and value in content_types:
                value_data = content_types[value]
            if header_format == "HTTP Content Encoding" and value in encoding_types:
                value_data = encoding_types[value]

            if value_data is not None:
                value_entry += (
                    f" : {value_data['description'].replace('\n', '\n      ')}"
                )

            header_format_display += value_entry

    return f"""
#### {header}

{options.get("description")}

* Required : {options["required"]}
* Cose Label : {label} {custom_header}
* Format : {header_format_display}
"""


def cose_header_parameters(doc_data):
    """
    Insert details about Cose header Parameters that are defined for use.
    """
    headers = doc_data["cose_headers"]
    header_order = doc_data["cose_headers_order"]
    # Make sure unordered headers get included in the documentation.
    for header in headers:
        if header not in header_order:
            header_order += header

    header_parameters_doc = ""
    for header in header_order:
        header_parameters_doc += header_parameter_doc(header, doc_data)
        headers.pop(header)

    return header_parameters_doc.strip()


def gen_spec_md(doc_defs):
    """
    Generate a `spec.md` file from the definitions.
    """
    return f"""
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

{cose_header_parameters(doc_defs)}

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of Metadata fields.
These fields are defined [here](./meta.md).

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

{insert_copyright(doc_defs)}
"""
