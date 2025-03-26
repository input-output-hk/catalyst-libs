# Generate the spec.md file


def gen_spec_index(doc_defs):
    """
    Generate a `.pages` file for the base specification files.
    """
    return """
title: Catalyst Signed Document
nav:
  - Specification: spec.md
  - Metadata Fields: metadata.md
  - Document Types: types.md
"""
