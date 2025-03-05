#!/usr/bin/env python

# Autogenerate Documentation Pages for each signed document
# Will only create new pages and will not modify existing ones.

import json
from pathlib import Path

# import re
# import textwrap

SIGNED_DOCS_SPECS = "../signed_doc.json"
SIGNED_DOCS_PAGES_DIR = "../../docs/src/architecture/08_concepts/catalyst_docs"


def get_signed_doc_data():
    """
    Load the Signed Document Data from its json file.
    """
    with open(SIGNED_DOCS_SPECS, "r") as f:
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


if __name__ == "__main__":
    create_missing_doc_files()
