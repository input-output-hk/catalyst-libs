# Common documentation generation functions


def get_latest_file_change(versions: list, doc_versions: list) -> str:
    """
    Get the largest document version date.
    """
    latest_date = ""
    for ver in versions:
        if ver["modified"] > latest_date:
            latest_date = ver["modified"]

    if doc_versions is not None:
        for ver in doc_versions:
            if ver["modified"] > latest_date:
                latest_date = ver["modified"]

    return latest_date


def insert_copyright(doc_data, changelog=True, document_name=None):
    """
    Generate a copyright notice into the given document data.

    document_name: Name of the signed document we also get copyright info from.
    """
    authors = doc_data["authors"]
    copyright = doc_data["copyright"]
    versions = copyright["versions"]
    if document_name is not None:
        authors = doc_data["docs"][document_name]["authors"] | authors
        doc_versions = doc_data["docs"][document_name]["versions"]
    else:
        doc_versions = None

    global_last_modified = get_latest_file_change(versions, doc_versions)

    copyright_year = copyright["created"][:4]
    last_modified_year = global_last_modified[:4]
    if last_modified_year != copyright_year:
        copyright_year = f"{copyright_year}-{last_modified_year}"

    copyright_notice = (
        f"""
## Copyright

| Copyright | :copyright: {copyright_year} {copyright["copyright"]} |
| --- | --- |
| License | This document is licensed under {copyright["license"]} |
| Created | {copyright["created"]} |
| Modified | {global_last_modified} |
""".strip()
        + "\n"
    )

    author_title = " Authors "
    for author in sorted(authors):
        copyright_notice += f"|{author_title}| {author} <{authors[author]}> |\n"
        author_title = " "

    if changelog:
        if document_name is None:
            versions = copyright["versions"]
        else:
            versions = doc_versions

        copyright_notice += "\n### Changelog\n\n"
        for version in versions:
            copyright_notice += f"#### {version['version']} ({version['modified']})\n\n{version['changes']}\n\n"

    return copyright_notice.strip()


def metadata_format_link(name: str, depth: int = 0):
    """
    Metadata Format link.
    """
    link = f"metadata.md#{name.lower().replace(' ', '-')}"

    while depth > 0:
        link = f"../{link}"
        depth -= 1

    return f"[{name}]({link})"


def doc_ref_link(name: str, depth: int = 0):
    """
    Metadata Document Reference link.
    """
    link = name.lower().replace(" ", "_") + ".md"

    if depth == 0:
        link = f"./docs/{link}"
    else:
        while depth > 1:
            link = f"../{link}"
            depth -= 1

    return f"[{name}]({link})"


def metadata_field_link(name: str, depth: int = 0):
    """
    Metadata Field link.
    """
    link = f"metadata.md#{name.lower().replace('`', '')}"

    while depth > 0:
        link = f"../{link}"
        depth -= 1

    return f"[`{name}`]({link})"


def metadata_fields(doc_data: dict, doc_name: str = None, depth: int = 0) -> str:
    """
    Display Metadata fields for the default set, or a specific document.
    """
    field_title_level = "###"
    if doc_name is not None:
        fields = doc_data["docs"][doc_name]["metadata"]
    else:
        fields = doc_data["metadata"]

    order = doc_data["metadata_order"]

    # make sure every field is listed in the ordering
    for field_name in fields:
        if field_name not in order:
            order += field_name

    field_display = ""
    for field_name in order:
        field = fields[field_name]

        validation = field["validation"]
        if (
            "exclusive" in field
            and field["exclusive"] is not None
            and len(field["exclusive"]) != 0
        ):
            exclusive = f"\n`{field['exclusive'][0]}`"
            if len(field["exclusive"]) > 1:
                if len(field["exclusive"]) > 2:
                    for exclude in field["exclusive"][:-1]:
                        # We break the line so it doesn't get too long.
                        exclusive += f"\n, `{exclude}"

                exclusive += f"\nand `{field['exclusive'][-1]}`"
            validation += f"\n* MUST NOT be present in any document that contains {exclusive} metadata."
        if (
            "linked_refs" in field
            and field["linked_refs"] is not None
            and len(field["linked_refs"]) != 0
        ):
            link_refs = field["linked_refs"]
            for ref in link_refs:
                validation += f"""
* The Document referenced by `{ref}`
  * MUST contain `{field_name}` metadata; AND
  * MUST match the referencing documents `{field_name}` value."""

        validation = validation.strip()

        # Don't display excluded fields in the docs for individual doc pages
        if doc_name is not None:
            if field["required"] == "excluded":
                continue

        field_display += f"""
{field_title_level} `{field_name}`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | {field["required"]} |
"""
        if field["required"] == "excluded":
            continue

        field_display += f"| Format | `{field['format']}` |\n"

        if doc_name is not None:
            if field_name == "type":
                # Display the actual documents type values
                monospace_types = []
                for type in doc_data["docs"][doc_name]["type"]:
                    monospace_types.append(f"`{type}`")
                field_display += f"| Type | {',<br/>'.join(monospace_types)} |\n"

        if field.get("multiple", False):
            field_display += f"| Multiple References | {field['multiple']} |\n"
        if "type" in field:
            ref_heading = "Valid References"
            ref_doc_names = field["type"]
            if isinstance(ref_doc_names, str):
                ref_doc_names = [ref_doc_names]
            for ref_doc in ref_doc_names:
                field_display += f"| {ref_heading} | `{ref_doc}` |\n"
                ref_heading = ""
        if "linked_refs" in field and field["linked_refs"] is not None:
            ref_heading = "Linked Reference Metadata"
            for ref_field in field["linked_refs"]:
                field_display += f"| {ref_heading} | [`{ref_field}`](#{ref_field}) |\n"
                ref_heading = ""

        exclusive = field.get("exclusive", None)
        if exclusive is not None:
            exclusive_title = "Exclusive"
            for ref in exclusive:
                field_display += f"| {exclusive_title} | `{ref}` |\n"
                exclusive_title = ""

        field_display += f"""<!-- markdownlint-enable MD033 -->
{field["description"]}

{field_title_level}# Validation

{validation}
"""
    return field_display.strip()
