# Generate the spec.md file
from common import insert_copyright

doc_config = """
vars: {
  d2-config: {
    layout-engine: elk
    theme-id: 4
    pad: 100
    center: true
  }
}
"""

title = """
title: |~md
  # Signed Document Relationship Hierarchy
~| {near: top-center}        
"""


def gen_doc_d2(doc: str, doc_defs: dict) -> str:
    """
    Generate an individual d2 table for an individual document.
    """
    ref_links = ""

    doc_data = doc_defs["docs"][doc]
    doc_metadata = doc_data["metadata"]
    doc_type = doc_data["type"]

    metadata_rows = ""

    for meta in doc_defs["metadata_order"]:
        if meta in doc_metadata and doc_metadata[meta]["required"] != "excluded":
            if meta == "type":
                type_count = 0
                for uuid in doc_type:
                    metadata_rows += f'  "type [{type_count}]": {uuid}\n'
                    type_count += 1
            elif doc_metadata[meta]["format"] == "Document Reference":
                ref_doc = doc_metadata[meta].get("type", "Unspecified")
                if doc_metadata[meta]["required"] == "optional":
                    metadata_rows += f'  "{meta}": {ref_doc} (Optional)\n'
                    if ref_doc == doc:
                        ref_links += f'"{doc}"."{meta}"->"{doc_metadata[meta]["type"]}": <{meta}> Optional\n'
                    else:
                        ref_links += f'"{doc}"."{meta}"->"{doc_metadata[meta]["type"]}": Optional\n'
                else:
                    metadata_rows += f'  "{meta}": {ref_doc}\n'
                    if ref_doc == doc:
                        ref_links += f'"{doc}"."{meta}"->"{doc_metadata[meta]["type"]}": <{meta}>n'
                    else:
                        ref_links += (
                            f'"{doc}"."{meta}"->"{doc_metadata[meta]["type"]}"\n'
                        )
            else:
                metadata_rows += f'  "{meta}": {doc_metadata[meta]["format"]}\n'

    return f"""
"{doc}": {{
  shape: sql_table
  "content type": {doc_defs["docs"][doc]["headers"]["content type"]["value"]}
{metadata_rows}
}}

{ref_links}
"""


def gen_doc_diagram(doc, doc_defs: dict) -> str:
    """
    Generate a D2 Relationship diagram for a single document.
    """
    doc_table = gen_doc_d2(doc, doc_defs)

    return doc_config + doc_table


def gen_docs_relationship_diagram(doc_defs: dict) -> str:
    """
    Generate a D2 Relationship diagram for all documents and their references.
    """

    copyright = f"""
copyright: |~md
  {insert_copyright(doc_defs)}
~|  {{near: bottom-right}}
"""

    doc_tables = ""
    for doc in doc_defs["docs"]:
        doc_tables += gen_doc_d2(doc, doc_defs)

    return doc_config + title + copyright + doc_tables
