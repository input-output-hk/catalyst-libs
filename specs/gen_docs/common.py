# Common documentation generation functions
import glob
import os
from datetime import datetime


def get_latest_file_change(directory):
    # Use glob to find all files ending with .cue in the directory
    cue_files = glob.glob(os.path.join(directory, "*.cue"))

    if not cue_files:
        return "????-??-??"

    latest_file = max(cue_files, key=os.path.getmtime)
    file_mod_time = os.path.getmtime(latest_file)
    mod_date = datetime.fromtimestamp(file_mod_time).strftime("%Y-%m-%d")

    return mod_date


def insert_copyright(doc_data, document_name=None):
    """
    Generate a copyright notice into the given document data.

    document_name: Name of the signed document we also get copyright info from.
    """
    authors = doc_data["authors"]
    copyright = doc_data["copyright"]
    global_last_modified = get_latest_file_change("../signed_docs")

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
    for author in authors:
        copyright_notice += f"|{author_title}| {author} <{authors[author]}> |\n"
        author_title = " "

    return copyright_notice


def metadata_format_link(name: str, depth: int = 0):
    """
    Metadata Format link.
    """
    link = f"metadata.md#{name.lower().replace(' ', '-')}"

    while depth > 0:
        link = f"../{link}"
        depth -= 1

    return f"[{name}]({link})"


def metadata_doc_ref_link(name: str, depth: int = 0):
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


def metadata_fields(doc_data: dict, doc_name: str = None, depth: int = 0):
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

        field_display += (
            f"| Format | {metadata_format_link(field['format'], depth)} |\n"
        )

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
                field_display += (
                    f"| {ref_heading} | {metadata_doc_ref_link(ref_doc, depth)} |\n"
                )
                ref_heading = ""
        exclusive = field.get("exclusive", None)
        if exclusive is not None:
            exclusive_title = "Exclusive"
            for ref in exclusive:
                field_display += f"| {exclusive_title} |  {ref}  |\n"
                exclusive_title = ""

        field_display += f"""<!-- markdownlint-enable MD033 -->
{field["description"]}

{field_title_level}# Validation

{field["validation"]}
"""
    return field_display.strip()
