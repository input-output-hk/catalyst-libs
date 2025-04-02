# Generate the spec.md file
from common import insert_copyright, metadata_fields


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
            if (ref_data["required"] != "excluded") and ref_data["format"] == "Document Reference":
                doc_refs += f'  "{ref}": {ref_data.get("type", "Unspecified")}\n'
                if "type" in ref_data:
                    optional = ": Optional { style: { stroke: orange } }" if ref_data["required"] == "optional" else ""
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

    return doc_config + doc_tables
