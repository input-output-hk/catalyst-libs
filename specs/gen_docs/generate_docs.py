#!/usr/bin/env python

# Autogenerate Documentation Pages from the formal specification

import argparse
import difflib
import json
import re
from pathlib import Path

from common import doc_ref_link, metadata_field_link
from gen_docs_page_md import gen_docs_page_md
from gen_docs_relationship_diagram_d2 import gen_docs_relationship_diagram
from gen_metadata_md import gen_metadata_md
from gen_spec_index import gen_spec_index
from gen_spec_md import gen_spec_md
from gen_types_md import gen_types_md

SIGNED_DOCS_SPECS = "../signed_doc.json"
SIGNED_DOCS_PAGES_DIR = "../../docs/src/architecture/08_concepts/catalyst_docs"


def get_signed_doc_data(spec_file: str) -> dict:
    """
    Load the Signed Document Data from its json file.
    """
    with open(spec_file, "r") as f:
        return json.load(f)


def remove_tabs(text: str, tabstop: int = 4) -> str:
    """
    Replace tabs in the input text with spaces so that the text aligns on tab stops.

    Args:
        text (str): The input text containing tabs.
        tabstop (int): The number of characters per tab stop. Default is 8.

    Returns:
        str: Text with tabs replaced by spaces, aligned at each tab stop.
    """
    # Create a regex pattern to match any tab character
    pattern = "\t"

    def replace_tab(match):
        # Calculate the number of spaces needed to reach the next tab stop
        position = match.start()
        return " " * (tabstop - (position % tabstop))

    # Substitute tabs with spaces, using a custom replacement function
    no_tabs_text = re.sub(pattern, replace_tab, text)

    return no_tabs_text


def strip_end_whitespace(s: str) -> str:
    """
    Strip all whitespace from the end of any lines.
    """
    lines = s.splitlines()
    stripped_lines = [line.rstrip() for line in lines]
    return "\n".join(stripped_lines).strip() + "\n"


def add_doc_ref_links(file_data: str, doc_data: dict, depth: int = 0) -> str:
    """
    Add Individual Document Reference cross reference links to the document.
    All Document References in text must be as `<name>` or they will not be linked.
    """
    lines = file_data.splitlines()
    file_data = ""
    for line in lines:
        if not line.startswith("#"):
            for field_name in doc_data["docs"]:
                fieldNameRegex = f"(^|\\s)`{field_name}`(\\.|\\s|$)"
                replacement = f"\\1{doc_ref_link(field_name, depth)}\\2"
                line = re.sub(
                    fieldNameRegex,
                    replacement,
                    line,
                    flags=re.IGNORECASE | re.MULTILINE,
                )
        file_data += f"{line}\n"

    return file_data


def add_metadata_links(file_data: str, doc_data: dict, depth: int = 0) -> str:
    """
    Add metadata field documentation cross reference links to the document.
    All metadata fields in text must be as `<name>` or they will not be linked.
    """
    lines = file_data.splitlines()
    file_data = ""
    for line in lines:
        if not line.startswith("#"):
            for field_name in doc_data["metadata"]:
                fieldNameRegex = f"(^|\\s)`{field_name}`(\\.|\\s|$)"
                replacement = f"\\1{metadata_field_link(field_name, depth)}\\2"
                line = re.sub(
                    fieldNameRegex,
                    replacement,
                    line,
                    flags=re.IGNORECASE | re.MULTILINE,
                )
        file_data += f"{line}\n"

    return file_data


def code_block_aware_re_subn(
    linkNameRegex, replacement, file_data: str
) -> tuple[str, int]:
    """
    Do a multiline regex replacement, but ignore anything inside a code block.
    """
    lines = file_data.splitlines()
    new_file_data = ""
    cnt = 0
    in_code_block = False
    for line in lines:
        if line.strip().startswith("```"):
            in_code_block = not in_code_block

        if in_code_block:
            this_cnt = 0
        else:
            (line, this_cnt) = re.subn(
                linkNameRegex,
                replacement,
                line,
                flags=re.IGNORECASE,
            )
        cnt += this_cnt
        new_file_data += line + "\n"

    return (new_file_data, cnt)


