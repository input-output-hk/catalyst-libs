# Generate the spec.md file
from common import insert_copyright


def uuid_as_cbor(uuid):
    return f"37(h'{uuid.replace('-', '')}')"


def name_to_spec_link(name, ref=None):
    """
    Create a link to a document type, and an optional ref inside the document.
    """
    link = "./../catalyst_docs/" + name.lower().replace(" ", "_") + ".md"
    if ref is not None:
        link += f"#{ref}"
    return link


def name_for_uuid(doc_types, uuid):
    """
    Get the name for a document base type, given its uuid
    """
    for k in doc_types:
        if doc_types[k] == uuid:
            return k
    return "Unknown"


def base_types(docs, doc_types, name):
    types = docs[name]["type"]
    type_names = ""
    for sub_type in types:
        type_names += name_for_uuid(doc_types, sub_type) + "/"
    return type_names[:-1]


def types_as_cbor(docs, name):
    types = docs[name]["type"]
    type_names = "["
    for sub_type in types:
        type_names += uuid_as_cbor(sub_type) + ",<br/>"
    return type_names[:-6] + "]"


def doc_type_details(doc_data):
    """
    Generate a Document Type Detailed Summary from the Document Specifications Data
    """

    doc_types = doc_data["base_types"]
    docs = doc_data["docs"]

    doc_type_details = """
<!-- markdownlint-disable MD033 -->
| Document Type | Base Types | CBOR | 
| :--- | :--- | :--- | 
"""

    for k in docs:
        doc_type_details += f"| [{k}]({name_to_spec_link(k)}) | {base_types(docs, doc_types, k)} | {types_as_cbor(docs, k)} |\n"

    doc_type_details += "<!-- markdownlint-enable MD033 -->"

    return doc_type_details.strip()


def doc_type_summary(doc_data):
    """
    Generate a Document Base Type Summary from the Document Specifications Data
    """
    doc_types = doc_data["base_types"]

    doc_type_summary = """
| Base Type | UUID | CBOR |
| :--- | :--- | :--- |
"""

    for k in doc_types:
        doc_type_summary += (
            f"| {k} | `{doc_types[k]}` | `{uuid_as_cbor(doc_types[k])}` |\n"
        )

    return doc_type_summary.strip()


def gen_types_md(doc_defs):
    """
    Generate a `types.md` file from the definitions.
    """
    return f"""
# Document Types Table

## Document Base Types

All Document Types are defined by composing these base document types:

{doc_type_summary(doc_defs)}

## Document Types

All Defined Document Types

{doc_type_details(doc_defs)}

{insert_copyright(doc_defs)}
"""
