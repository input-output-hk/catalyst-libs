# Generate the spec.md file
from common import insert_copyright


def gen_docs_relationship_diagram(doc_defs: dict) -> str:
    """
    Generate a D2 Relationship diagram for all documents and their references.
    """
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

    copyright = f"""
copyright: |~md
  {insert_copyright(doc_defs)}
~|  {{near: bottom-right}}
"""

    doc_tables = ""
    for doc in doc_defs["docs"]:
        doc_refs = ""
        ref_links = ""
        uuids = ""
        type_count = 0
        for uuid in doc_defs["docs"][doc]["type"]:
            uuids += f'  "type [{type_count}]": {uuid}\n'
            type_count += 1

        for ref in doc_defs["docs"][doc]["metadata"]:
            ref_data = doc_defs["docs"][doc]["metadata"][ref]
            if (ref_data["required"] != "excluded") and ref_data[
                "format"
            ] == "Document Reference":
                ref_doc = ref_data.get("type", "Unspecified")
                doc_refs += f'  "{ref}": {ref_doc}\n'
                if "type" in ref_data:
                    optional = (
                        "Optional { style: { stroke: orange } }"
                        if ref_data["required"] == "optional"
                        else ""
                    )
                    # Self Reference
                    if ref_doc == doc:
                        optional = f"({ref}) {optional}"
                    if len(optional) > 0:
                        optional = f": {optional}".strip()
                    ref_links += f'"{doc}"."{ref}"->"{ref_data["type"]}"{optional}\n'

        doc_table = f"""
"{doc}": {{
  shape: sql_table
  "content type": {doc_defs["docs"][doc]["headers"]["content type"]["value"]}
  {uuids}
  {doc_refs}
}}

{ref_links}
"""
        doc_tables += doc_table

    return doc_config + title + copyright + doc_tables
