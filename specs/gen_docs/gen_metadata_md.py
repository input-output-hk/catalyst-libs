# Generate the spec.md file
from common import insert_copyright, metadata_fields


def get_cddl(name, defs, found=[]):
    """Get the CDDL for a metadatum."""
    this_cddl = ""
    # Add required definitions to this one (recursive)
    for requires in defs[name]["requires"]:
        if requires not in found:
            next_cddl, found = get_cddl(requires, defs, found)
            found.append(requires)
            this_cddl += next_cddl
    this_cddl = f"{name} = {defs[name]['def']}\n{this_cddl}"

    return this_cddl, found


def metadata_types(doc_defs):
    """Generate the metadata types documentation."""
    metadata = doc_defs["metadataFormats"]
    cddl = doc_defs["cddlDefinitions"]

    metadata_types = ""

    for metadatum in metadata:
        cddl_def, _ = get_cddl(metadata[metadatum]["cddl"], cddl, [])
        cddl_def = cddl_def.strip()
        # TODO: We could check if the `cddl_def` is actually valid CDDL here.
        metadata_types += f"""
### {metadatum}

{metadata[metadatum]["description"]}

#### CDDL Specification

```cddl
{cddl_def}
```
"""

    return metadata_types.strip()


def gen_metadata_md(doc_defs):
    """Generate a `metadata.md` file from the definitions."""
    return f"""
# Metadata Fields

## Metadata Types

The following types of metadata have been defined.
All Metadata fields use one of these types.

{metadata_types(doc_defs)}

## Individual Metadata field definitions

{metadata_fields(doc_defs)}

{insert_copyright(doc_defs, changelog=False)}
"""
