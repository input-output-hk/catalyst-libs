import json
import os

# import re
# import textwrap

if __name__ == "__main__":
    SIGNED_DOCS_SPECS = "signed_doc.json"
else:
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


def external_links(env):
    """
    Insert External Links we might have used in descriptions.
    """
    doc_data = get_signed_doc_data(env)
    links = doc_data["documentationLinks"]

    link_display = ""
    for name in links:
        link_display += f"[{name}]: {links[name]}\n"

    return link_display


def metadata_fields(env, doc_name=None):
    """
    Display Metadata fields for the default set, or a specific document.
    """
    doc_data = get_signed_doc_data(env)
    if doc_name is not None:
        fields = doc_data["docs"][doc_name]["metadata"]
        field_title_level = "###"
    else:
        fields = doc_data["metadata"]
        field_title_level = "##"

    order = doc_data["metadata_order"]

    # make sure every field is listed in the ordering
    for field_name in fields:
        if field_name not in order:
            order += field_name

    field_display = ""
    for field_name in order:
        field = fields[field_name]
        field_display += f"""
{field_title_level} `{field_name}`

| Parameter | Value |
| --- | --- |
| Required | {field["required"]} |
"""
        if field["required"] != "excluded":
            field_display += f"| Format | {field['format']} |\n"
        if "multiple" in field:
            field_display += f"| Multiple References | {field['multiple']} |\n"
        if "type" in field:
            ref_heading = "Valid References"
            ref_doc_names = field["type"]
            if isinstance(ref_doc_names, str):
                ref_doc_names = [ref_doc_names]
            for ref_doc in ref_doc_names:
                field_display += f"| {ref_heading} | {ref_doc} |\n"
                ref_heading = ""

        field_display += f"""
{field["description"]}

{field_title_level}# Validation

{field["validation"]}
"""
    return field_display


def signed_doc_details(env, name):
    """
    Generate Signed Document Detailed Documentation Page.
    """
    return name + "\n" + "test\n"


# run as a program to debug the macros
if __name__ == "__main__":

    class env:
        project_dir = "/home/steven/Development/iohk/catalyst-libs/specs"

    print()
    print("### DOC TYPE DETAILS ###")
    print(doc_type_details(env))

    print()
    print("### DOC TYPE SUMMARY ###")
    print(doc_type_summary(env))

    print()
    print("### COSE HEADER PARAMETERS ###")
    print(cose_header_parameters(env))

    print()
    print("### EXTERNAL LINKS ###")
    print(external_links(env))

    print()
    print("### GLOBAL METADATA ###")
    print(metadata_fields(env))
