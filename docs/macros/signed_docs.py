import json
import os

# import re
# import textwrap

# SIGNED_DOCS_SPECS="signed_doc.json"
SIGNED_DOCS_SPECS = "includes/signed_doc.json"


def uuid_as_cbor(uuid):
    return f"37(h'{uuid.replace('-', '')}')"


def get_signed_doc_data(env):
    """
    Load the Signed Document Data from its json file.
    """
    full_filename = os.path.join(env.project_dir, SIGNED_DOCS_SPECS)

    with open(full_filename, "r") as f:
        return json.load(f)


def doc_type_summary(env):
    """
    Generate a Document Base Type Summary from the Document Specifications Data
    """

    try:
        doc_data = get_signed_doc_data(env)
        doc_types = doc_data["base_types"]

        doc_type_summary = """
| Base Type | [UUID] | [CBOR] |
| :--- | :--- | :--- |
"""

        for k in doc_types:
            doc_type_summary += (
                f"| {k} | `{doc_types[k]}` | `{uuid_as_cbor(doc_types[k])}` |\n"
            )

        return doc_type_summary
    except Exception as exc:
        return f"{exc}"


def name_for_uuid(doc_types, uuid):
    """
    Get the name for a document base type, given its uuid
    """
    for k in doc_types:
        if doc_types[k] == uuid:
            return k
    return "Unknown"


def name_to_spec_link(name, ref=None):
    """
    Create a link to a document type, and an optional ref inside the document.
    """
    link = "./../catalyst_docs/" + name.lower().replace(" ", "_") + ".md"
    if ref is not None:
        link += f"#{ref}"
    return link


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


def doc_type_details(env):
    """
    Generate a Document Type Detailed Summary from the Document Specifications Data
    """

    try:
        doc_data = get_signed_doc_data(env)
        doc_types = doc_data["base_types"]
        docs = doc_data["docs"]

        doc_type_details = """
| Document Type | Base Types | [CBOR] | Specification |
| :--- | :--- | :--- | :--- |
"""

        for k in docs:
            doc_type_details += f"| {k} | {base_types(docs, doc_types, k)} | {types_as_cbor(docs, k)} | [Specification]({name_to_spec_link(k)}) | \n"

        return doc_type_details
    except Exception as exc:
        return f"{exc}"


def signed_doc_details(env, name):
    """
    Generate Signed Document Detailed Documentation Page.
    """
    return name + "\n" + "test\n"


# class env:
#    project_dir = "/home/steven/Development/iohk/catalyst-libs/specs"

# if __name__ == '__main__':

#    print(doc_type_details(env))
#    print(doc_type_summary(env))
