# Generate the spec.md file
from common import insert_copyright


def gen_docs_page_md(name: str, doc_defs: dict) -> str:
    """
    Generate an individual Documents Specification Page file from the definitions.
    """
    return f"""
# {name}

## Description

{doc_defs["docs"][name].get("description", "TODO")}

### Validation

{doc_defs["docs"][name].get("validation", "TODO")}

### Business Logic

#### Front End

{doc_defs["docs"][name].get("business_logic", {}).get("front_end", "TODO")}

#### Back End

{doc_defs["docs"][name].get("business_logic", {}).get("back_end", "TODO")}

## COSE Header Parameters

## Metadata

## Payload

## Signers

{insert_copyright(doc_defs)}
"""