def add_reference_links(file_data, doc_data) -> str:
    """
    Add Markdown reference links to the document.
    Only Reference links found to be used by the document will be added.
    """
    links = doc_data["documentationLinks"]
    linkAka = doc_data["linkAKA"]

    file_data = strip_end_whitespace(file_data)

    allLinkNames = sorted(
        list(linkAka.keys()) + list(links.keys()), key=lambda x: -len(x)
    )

    actualLinksUsed = {}
    for linkName in allLinkNames:
        escLinkName = re.escape(linkName)
        linkNameRegex = f"(^|\\s)({escLinkName})(\\.|\\s|$)"
        if linkName in linkAka:
            replacement = f"\\1[\\2][{linkAka[linkName]}]\\3"
            (file_data, cnt) = code_block_aware_re_subn(
                linkNameRegex,
                replacement,
                file_data,
            )
            if cnt > 0:
                actualLinksUsed[linkAka[linkName]] = links[linkAka[linkName]]
        else:
            replacement = r"\1[\2]\3"
            (file_data, cnt) = code_block_aware_re_subn(
                linkNameRegex,
                replacement,
                file_data,
            )
            if cnt > 0:
                actualLinksUsed[linkName] = links[linkName]

    for link in actualLinksUsed:
        file_data += f"\n[{link}]: {actualLinksUsed[link]}"

    return file_data


def save_or_validate(
    file_name: str,
    gen_func: callable,
    args: argparse.Namespace,
    doc_data: dict,
    depth=0,
) -> bool:
    """Save a file or Validate it, depending on whats required."""
    # Generate the document.
    file_data = gen_func(doc_data)
    # Remove any tabs from the file data.
    file_data = remove_tabs(file_data)
    # Add any links we find in the document.
    if file_name.endswith(".md"):
        file_data = add_reference_links(file_data, doc_data)
        file_data = add_metadata_links(file_data, doc_data, depth)
        file_data = add_doc_ref_links(file_data, doc_data, depth)

    # Remove any leading or trailing newlines and add a single newline at the end/
    # Helps make clean markdown files.
    file_data = strip_end_whitespace(file_data)
    md_file = Path(args.output).joinpath(file_name)
    if args.generate:
        print(f"Generating {file_name}")
        if md_file.exists():
            old_contents = md_file.read_text()
            if old_contents == file_data:
                print(f"{file_name} is already up-to-date.")
                return True
        md_file.write_text(file_data)
    else:
        print(f"Validating {file_name}")
        if not md_file.exists():
            print(f"Documentation file missing: {file_name}.")
            return False

        current_file = md_file.read_text()
        if current_file != file_data:
            print(f"Documentation not generated correctly: {file_name}.")
            print(current_file)
            diff = difflib.unified_diff(
                current_file.splitlines(),
                file_data.splitlines(),
                fromfile="Existing File",
                tofile="New File",
                fromfiledate="",
                tofiledate="",
                n=3,
                lineterm="\n",
            )
            for line in diff:
                print(line.rstrip())
            return False
    return True


def create_individual_doc_files(docs: dict, args: argparse.Namespace) -> bool:
    """
    Create Individual markdown files for all document types.
    """
    all_docs = docs["docs"]

    good = True

    for doc_name in all_docs:
        file_name = "docs/" + doc_name.lower().replace(" ", "_") + ".md"
        good &= save_or_validate(
            file_name,
            lambda docs: gen_docs_page_md(doc_name, docs),
            args,
            docs,
            depth=1,
        )

    return good


def check_is_dir(base_path: Path) -> bool:
    """
    Check if the path exists, and is a directory.
    Otherwise try and make it.
    Fails if it exists and is NOT a directory.
    """
    # Check the base path exists and is a directory.
    if not base_path.exists():
        base_path.mkdir(parents=True)
    else:
        if not base_path.is_dir():
            print("Base path is not a Directory")
            return False
    return True


def init_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Markdown Documentation Generator for the Signed Documentation Specifications"
    )
    parser.add_argument("spec", help="Path to JSON Specification file")
    parser.add_argument(
        "-o",
        "--output",
        help="Where the docs are generated/located (directory)",
        required=True,
    )
    parser.add_argument(
        "-g",
        "--generate",
        action="store_true",
        help="Set to cause docs to be generated, otherwise they are validated",
    )

    args = parser.parse_args()

    # Check the base path exists and is a directory.
    if not check_is_dir(Path(args.output)):
        exit(1)
    if not check_is_dir(Path(args.output, "docs")):
        exit(1)

    return args


if __name__ == "__main__":
    # Initialize parser
    args = init_parser()

    # Get the compiled documentation json file
    docs = get_signed_doc_data(args.spec)

    # We start out hoping everything is OK.
    good = True

    # Generate each of the files.
    good &= save_or_validate(".pages", gen_spec_index, args, docs)
    good &= save_or_validate("spec.md", gen_spec_md, args, docs)
    good &= save_or_validate("types.md", gen_types_md, args, docs)
    good &= save_or_validate("metadata.md", gen_metadata_md, args, docs)
    good &= create_individual_doc_files(docs, args)
    good &= save_or_validate(
        "doc_relationships.d2", gen_docs_relationship_diagram, args, docs
    )

    if not good:
        print("File Comparisons Failed, Documentation is not current.")
        exit(1)

    if args.generate:
        print("Documentation Generated Successfully.")
    else:
        print("Documentation Validated Successfully.")
