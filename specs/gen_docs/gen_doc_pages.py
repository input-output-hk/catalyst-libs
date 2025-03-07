#!/usr/bin/env python

# Autogenerate Documentation Pages from the formal specification

import argparse

import json
from pathlib import Path
from gen_spec_md import gen_spec_md

# import re
# import textwrap

SIGNED_DOCS_SPECS = "../signed_doc.json"
SIGNED_DOCS_PAGES_DIR = "../../docs/src/architecture/08_concepts/catalyst_docs"


def get_signed_doc_data(spec_file):
    """
    Load the Signed Document Data from its json file.
    """
    with open(spec_file, "r") as f:
        return json.load(f)


def create_missing_doc_files():
    """
    Create basic markdown files for any document types that are missing.
    """

    docs = get_signed_doc_data()
    all_docs = docs["docs"]

    for doc_name in all_docs:
        file_name = doc_name.lower().replace(" ", "_") + ".md"
        print(f"{doc_name} : {file_name}")

        doc_path = Path(SIGNED_DOCS_PAGES_DIR, file_name)
        if doc_path.is_file():
            print("Exists")
        else:
            new_doc_contents = '# {{{{ insert_signed_doc_details( "{}" ) }}}}\n'.format(
                doc_name
            )
            doc_path.write_text(new_doc_contents)
            print("Created")

def save_or_validate(file_name, file_data, args):
    """Save a file or Validate it, depending on whats required."""
    md_file = Path(args.output).joinpath("spec.md")
    if args.generate:
        print(f"Generating {file_name}")
        md_file.write_text(file_data)
    else:
        print(f"Validating {file_name}")
        if not md_file.exists():
            print(f"Documentation file missing: {file_name}.")
            return False
        
        current_file = md_file.read_text()
        if current_file != file_data:
            print(f"Documentation not generated correctly: {file_name}.")
            return False
    return True


if __name__ == "__main__":
    # Initialize parser
    parser = argparse.ArgumentParser(description = "Markdown Documentation Generator for the Signed Documentation Specifications")
    parser.add_argument("spec", help = "Path to JSON Specification file")    
    parser.add_argument("-o", "--output", help = "Where the docs are generated/located (directory)", required=True)    
    parser.add_argument("-g", "--generate", action="store_true", help="Set to cause docs to be generated, otherwise they are validated")

    args = parser.parse_args()    

    # Check the base path exists and is a directory.
    base_path = Path(args.output)
    if not base_path.exists():
        base_path.mkdir(parents=True)
    else:
        if not base_path.is_dir():
            print("Base path is not a Directory")
            exit(1)

    docs = get_signed_doc_data(args.spec)

    good = True
    # Generate each of the files.
    spec_md = Path(args.output).joinpath("spec.md")

    good |= save_or_validate("specs.md",gen_spec_md(docs),args)

    if not good:
        print("File Comparisons Failed, Documentation is not current.")
        exit(1)

    if args.generate:
        print("Documentation Generated Successfully.")
    else:
        print("Documentation Validated Successfully.")
