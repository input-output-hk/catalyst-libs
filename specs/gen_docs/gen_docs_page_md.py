# Generate the spec.md file
import json
from urllib.parse import urlparse

from common import insert_copyright, metadata_fields
from gen_docs_relationship_diagram_d2 import gen_doc_d2


def header_parameter_summary(name, doc_defs: dict) -> str:
    """
    Generate concrete Cose header parameter settings for a specific document.
    """
    headers = doc_defs["docs"][name]["headers"]
    header_docs = ""
    for header in headers:
        value = headers[header]["value"]
        if isinstance(value, list):
            value = f"[{','.join(value)}]"
        link = f"../spec.md#{header.replace(' ', '-')}"
        header_docs += f"* [{header}]({link}) = `{value}`\n"
    return header_docs.strip()


def metadata_summary(name, doc_defs: dict) -> str:
    """
    Generate concrete Metadata summary for a specific document.
    """
    return metadata_fields(doc_defs, name, depth=1)


def uri_validator(uri: str):
    try:
        result = urlparse(uri)
        return all([result.scheme in ["http", "https"], result.netloc])
    except Exception as _e:
        return False


def document_payload(name: str, doc_defs: dict) -> str:
    """
    Generate Payload Documentation
    """
    if "payload" not in doc_defs["docs"][name]:
        return "TODO"

    payload = doc_defs["docs"][name]["payload"]

    payload_docs = payload["description"] + "\n"

    if "schema" in payload:
        schema = payload["schema"]
        if uri_validator(schema):
            if schema == "https://json-schema.org/draft-07/schema":
                payload_docs += "\n**Must be a valid JSON Schema Draft 7 document.**"
            else:
                payload_docs += f"\nMust be a valid according to <{schema}>."
        else:
            payload_docs += f"""\nSchema : 
<!-- markdownlint-disable MD013 -->
```json
{json.dumps(schema, indent=2, sort_keys=True)}
```
<!-- markdownlint-enable MD013 -->
"""

    return payload_docs.strip()


def document_signers(name: str, doc_defs: dict) -> str:
    """
    Generate documentation about who may sign this document.s
    """
    signers = doc_defs["docs"][name]["signers"]
    signers_doc = ""

    for role_group in signers["roles"]:
        roles = signers["roles"][role_group]
        if roles:
            signers_doc += f"\nThe following {role_group} roles may sign documents of this type:\n\n"
            for role in roles:
                signers_doc += f"* {role}\n"

    signers_doc = signers_doc.strip()

    signers_doc += "\n\nNew versions of this document may be published by:\n\n"
    for updater in signers["update"]:
        if signers["update"][updater]:
            signers_doc += f"* {updater}\n"

    return signers_doc.strip()


def gen_docs_page_md(name: str, doc_defs: dict) -> str:
    """
    Generate an individual Documents Specification Page file from the definitions.
    """

    doc_d2 = gen_doc_d2(name, doc_defs, depth=1, stand_alone=True).strip()
    todo_msg = f"""
This specification outlines the required definitions for the current features, the document will be
incrementally improved in future iterations as more functionality and features are added.
This section will be included and updated in future iterations.
"""
    return f"""
# {name}

## Description

{doc_defs["docs"][name].get("description", todo_msg)}

```d2 layout="elk"
{doc_d2}
```

### Validation

{doc_defs["docs"][name].get("validation", todo_msg)}

### Business Logic

#### Front End

{doc_defs["docs"][name].get("business_logic", {}).get("front_end", todo_msg)}

#### Back End

{doc_defs["docs"][name].get("business_logic", {}).get("back_end", todo_msg)}

## COSE Header Parameters

{header_parameter_summary(name, doc_defs)}

## Metadata

{metadata_summary(name, doc_defs)}

## Payload

{document_payload(name, doc_defs)}

## Signers

{document_signers(name, doc_defs)}

{insert_copyright(doc_defs, document_name=name)}
"""
